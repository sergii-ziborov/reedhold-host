//! Conversation-key sealed payloads.

use crate::body::{OpenBody, PlainOut, SealedBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::State;
use reedhold_api::Session;
use std::sync::Mutex;

pub(crate) fn emit_sealed(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SealedBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match mutate(state, |session| session.emit_sealed(&parsed.conversation_key, &parsed.plaintext))
    {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
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

fn mutate<T, E: ToString>(
    state: &Mutex<State>,
    op: impl FnOnce(&mut Session) -> Result<T, E>,
) -> Result<T, String> {
    let mut guard = state.lock().map_err(|_| "lock".to_owned())?;
    guard.with_mut(op)
}
