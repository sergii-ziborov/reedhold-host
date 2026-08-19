//! Account create / restore / emit / verify / password / history.

use crate::body::{CreateOut, EmitBody, PasswordBody, SeatOut, SecretBody, VerifyBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::{self, State};
use reedhold_api::Session;
use std::sync::Mutex;

pub(crate) fn create(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SecretBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match Session::create(&parsed.password, &parsed.device_secret) {
        Ok(created) => {
            let account = created.session.view();
            let manifest = created.manifest;
            let seat = match state.lock() {
                Ok(mut lock) => lock.issue_seat(created.session),
                Err(_) => return fail("lock"),
            };
            json(&CreateOut { account, manifest, seat })
        }
        Err(error) => fail(&error.to_string()),
    }
}

pub(crate) fn restore(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SecretBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let Some(manifest) = parsed.manifest_hex.as_deref() else {
        return bad("manifest_hex is required");
    };
    match Session::restore(manifest, &parsed.password, &parsed.device_secret) {
        Ok(session) => {
            let account = session.view();
            let seat = match state.lock() {
                Ok(mut lock) => lock.issue_seat(session),
                Err(_) => return fail("lock"),
            };
            json(&SeatOut { account, seat })
        }
        Err(error) => fail(&error.to_string()),
    }
}

pub(crate) fn account(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| host.view(seat))
}

pub(crate) fn manifest(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| host.with(seat, Session::manifest))
}

pub(crate) fn history(state: &Mutex<State>, seat: &str) -> Reply {
    state::inspect(state, |host| host.with(seat, Session::history))
}

pub(crate) fn emit(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<EmitBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| host.with_mut(seat, |session| session.emit(&parsed.kind, &parsed.payload)))
}

pub(crate) fn verify(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<VerifyBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| host.with(seat, |session| session.verify(&parsed.event_hex)))
}

pub(crate) fn password(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<PasswordBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| host.with_mut(seat, |session| session.change_password(&parsed.password)))
}
