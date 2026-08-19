//! Proof of contribution. Credits move; history does not.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use reedhold_api::WorkSession;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize)]
struct RecordBody {
    node: String,
    kind: String,
    units: u32,
    epoch: u64,
    reliable: bool,
}

#[derive(Deserialize)]
struct ViewBody {
    node: String,
    social: u32,
}

#[derive(Deserialize)]
struct TransferBody {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Serialize)]
struct UnitsOut {
    units: u32,
}

pub(crate) fn open(state: &Mutex<State>) -> Reply {
    state::mutate_ok(state, |host| {
        host.work = Some(WorkSession::open());
        Ok(())
    })
}

pub(crate) fn record(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<RecordBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let units = book_mut(host)?
            .record(&parsed.node, &parsed.kind, parsed.units, parsed.epoch, parsed.reliable)
            .map_err(|error| error.to_string())?;
        Ok(UnitsOut { units })
    })
}

pub(crate) fn view(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<ViewBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        book(host)?.view(&parsed.node, parsed.social).map_err(|error| error.to_string())
    })
}

pub(crate) fn transfer(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<TransferBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        book_mut(host)?
            .transfer(&parsed.from, &parsed.to, parsed.amount)
            .map_err(|error| error.to_string())
    })
}

fn book(host: &State) -> Result<&WorkSession, String> {
    host.work.as_ref().ok_or_else(|| "work book is not open".to_owned())
}

fn book_mut(host: &mut State) -> Result<&mut WorkSession, String> {
    host.work.as_mut().ok_or_else(|| "work book is not open".to_owned())
}
