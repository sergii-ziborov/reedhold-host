//! Reputation v0. Not a token.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use reedhold_api::RepSession;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct SeedBody {
    identity: String,
    continuity: u32,
    social: u32,
    content: u32,
    curation: u32,
}

#[derive(Deserialize)]
struct ReactBody {
    author: String,
    target: String,
    kind: String,
    cluster: Option<String>,
    now: u64,
}

#[derive(Deserialize)]
struct IdentityBody {
    identity: String,
    now: u64,
}

#[derive(Deserialize)]
struct ContentBody {
    target: String,
    now: u64,
}

#[derive(Deserialize)]
struct TransferBody {
    from: String,
    to: String,
    amount: u32,
}

pub(crate) fn open(state: &Mutex<State>) -> Reply {
    state::mutate_ok(state, |host| {
        host.rep = Some(RepSession::open());
        Ok(())
    })
}

pub(crate) fn seed(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SeedBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        book_mut(host)?
            .seed(
                &parsed.identity,
                parsed.continuity,
                parsed.social,
                parsed.content,
                parsed.curation,
            )
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn react(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<ReactBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let cluster = parsed.cluster.as_deref().unwrap_or("");
    state::mutate(state, |host| {
        book_mut(host)?
            .react(&parsed.author, &parsed.target, &parsed.kind, cluster, parsed.now)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn identity(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<IdentityBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        book(host)?.identity(&parsed.identity, parsed.now).map_err(|error| error.to_string())
    })
}

pub(crate) fn content(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<ContentBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        book(host)?.content(&parsed.target, parsed.now).map_err(|error| error.to_string())
    })
}

pub(crate) fn transfer(body: &str) -> Reply {
    let parsed = match parse_json::<TransferBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match RepSession::transfer(&parsed.from, &parsed.to, parsed.amount) {
        Ok(()) => crate::reply::ok_flag(),
        Err(error) => crate::reply::fail(&error.to_string()),
    }
}

fn book(host: &State) -> Result<&RepSession, String> {
    host.rep.as_ref().ok_or_else(|| "reputation book is not open".to_owned())
}

fn book_mut(host: &mut State) -> Result<&mut RepSession, String> {
    host.rep.as_mut().ok_or_else(|| "reputation book is not open".to_owned())
}
