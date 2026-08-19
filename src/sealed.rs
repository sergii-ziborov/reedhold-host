//! Conversation-key sealed payloads.

use crate::body::{OpenBody, PlainOut, SealedBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::State;
use reedhold_api::Session;
use std::sync::Mutex;

pub(crate) fn emit_sealed(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<SealedBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    crate::state::mutate(state, |host| {
        host.with_mut(seat, |session| session.emit_sealed(&parsed.conversation_key, &parsed.plaintext))
    })
}

pub(crate) fn open_sealed(body: &str) -> Reply {
    let parsed = match parse_json::<OpenBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match Session::open_sealed(&parsed.conversation_key, &parsed.envelope_hex) {
        Ok(plaintext) => json(&PlainOut { plaintext }),
        Err(error) => fail(&error.to_string()),
    }
}


