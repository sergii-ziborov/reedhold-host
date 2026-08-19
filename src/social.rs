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
}

pub(crate) fn claim(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<NickBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        host.aliases.claim(session, &parsed.nick).map_err(|error| error.to_string())
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

pub(crate) fn add_contact(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<ContactBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let petname = parsed.petname.unwrap_or_default();
    state::mutate(state, |host| {
        let session = host.session.as_mut().ok_or_else(|| "no unlocked session".to_owned())?;
        let view = session
            .add_contact(&parsed.identity, &parsed.messaging_public, &petname)
            .map_err(|error| error.to_string())?;
        host.talk = None;
        ensure_talk(host)?;
        Ok(view)
    })
}

pub(crate) fn remove_contact(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<IdentityBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        host.session
            .as_mut()
            .ok_or_else(|| "no unlocked session".to_owned())?
            .remove_contact(&parsed.identity)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn contacts(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| {
        Ok(host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?.contacts())
    })
}

pub(crate) fn circles(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| {
        Ok(host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?.circles())
    })
}

pub(crate) fn chats(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| {
        let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        Ok(ChatsOut {
            nick: host.aliases.nick_of(&session.peer_hex()),
            contacts: session.contacts(),
            groups: session.circles(),
            rooms: host.rooms.list(session),
            interests: host.rooms.interests(),
            catalog: TOPIC_CATALOG.iter().map(|topic| (*topic).to_owned()).collect(),
        })
    })
}

pub(crate) fn ensure_talk(host: &mut State) -> Result<(), String> {
    if host.talk.is_some() {
        return Ok(());
    }
    let session = host.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
    let me = session.peer_hex();
    let mut candidates = vec![me.clone()];
    for contact in session.contacts() {
        candidates.push(contact.identity);
    }
    for byte in 10_u8..=16 {
        candidates.push(format!("{byte:02x}").repeat(32));
    }
    let mut talk = TalkNet::open(1, &"00".repeat(32), &candidates, None, Some(2))
        .map_err(|error| error.to_string())?;
    talk.online(&me).map_err(|error| error.to_string())?;
    host.talk = Some(talk);
    Ok(())
}
