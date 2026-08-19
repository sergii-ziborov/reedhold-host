//! URL dispatch onto `reedhold-api`.

use crate::body::{
    EmitBody, PasswordBody, SecretBody, VerifyBody, error_json, read_json, write_json,
};
use reedhold_api::{Session, advertising_limits, invariants};
use std::sync::Mutex;
use tiny_http::Request;

/// HTTP reply.
pub struct Reply {
    /// Status code.
    pub status: u16,
    /// JSON body.
    pub body: String,
}

/// In-memory unlocked session.
#[derive(Default)]
pub struct State {
    session: Option<Session>,
}

/// Route one request.
pub fn dispatch(state: &Mutex<State>, method: &str, url: &str, request: &mut Request) -> Reply {
    if method == "OPTIONS" {
        return Reply { status: 204, body: String::new() };
    }
    match (method, url) {
        ("GET", "/health") => ok(&serde_health()),
        ("GET", "/v1/invariants") => json(&invariants()),
        ("GET", "/v1/advertising/limits") => json(&advertising_limits()),
        ("POST", "/v1/account") => create(state, request),
        ("POST", "/v1/account/restore") => restore(state, request),
        ("GET", "/v1/account") => account(state),
        ("POST", "/v1/account/emit") => emit(state, request),
        ("POST", "/v1/account/verify") => verify(state, request),
        ("POST", "/v1/account/password") => password(state, request),
        _ => Reply { status: 404, body: error_json("not found") },
    }
}

fn create(state: &Mutex<State>, request: &mut Request) -> Reply {
    let body = match read_json::<SecretBody>(request) {
        Ok(body) => body,
        Err(error) => return bad(&error),
    };
    match Session::create(&body.password, &body.device_secret) {
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

fn restore(state: &Mutex<State>, request: &mut Request) -> Reply {
    let body = match read_json::<SecretBody>(request) {
        Ok(body) => body,
        Err(error) => return bad(&error),
    };
    let Some(manifest) = body.manifest_hex.as_deref() else {
        return bad("manifest_hex is required");
    };
    match Session::restore(manifest, &body.password, &body.device_secret) {
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

fn account(state: &Mutex<State>) -> Reply {
    match session(state) {
        Ok(view) => json(&view),
        Err(error) => fail(error),
    }
}

fn emit(state: &Mutex<State>, request: &mut Request) -> Reply {
    let body = match read_json::<EmitBody>(request) {
        Ok(body) => body,
        Err(error) => return bad(&error),
    };
    match session_mut(state, |session| session.emit(&body.kind, &body.payload)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

fn verify(state: &Mutex<State>, request: &mut Request) -> Reply {
    let body = match read_json::<VerifyBody>(request) {
        Ok(body) => body,
        Err(error) => return bad(&error),
    };
    match session_mut(state, |session| session.verify(&body.event_hex)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

fn password(state: &Mutex<State>, request: &mut Request) -> Reply {
    let body = match read_json::<PasswordBody>(request) {
        Ok(body) => body,
        Err(error) => return bad(&error),
    };
    match session_mut(state, |session| session.change_password(&body.password)) {
        Ok(view) => json(&view),
        Err(error) => fail(&error),
    }
}

fn session(state: &Mutex<State>) -> Result<reedhold_api::AccountView, &'static str> {
    let lock = state.lock().map_err(|_| "lock")?;
    lock.session.as_ref().map(Session::view).ok_or("no unlocked session")
}

fn session_mut<T, E: ToString>(
    state: &Mutex<State>,
    op: impl FnOnce(&mut Session) -> Result<T, E>,
) -> Result<T, String> {
    let mut lock = state.lock().map_err(|_| "lock".to_owned())?;
    let session = lock.session.as_mut().ok_or_else(|| "no unlocked session".to_owned())?;
    op(session).map_err(|error| error.to_string())
}

fn json<T: serde::Serialize>(value: &T) -> Reply {
    match write_json(value) {
        Ok(body) => Reply { status: 200, body },
        Err(error) => fail(&error),
    }
}

fn ok(body: &str) -> Reply {
    Reply { status: 200, body: body.to_owned() }
}

fn bad(message: &str) -> Reply {
    Reply { status: 400, body: error_json(message) }
}

fn fail(message: &str) -> Reply {
    Reply { status: 409, body: error_json(message) }
}

fn serde_health() -> String {
    "{\"ok\":true}".to_owned()
}

#[derive(serde::Serialize)]
struct CreateOut {
    account: reedhold_api::AccountView,
    manifest: reedhold_api::ManifestView,
}
