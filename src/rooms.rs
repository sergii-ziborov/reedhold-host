//! Public topic rooms. Slugs stay off the signed event.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct TopicBody {
    topic: String,
}

#[derive(Deserialize)]
struct RoomPostBody {
    topic: String,
    text: String,
}

#[derive(Deserialize)]
struct InterestsBody {
    topics: Vec<String>,
}

pub(crate) fn join(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<TopicBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let State { seats, rooms, .. } = host;
        let session = seats.get(seat).ok_or_else(|| "no unlocked session".to_owned())?;
        rooms.join(session, &parsed.topic).map_err(|error| error.to_string())
    })
}

pub(crate) fn leave(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<TopicBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        let State { seats, rooms, .. } = host;
        let session = seats.get(seat).ok_or_else(|| "no unlocked session".to_owned())?;
        rooms.leave(session, &parsed.topic).map_err(|error| error.to_string())
    })
}

pub(crate) fn post(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<RoomPostBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let State { seats, rooms, .. } = host;
        let session = seats.get_mut(seat).ok_or_else(|| "no unlocked session".to_owned())?;
        rooms.post(session, &parsed.topic, &parsed.text).map_err(|error| error.to_string())
    })
}

pub(crate) fn list(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| {
        let session = host.seat(seat)?;
        Ok(host.rooms.list(session))
    })
}

pub(crate) fn set_interests(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<InterestsBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        host.rooms.set_interests(&parsed.topics);
        Ok(host.rooms.interests())
    })
}

pub(crate) fn interests(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| Ok(host.rooms.interests()))
}

pub(crate) fn catalog() -> Reply {
    crate::reply::json(&reedhold_api::RoomBoard::catalog())
}
