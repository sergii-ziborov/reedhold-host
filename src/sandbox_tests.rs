//! Overlay route tests. No live socket.

use crate::{State, dispatch};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct Created {
    account: IdentityOnly,
    seat: String,
}

#[derive(Deserialize)]
struct IdentityOnly {
    identity: String,
    messaging_public: String,
}

#[derive(Deserialize)]
struct Flag {
    ok: bool,
}

#[derive(Deserialize)]
struct CircleBare {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct ObjectBare {
    id: String,
    k: u8,
    n: u8,
    holders: Vec<String>,
}

#[derive(Deserialize)]
struct PayloadBare {
    payload: String,
}

#[derive(Deserialize)]
struct HeaderBare {
    height: u64,
    encoded_len: u32,
}

#[derive(Deserialize)]
struct ProofBare {
    root: String,
    index: u32,
    siblings: Vec<String>,
}

#[derive(Deserialize)]
struct SelectBare {
    campaign: Option<String>,
}

#[derive(Deserialize)]
struct WorkBare {
    credits: u64,
    eligible: bool,
    repair: u32,
}

fn hex(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn call(state: &Mutex<State>, method: &str, url: &str, body: &str) -> (u16, String) {
    let reply = dispatch(state, method, url, body, "");
    (reply.status, reply.body)
}

fn as_seat(state: &Mutex<State>, method: &str, url: &str, body: &str, seat: &str) -> (u16, String) {
    let reply = dispatch(state, method, url, body, seat);
    (reply.status, reply.body)
}

fn create(state: &Mutex<State>) -> Created {
    let body = format!("{{\"password\":\"pw\",\"device_secret\":\"{}\"}}", hex(0xab));
    let (status, raw) = call(state, "POST", "/v1/account", &body);
    assert_eq!(status, 200, "{raw}");
    blazingly_json::from_str(&raw).expect("created")
}

fn identity_hex(uri: &str) -> String {
    uri.rsplit(':').next().unwrap_or(uri).to_owned()
}

#[test]
fn talk_opens_and_creates_a_group() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    let me = identity_hex(&created.account.identity);
    let extras: Vec<String> = (10_u8..=16).map(hex).collect();
    let listed = std::iter::once(me.clone())
        .chain(extras)
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(",");
    let open = format!("{{\"epoch\":4,\"candidates\":[{listed}],\"relay_count\":2}}");
    let (status, raw) = call(&state, "POST", "/v1/talk/open", &open);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) =
        as_seat(&state, "POST", "/v1/talk/group", "{\"name\":\"room\"}", &created.seat);
    assert_eq!(status, 200, "{raw}");
    let group: CircleBare = blazingly_json::from_str(&raw).expect("group");
    assert_eq!(group.name, "room");
    assert!(!group.id.is_empty());
    let (status, raw) = as_seat(&state, "GET", "/v1/talk/inbox", "", &created.seat);
    assert_eq!(status, 200, "{raw}");
    let _ = created.account.messaging_public;
}

#[test]
fn durable_survives_two_dead_holders() {
    let state = Mutex::new(State::default());
    let holders: Vec<String> = (1_u8..=8).map(hex).collect();
    let listed = holders.iter().map(|id| format!("\"{id}\"")).collect::<Vec<_>>().join(",");
    let open = format!("{{\"holders\":[{listed}],\"company\":\"{}\"}}", hex(99));
    let (status, raw) = call(&state, "POST", "/v1/durable/open", &open);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) = call(&state, "POST", "/v1/durable/put", "{\"payload\":\"hello-grid\"}");
    assert_eq!(status, 200, "{raw}");
    let stored: ObjectBare = blazingly_json::from_str(&raw).expect("object");
    assert_eq!(stored.k, 4);
    assert_eq!(stored.n, 6);
    let live: Vec<String> =
        stored.holders.into_iter().filter(|holder| !holder.is_empty()).collect();
    let kill = format!("{{\"holder\":\"{}\"}}", live[0]);
    let (status, raw) = call(&state, "POST", "/v1/durable/kill", &kill);
    assert_eq!(status, 200, "{raw}");
    let kill = format!("{{\"holder\":\"{}\"}}", live[1]);
    let (status, raw) = call(&state, "POST", "/v1/durable/kill", &kill);
    assert_eq!(status, 200, "{raw}");
    let fetch = format!("{{\"id\":\"{}\"}}", stored.id);
    let (status, raw) = call(&state, "POST", "/v1/durable/get", &fetch);
    assert_eq!(status, 200, "{raw}");
    let object: PayloadBare = blazingly_json::from_str(&raw).expect("payload");
    assert_eq!(object.payload, "hello-grid");
}

#[test]
fn chain_headers_stay_compact() {
    let state = Mutex::new(State::default());
    let (status, raw) = call(&state, "POST", "/v1/chain/open", "");
    assert_eq!(status, 200, "{raw}");
    let commit = format!(
        "{{\"epoch\":1,\"identity\":\"{}\",\"groups\":\"{}\",\"storage\":\"{}\"}}",
        hex(1),
        hex(2),
        hex(3)
    );
    let (status, raw) = call(&state, "POST", "/v1/chain/commit", &commit);
    assert_eq!(status, 200, "{raw}");
    let first: HeaderBare = blazingly_json::from_str(&raw).expect("header");
    let commit =
        format!("{{\"epoch\":2,\"identity\":\"{}\",\"groups\":\"\",\"storage\":\"\"}}", hex(9));
    let (status, raw) = call(&state, "POST", "/v1/chain/commit", &commit);
    assert_eq!(status, 200, "{raw}");
    let second: HeaderBare = blazingly_json::from_str(&raw).expect("header");
    assert_eq!(first.encoded_len, second.encoded_len);
    assert_eq!(second.height, first.height + 1);
    let prove =
        format!("{{\"leaves\":[\"{}\",\"{}\",\"{}\"],\"index\":1}}", hex(1), hex(2), hex(3));
    let (status, raw) = call(&state, "POST", "/v1/chain/prove", &prove);
    assert_eq!(status, 200, "{raw}");
    let proof: ProofBare = blazingly_json::from_str(&raw).expect("proof");
    let verify = format!(
        "{{\"leaf\":\"{}\",\"root\":\"{}\",\"index\":{},\"siblings\":[{}]}}",
        hex(2),
        proof.root,
        proof.index,
        proof.siblings.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(",")
    );
    let (status, raw) = call(&state, "POST", "/v1/chain/verify", &verify);
    assert_eq!(status, 200, "{raw}");
    let flag: Flag = blazingly_json::from_str(&raw).expect("flag");
    assert!(flag.ok);
}

#[test]
fn reputation_cannot_be_transferred() {
    let state = Mutex::new(State::default());
    let (status, raw) = call(&state, "POST", "/v1/rep/open", "");
    assert_eq!(status, 200, "{raw}");
    let transfer = format!("{{\"from\":\"{}\",\"to\":\"{}\",\"amount\":10}}", hex(2), hex(3));
    let (status, raw) = call(&state, "POST", "/v1/rep/transfer", &transfer);
    assert_eq!(status, 409, "{raw}");
    assert!(raw.contains("error"));
}

#[test]
fn ads_select_has_no_user_id() {
    let state = Mutex::new(State::default());
    let (status, raw) = call(&state, "POST", "/v1/ads/open", "");
    assert_eq!(status, 200, "{raw}");
    let post = format!(
        "{{\"advertiser\":\"{}\",\"campaign\":\"{}\",\"payload\":\"{}\",\"topic\":\"{}\",\"bucket_min\":0,\"bucket_max\":5,\"budget\":500,\"expiry\":9}}",
        hex(1),
        hex(1),
        hex(50),
        hex(7)
    );
    let (status, raw) = call(&state, "POST", "/v1/ads/post", &post);
    assert_eq!(status, 200, "{raw}");
    let register = format!("{{\"id\":\"{}\",\"strength\":5000}}", hex(9));
    let (status, raw) = call(&state, "POST", "/v1/ads/register", &register);
    assert_eq!(status, 200, "{raw}");
    let bid = format!(
        "{{\"advertiser\":\"{}\",\"campaign\":\"{}\",\"topic\":\"{}\",\"bucket\":5,\"epoch\":1,\"price\":200}}",
        hex(1),
        hex(1),
        hex(7)
    );
    let (status, raw) = call(&state, "POST", "/v1/ads/bid", &bid);
    assert_eq!(status, 200, "{raw}");
    let book = format!("{{\"topic\":\"{}\",\"bucket\":5,\"epoch\":1}}", hex(7));
    let (status, raw) = call(&state, "POST", "/v1/ads/clear", &book);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) = call(&state, "POST", "/v1/ads/select", &book);
    assert_eq!(status, 200, "{raw}");
    let selected: SelectBare = blazingly_json::from_str(&raw).expect("select");
    assert_eq!(selected.campaign, Some(hex(1)));
    let weak = format!("{{\"id\":\"{}\",\"strength\":50}}", hex(2));
    let (status, _) = call(&state, "POST", "/v1/ads/register", &weak);
    assert_eq!(status, 409);
}

#[test]
fn work_credits_move_and_history_stays() {
    let state = Mutex::new(State::default());
    let (status, raw) = call(&state, "POST", "/v1/work/open", "");
    assert_eq!(status, 200, "{raw}");
    let record = format!(
        "{{\"node\":\"{}\",\"kind\":\"repair\",\"units\":4000,\"epoch\":1,\"reliable\":true}}",
        hex(1)
    );
    let (status, raw) = call(&state, "POST", "/v1/work/record", &record);
    assert_eq!(status, 200, "{raw}");
    let view = format!("{{\"node\":\"{}\",\"social\":200}}", hex(1));
    let (status, raw) = call(&state, "POST", "/v1/work/view", &view);
    assert_eq!(status, 200, "{raw}");
    let before: WorkBare = blazingly_json::from_str(&raw).expect("work");
    assert!(before.eligible);
    let transfer = format!("{{\"from\":\"{}\",\"to\":\"{}\",\"amount\":5}}", hex(1), hex(2));
    let (status, raw) = call(&state, "POST", "/v1/work/transfer", &transfer);
    assert_eq!(status, 200, "{raw}");
    let view = format!("{{\"node\":\"{}\",\"social\":0}}", hex(2));
    let (status, raw) = call(&state, "POST", "/v1/work/view", &view);
    assert_eq!(status, 200, "{raw}");
    let receiver: WorkBare = blazingly_json::from_str(&raw).expect("receiver");
    assert_eq!(receiver.credits, 5);
    assert_eq!(receiver.repair, 0);
    let view = format!("{{\"node\":\"{}\",\"social\":0}}", hex(1));
    let (status, raw) = call(&state, "POST", "/v1/work/view", &view);
    assert_eq!(status, 200, "{raw}");
    let sender: WorkBare = blazingly_json::from_str(&raw).expect("sender");
    assert_eq!(sender.repair, before.repair);
}
