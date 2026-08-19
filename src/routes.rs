//! URL dispatch onto `reedhold-api`.

use crate::account;
use crate::ads;
use crate::chain;
use crate::durable;
use crate::recovery;
use crate::rep;
use crate::reply::Reply;
use crate::sealed;
use crate::state::State;
use crate::sync;
use crate::talk;
use crate::work;
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
        ("POST", "/v1/talk/open") => talk::open(state, body),
        ("POST", "/v1/talk/online") => talk::online(state, body),
        ("POST", "/v1/talk/offline") => talk::offline(state, body),
        ("POST", "/v1/talk/block") => talk::block(state, body),
        ("POST", "/v1/talk/dm") => talk::dm(state, body),
        ("POST", "/v1/talk/group") => talk::create_group(state, body),
        ("POST", "/v1/talk/invite") => talk::invite(state, body),
        ("POST", "/v1/talk/send") => talk::send(state, body),
        ("POST", "/v1/talk/remove") => talk::remove(state, body),
        ("GET", "/v1/talk/inbox") => talk::inbox(state),
        ("POST", "/v1/durable/open") => durable::open(state, body),
        ("POST", "/v1/durable/put") => durable::put(state, body),
        ("POST", "/v1/durable/get") => durable::get(state, body),
        ("POST", "/v1/durable/kill") => durable::kill(state, body),
        ("POST", "/v1/durable/repair") => durable::repair(state, body),
        ("POST", "/v1/chain/open") => chain::open(state),
        ("POST", "/v1/chain/commit") => chain::commit(state, body),
        ("GET", "/v1/chain/head") => chain::head(state),
        ("GET", "/v1/chain/headers") => chain::headers(state),
        ("POST", "/v1/chain/prove") => chain::prove(state, body),
        ("POST", "/v1/chain/verify") => chain::verify(state, body),
        ("POST", "/v1/rep/open") => rep::open(state),
        ("POST", "/v1/rep/seed") => rep::seed(state, body),
        ("POST", "/v1/rep/react") => rep::react(state, body),
        ("POST", "/v1/rep/identity") => rep::identity(state, body),
        ("POST", "/v1/rep/content") => rep::content(state, body),
        ("POST", "/v1/rep/transfer") => rep::transfer(body),
        ("POST", "/v1/ads/open") => ads::open(state),
        ("POST", "/v1/ads/post") => ads::post(state, body),
        ("POST", "/v1/ads/register") => ads::register(state, body),
        ("POST", "/v1/ads/bid") => ads::bid(state, body),
        ("POST", "/v1/ads/clear") => ads::clear(state, body),
        ("POST", "/v1/ads/select") => ads::select(state, body),
        ("POST", "/v1/ads/hide") => ads::hide(state, body),
        ("POST", "/v1/ads/settle") => ads::settle(state, body),
        ("POST", "/v1/ads/bucket") => ads::bucket(body),
        ("POST", "/v1/work/open") => work::open(state),
        ("POST", "/v1/work/record") => work::record(state, body),
        ("POST", "/v1/work/view") => work::view(state, body),
        ("POST", "/v1/work/transfer") => work::transfer(state, body),
        _ => Reply { status: 404, body: crate::body::error_json("not found") },
    }
}
