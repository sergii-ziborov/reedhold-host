//! Contacts, public aliases, and chat list. Aliases never enter crypto.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use reedhold_api::{TOPIC_CATALOG, TalkNet};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize)]
struct NickBody {
    nick: String,
}

#[derive(Deserialize)]
struct ContactBody {
    identity: String,
    messaging_public: String,
    petname: Option<String>,
}

#[derive(Deserialize)]
struct IdentityBody {
    identity: String,
}

#[derive(Serialize)]
struct ChatsOut {
    nick: Option<String>,
    contacts: Vec<reedhold_api::ContactView>,
    groups: Vec<reedhold_api::CircleView>,
    rooms: Vec<reedhold_api::RoomView>,
    interests: Vec<String>,
    catalog: Vec<String>,
    threads: std::collections::BTreeMap<String, Vec<reedhold_api::TalkView>>,
}

pub(crate) fn claim(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<NickBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let State { seats, aliases, .. } = host;
        let session = seats.get(seat).ok_or_else(|| "no unlocked session".to_owned())?;
        aliases.claim(session, &parsed.nick).map_err(|error| error.to_string())
    })
}

pub(crate) fn lookup(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<NickBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| match host.aliases.lookup(&parsed.nick) {
        Some(view) => Ok(view),
        None => Err("alias not found".to_owned()),
    })
}

pub(crate) fn add_contact(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<ContactBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let petname = parsed.petname.unwrap_or_default();
    state::mutate(state, |host| {
        let view = host.seat_mut(seat)?.add_contact(
            &parsed.identity,
            &parsed.messaging_public,
            &petname,
        )
        .map_err(|error| error.to_string())?;
        ensure_talk(host)?;
        host.join_talk(&parsed.identity);
        Ok(view)
    })
}

pub(crate) fn remove_contact(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<IdentityBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        host.seat_mut(seat)?.remove_contact(&parsed.identity).map_err(|error| error.to_string())
    })
}

pub(crate) fn contacts(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| Ok(host.seat(seat)?.contacts()))
}

pub(crate) fn circles(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| Ok(host.seat(seat)?.circles()))
}

pub(crate) fn chats(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| {
        let session = host.seat(seat)?;
        Ok(ChatsOut {
            nick: host.aliases.nick_of(&session.peer_hex()),
            contacts: session.contacts(),
            groups: session.circles(),
            rooms: host.rooms.list(session),
            interests: host.rooms.interests(),
            catalog: TOPIC_CATALOG.iter().map(|topic| (*topic).to_owned()).collect(),
            threads: session.threads(),
        })
    })
}

pub(crate) fn ensure_talk(host: &mut State) -> Result<(), String> {
    if host.talk.is_some() {
        return Ok(());
    }
    if host.seats.is_empty() {
        return Err("no unlocked session".to_owned());
    }
    let mut candidates = Vec::new();
    for session in host.seats.values() {
        candidates.push(session.peer_hex());
        for contact in session.contacts() {
            candidates.push(contact.identity);
        }
    }
    for byte in 10_u8..=16 {
        candidates.push(format!("{byte:02x}").repeat(32));
    }
    let mut talk = TalkNet::open(1, &"00".repeat(32), &candidates, None, Some(2))
        .map_err(|error| error.to_string())?;
    let peers: Vec<String> = host.seats.values().map(reedhold_api::Session::peer_hex).collect();
    for peer in peers {
        talk.online(&peer).map_err(|error| error.to_string())?;
    }
    host.talk = Some(talk);
    Ok(())
}
