//! Alias, contacts, groups, and public rooms.

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
struct AliasBare {
    nick: String,
    identity: String,
}

#[derive(Deserialize)]
struct CircleBare {
    name: String,
    you_admin: bool,
}

#[derive(Deserialize)]
struct EventBare {
    event_hex: String,
}

#[derive(Deserialize)]
struct RoomBare {
    topic: String,
    posts: Vec<PostBare>,
}

#[derive(Deserialize)]
struct PostBare {
    from: String,
    text: String,
}

fn hex(byte: u8) -> String {
    format!("{byte:02x}").repeat(32)
}

fn call(state: &Mutex<State>, method: &str, url: &str, body: &str, seat: &str) -> (u16, String) {
    let reply = dispatch(state, method, url, body, seat);
    (reply.status, reply.body)
}

fn create(state: &Mutex<State>) -> Created {
    let body = format!("{{\"password\":\"pw\",\"device_secret\":\"{}\"}}", hex(0xab));
    let (status, raw) = call(state, "POST", "/v1/account", &body, "");
    assert_eq!(status, 200, "{raw}");
    blazingly_json::from_str(&raw).expect("created")
}

fn identity_hex(uri: &str) -> String {
    uri.rsplit(':').next().unwrap_or(uri).to_owned()
}

#[test]
fn alias_lookup_does_not_embed_nick_in_events() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    let seat = &created.seat;
    let (status, raw) = call(&state, "POST", "/v1/alias", "{\"nick\":\"@Alice_01\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let alias: AliasBare = blazingly_json::from_str(&raw).expect("alias");
    assert_eq!(alias.nick, "alice_01");
    assert_eq!(alias.identity, identity_hex(&created.account.identity));
    let (status, raw) = call(&state, "POST", "/v1/alias/lookup", "{\"nick\":\"alice_01\"}", "");
    assert_eq!(status, 200, "{raw}");
    let (status, raw) =
        call(&state, "POST", "/v1/account/emit", "{\"kind\":\"post\",\"payload\":\"hello\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let event: EventBare = blazingly_json::from_str(&raw).expect("event");
    assert!(!event.event_hex.to_ascii_lowercase().contains("alice_01"));
}

#[test]
fn owner_is_group_admin_and_contacts_list() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    let seat = &created.seat;
    let add = format!(
        "{{\"identity\":\"{}\",\"messaging_public\":\"{}\",\"petname\":\"Bob\"}}",
        hex(2),
        hex(3)
    );
    let (status, raw) = call(&state, "POST", "/v1/contacts", &add, seat);
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains("Bob"));
    let (status, raw) = call(&state, "POST", "/v1/talk/group", "{\"name\":\"ops\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let group: CircleBare = blazingly_json::from_str(&raw).expect("group");
    assert_eq!(group.name, "ops");
    assert!(group.you_admin);
    let (status, raw) = call(&state, "GET", "/v1/chats", "", seat);
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains("ops"));
    assert!(raw.contains("Bob"));
}

#[test]
fn public_room_post_omits_topic_from_event() {
    let state = Mutex::new(State::default());
    let created = create(&state);
    let seat = &created.seat;
    let (status, raw) = call(&state, "POST", "/v1/rooms/join", "{\"topic\":\"Privacy\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) =
        call(&state, "POST", "/v1/rooms/post", "{\"topic\":\"privacy\",\"text\":\"hello room\"}", seat);
    assert_eq!(status, 200, "{raw}");
    let (status, raw) = call(&state, "GET", "/v1/account/history", "", seat);
    assert_eq!(status, 200, "{raw}");
    assert!(!raw.to_ascii_lowercase().contains("privacy"));
    let (status, raw) = call(&state, "GET", "/v1/rooms", "", seat);
    assert_eq!(status, 200, "{raw}");
    let rooms: Vec<RoomBare> = blazingly_json::from_str(&raw).expect("rooms");
    assert_eq!(rooms[0].topic, "privacy");
    assert_eq!(rooms[0].posts[0].text, "hello room");
    assert_eq!(rooms[0].posts[0].from, identity_hex(&created.account.identity));
    let _ = created.account.messaging_public;
}

#[test]
fn a_stranger_does_not_inherit_the_last_account() {
    let state = Mutex::new(State::default());
    let first = create(&state);
    let (status, raw) = call(&state, "GET", "/v1/account", "", "");
    assert_eq!(status, 409, "{raw}");
    let (status, raw) = call(&state, "GET", "/v1/account", "", &first.seat);
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains(&first.account.identity));
    let second = create(&state);
    let (status, raw) = call(&state, "GET", "/v1/account", "", &second.seat);
    assert_eq!(status, 200, "{raw}");
    assert!(raw.contains(&second.account.identity));
    assert!(!raw.contains(&first.account.identity));
}
