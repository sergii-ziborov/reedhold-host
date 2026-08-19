//! URL dispatch onto `reedhold-api`.

use crate::account;
use crate::recovery;
use crate::reply::Reply;
use crate::sealed;
use crate::state::State;
use crate::sync;
use reedhold_api::{advertising_limits, invariants};
use std::sync::Mutex;

/// Route one request. `body` is the raw JSON payload (empty on GET).
#[must_use]
pub fn dispatch(state: &Mutex<State>, method: &str, url: &str, body: &str) -> Reply {
    if method == "OPTIONS" {
        return Reply { status: 204, body: String::new() };
    }
    match (method, url) {
        ("GET", "/health") => crate::reply::ok("{\"ok\":true}"),
        ("GET", "/v1/invariants") => crate::reply::json(&invariants()),
        ("GET", "/v1/advertising/limits") => crate::reply::json(&advertising_limits()),
        ("POST", "/v1/account") => account::create(state, body),
        ("POST", "/v1/account/restore") => account::restore(state, body),
        ("GET", "/v1/account") => account::account(state),
        ("GET", "/v1/account/manifest") => account::manifest(state),
        ("GET", "/v1/account/history") => account::history(state),
        ("POST", "/v1/account/emit") => account::emit(state, body),
        ("POST", "/v1/account/verify") => account::verify(state, body),
        ("POST", "/v1/account/password") => account::password(state, body),
        ("POST", "/v1/account/split") => recovery::split(state, body),
        ("POST", "/v1/account/combine") => recovery::combine(state, body),
        ("POST", "/v1/account/sealed") => sealed::emit_sealed(state, body),
        ("POST", "/v1/account/open") => sealed::open_sealed(body),
        ("POST", "/v1/sync/plan") => sync::plan(body),
        _ => Reply { status: 404, body: crate::body::error_json("not found") },
    }
}
