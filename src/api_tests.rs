//! Route tests. No live socket.

use crate::{State, dispatch};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct Created {
    account: IdentityOnly,
    manifest: ManifestOnly,
    seat: String,
}

#[derive(Deserialize)]
struct IdentityOnly {
    identity: String,
}

#[derive(Deserialize)]
struct ManifestOnly {
    manifest_hex: String,
}

#[derive(Deserialize)]
struct EventBare {
    kind: String,
    event_hex: String,
}

#[derive(Deserialize)]
struct ShareBare {
    index: u8,
    body_hex: String,
}

#[derive(Deserialize)]
struct PlanBare {
    company_required: bool,
    blocking_is_fatal: bool,
    relays: Vec<String>,
}

fn secret() -> String {
    "ab".repeat(32)
}

fn call(state: &Mutex<State>, method: &str, url: &str, body: &str, seat: &str) -> (u16, String) {
    let reply = dispatch(state, method, url, body, seat);
    (reply.status, reply.body)
}

fn create(state: &Mutex<State>) -> Created {
    let body = format!("{{\"password\":\"pw\",\"device_secret\":\"{}\"}}", secret());
    let (status, raw) = call(state, "POST", "/v1/account", &body, "");
    assert_eq!(status, 200, "{raw}");
    blazingly_json::from_str(&raw).expect("created")
}

#[test]
fn health_and_limits() {
    let state = Mutex::new(State::default());
    let (status, body) = call(&state, "GET", "/health", "", "");
    assert_eq!(status, 200);
    assert!(body.contains("true"));
    let (status, body) = call(&state, "GET", "/v1/advertising/limits", "", "");
    assert_eq!(status, 200);
    assert!(body.contains("market_only"));
    let (status, _) = call(&state, "GET", "/nope", "", "");
    assert_eq!(status, 404);
}

#[test]
fn create_emit_verify_restore() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    assert!(created.account.identity.starts_with("reedhold:identity:"));
    let seat = &created.seat;
    let (status, raw) =
        call(&state, "POST", "/v1/account/emit", "{\"kind\":\"post\",\"payload\":\"hello\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let event: EventBare = blazingly_json::from_str(&raw).expect("event");
    assert_eq!(event.kind, "post");
    let verify = format!("{{\"event_hex\":\"{}\"}}", event.event_hex);
    let (status, raw) = call(&state, "POST", "/v1/account/verify", &verify, seat);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) = call(&state, "GET", "/v1/account/history", "", seat);
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains("hello") || raw.contains("post"));
    let restore = format!(
        "{{\"password\":\"pw\",\"device_secret\":\"{}\",\"manifest_hex\":\"{}\"}}",
        secret(),
        created.manifest.manifest_hex
    );
    let (status, raw) = call(&state, "POST", "/v1/account/restore", &restore, "");
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains(&created.account.identity));
}

#[test]
fn split_then_combine() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    let (status, raw) =
        call(&state, "POST", "/v1/account/split", "{\"threshold\":2,\"total\":3}", &created.seat);
    assert_eq!(status, 200, "{raw}");
    let shares: Vec<ShareBare> = blazingly_json::from_str(&raw).expect("shares");
    assert_eq!(shares.len(), 3);
    let combine = format!(
        "{{\"password\":\"new\",\"device_secret\":\"{}\",\"threshold\":2,\"shares\":[{},{}]}}",
        secret(),
        share_json(&shares[0]),
        share_json(&shares[2])
    );
    let (status, raw) = call(&state, "POST", "/v1/account/combine", &combine, "");
    assert_eq!(status, 200, "{raw}");
    let restored: Created = blazingly_json::from_str(&raw).expect("combined");
    assert_eq!(restored.account.identity, created.account.identity);
}

#[test]
fn sync_plan_never_requires_company() {
    let state = Mutex::new(State::default());
    let candidates: Vec<String> = (1_u8..=8).map(|byte| format!("{byte:02x}").repeat(32)).collect();
    let listed = candidates.iter().map(|hex| format!("\"{hex}\"")).collect::<Vec<_>>().join(",");
    let body = format!(
        "{{\"epoch\":5,\"candidates\":[{listed}],\"company\":\"{}\",\"relay_count\":3}}",
        "99".repeat(32)
    );
    let (status, raw) = call(&state, "POST", "/v1/sync/plan", &body, "");
    assert_eq!(status, 200, "{raw}");
    let plan: PlanBare = blazingly_json::from_str(&raw).expect("plan");
    assert!(!plan.company_required);
    assert!(!plan.blocking_is_fatal);
    assert_eq!(plan.relays.len(), 3);
}

fn share_json(share: &ShareBare) -> String {
    format!("{{\"index\":{},\"body_hex\":\"{}\"}}", share.index, share.body_hex)
}
