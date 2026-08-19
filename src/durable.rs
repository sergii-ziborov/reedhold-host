//! Reed-Solomon durable grid. Company is never a required holder.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use crate::sync::company_hex;
use reedhold_api::DurableSession;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize)]
struct OpenBody {
    holders: Vec<String>,
    company: Option<String>,
}

#[derive(Deserialize)]
struct PutBody {
    payload: String,
    tier: Option<String>,
}

#[derive(Deserialize)]
struct IdBody {
    id: String,
}

#[derive(Deserialize)]
struct HolderBody {
    holder: String,
}

#[derive(Serialize)]
struct PayloadOut {
    payload: String,
}

pub(crate) fn open(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<OpenBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        host.durable = Some(
            DurableSession::open(&parsed.holders, company_hex(parsed.company.as_deref()))
                .map_err(|error| error.to_string())?,
        );
        Ok(())
    })
}

pub(crate) fn put(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<PutBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let tier = parsed.tier.as_deref().unwrap_or("critical");
    state::mutate(state, |host| {
        grid_mut(host)?.put(&parsed.payload, tier).map_err(|error| error.to_string())
    })
}

pub(crate) fn get(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<IdBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        let payload = grid(host)?.get(&parsed.id).map_err(|error| error.to_string())?;
        Ok(PayloadOut { payload })
    })
}

pub(crate) fn kill(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<HolderBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        grid_mut(host)?.kill(&parsed.holder).map_err(|error| error.to_string())
    })
}

pub(crate) fn repair(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<IdBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        grid_mut(host)?.repair(&parsed.id).map_err(|error| error.to_string())
    })
}

fn grid(host: &State) -> Result<&DurableSession, String> {
    host.durable.as_ref().ok_or_else(|| "durable grid is not open".to_owned())
}

fn grid_mut(host: &mut State) -> Result<&mut DurableSession, String> {
    host.durable.as_mut().ok_or_else(|| "durable grid is not open".to_owned())
}
