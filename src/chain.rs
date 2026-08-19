//! Compact checkpoint headers. No message bytes.

use crate::body::{FlagOut, parse_json};
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use reedhold_api::ChainSession;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
struct CommitBody {
    epoch: u64,
    identity: String,
    groups: String,
    storage: String,
}

#[derive(Deserialize)]
struct ProveBody {
    leaves: Vec<String>,
    index: u32,
}

#[derive(Deserialize)]
struct VerifyBody {
    leaf: String,
    root: String,
    index: u32,
    siblings: Vec<String>,
}

pub(crate) fn open(state: &Mutex<State>) -> Reply {
    state::mutate_ok(state, |host| {
        host.chain = Some(ChainSession::open().map_err(|error| error.to_string())?);
        Ok(())
    })
}

pub(crate) fn commit(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<CommitBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        chain_mut(host)?
            .commit(parsed.epoch, &parsed.identity, &parsed.groups, &parsed.storage)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn head(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| Ok(chain(host)?.head()))
}

pub(crate) fn headers(state: &Mutex<State>) -> Reply {
    state::inspect(state, |host| Ok(chain(host)?.headers()))
}

pub(crate) fn prove(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<ProveBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        chain(host)?.prove(&parsed.leaves, parsed.index).map_err(|error| error.to_string())
    })
}

pub(crate) fn verify(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<VerifyBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        let ok = chain(host)?
            .verify(&parsed.leaf, &parsed.root, parsed.index, &parsed.siblings)
            .map_err(|error| error.to_string())?;
        Ok(FlagOut { ok })
    })
}

fn chain(host: &State) -> Result<&ChainSession, String> {
    host.chain.as_ref().ok_or_else(|| "chain is not open".to_owned())
}

fn chain_mut(host: &mut State) -> Result<&mut ChainSession, String> {
    host.chain.as_mut().ok_or_else(|| "chain is not open".to_owned())
}
