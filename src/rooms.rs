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

pub(crate) fn join(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<TopicBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        host.rooms.join(session, &parsed.topic).map_err(|error| error.to_string())
    })
}

pub(crate) fn leave(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<TopicBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        host.rooms.leave(session, &parsed.topic).map_err(|error| error.to_string())
    })
}

pub(crate) fn post(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<RoomPostBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let session = host.session.as_mut().ok_or_else(|| "no unlocked session".to_owned())?;
        host.rooms.post(session, &parsed.topic, &parsed.text).map_err(|error| error.to_string())
    })
}

pub(crate) fn list(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| {
        let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
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
