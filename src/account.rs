//! Account create / restore / emit / verify / password / history.

use crate::body::{CreateOut, EmitBody, PasswordBody, SecretBody, VerifyBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::State;
use reedhold_api::Session;
use std::sync::Mutex;

pub(crate) fn create(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SecretBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match Session::create(&parsed.password, &parsed.device_secret) {
        Ok(created) => {
            let view = created.session.view();
            let manifest = created.manifest;
            if let Ok(mut lock) = state.lock() {
                lock.session = Some(created.session);
            }
            json(&CreateOut { account: view, manifest })
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
            let view = session.view();
            if let Ok(mut lock) = state.lock() {
                lock.session = Some(session);
            }
            json(&view)
        }
        Err(error) => fail(&error.to_string()),
    }
}

pub(crate) fn account(state: &Mutex<State>) -> Reply {
    match locked(state).and_then(|guard| guard.view().map_err(str::to_owned)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

pub(crate) fn manifest(state: &Mutex<State>) -> Reply {
    match locked(state).and_then(|guard| guard.with(Session::manifest)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

pub(crate) fn history(state: &Mutex<State>) -> Reply {
    match locked(state).and_then(|guard| guard.with(Session::history)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

pub(crate) fn emit(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<EmitBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match mutate(state, |session| session.emit(&parsed.kind, &parsed.payload)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

pub(crate) fn verify(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<VerifyBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match locked(state).and_then(|guard| guard.with(|session| session.verify(&parsed.event_hex))) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

pub(crate) fn password(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<PasswordBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match mutate(state, |session| session.change_password(&parsed.password)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

fn locked(state: &Mutex<State>) -> Result<std::sync::MutexGuard<'_, State>, String> {
    state.lock().map_err(|_| "lock".to_owned())
}

fn mutate<T, E: ToString>(
    state: &Mutex<State>,
    op: impl FnOnce(&mut Session) -> Result<T, E>,
) -> Result<T, String> {
    let mut guard = state.lock().map_err(|_| "lock".to_owned())?;
    guard.with_mut(op)
}
