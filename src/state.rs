//! Many unlocked seats in one process. The website is not a single shared user.

use crate::reply::{Reply, fail, json, ok_flag};
use reedhold_api::{
    AccountView, AliasDirectory, ChainSession, DurableSession, MarketSession, RepSession,
    RoomBoard, Session, TalkNet, WorkSession,
};
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

/// Per-browser seats plus shared public overlays (alias directory, rooms).
#[derive(Default)]
pub struct State {
    pub(crate) seats: BTreeMap<String, Session>,
    pub(crate) next_seat: u64,
    pub(crate) talk: Option<TalkNet>,
    pub(crate) durable: Option<DurableSession>,
    pub(crate) chain: Option<ChainSession>,
    pub(crate) rep: Option<RepSession>,
    pub(crate) ads: Option<MarketSession>,
    pub(crate) work: Option<WorkSession>,
    pub(crate) aliases: AliasDirectory,
    pub(crate) rooms: RoomBoard,
}

impl State {
    pub(crate) fn issue_seat(&mut self, session: Session) -> String {
        self.next_seat = self.next_seat.saturating_add(1);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let id = format!("{:x}{:x}", self.next_seat, now);
        self.talk = None;
        self.seats.insert(id.clone(), session);
        id
    }

    pub(crate) fn seat(&self, id: &str) -> Result<&Session, String> {
        if id.is_empty() {
            return Err("no unlocked session".to_owned());
        }
        self.seats.get(id).ok_or_else(|| "no unlocked session".to_owned())
    }

    pub(crate) fn seat_mut(&mut self, id: &str) -> Result<&mut Session, String> {
        if id.is_empty() {
            return Err("no unlocked session".to_owned());
        }
        self.seats.get_mut(id).ok_or_else(|| "no unlocked session".to_owned())
    }

    pub(crate) fn view(&self, id: &str) -> Result<AccountView, String> {
        Ok(self.seat(id)?.view())
    }

    pub(crate) fn with<T, E: ToString>(
        &self,
        id: &str,
        op: impl FnOnce(&Session) -> Result<T, E>,
    ) -> Result<T, String> {
        op(self.seat(id)?).map_err(|error| error.to_string())
    }

    pub(crate) fn with_mut<T, E: ToString>(
        &mut self,
        id: &str,
        op: impl FnOnce(&mut Session) -> Result<T, E>,
    ) -> Result<T, String> {
        op(self.seat_mut(id)?).map_err(|error| error.to_string())
    }

    pub(crate) fn talk_and_seat(
        &mut self,
        id: &str,
    ) -> Result<(&mut TalkNet, &mut Session), String> {
        if !self.seats.contains_key(id) {
            return Err("no unlocked session".to_owned());
        }
        let talk = self.talk.as_mut().ok_or_else(|| "talk net is not open".to_owned())?;
        let session = self.seats.get_mut(id).ok_or_else(|| "no unlocked session".to_owned())?;
        Ok((talk, session))
    }
}

pub(crate) fn locked(state: &Mutex<State>) -> Result<MutexGuard<'_, State>, String> {
    state.lock().map_err(|_| "lock".to_owned())
}

pub(crate) fn mutate<T: serde::Serialize>(
    state: &Mutex<State>,
    op: impl FnOnce(&mut State) -> Result<T, String>,
) -> Reply {
    match locked(state).and_then(|mut guard| op(&mut guard)) {
        Ok(value) => json(&value),
        Err(error) => fail(&error),
    }
}

pub(crate) fn inspect<T: serde::Serialize>(
    state: &Mutex<State>,
    op: impl FnOnce(&State) -> Result<T, String>,
) -> Reply {
    match locked(state).and_then(|guard| op(&guard)) {
        Ok(value) => json(&value),
        Err(error) => fail(&error),
    }
}

pub(crate) fn mutate_ok(
    state: &Mutex<State>,
    op: impl FnOnce(&mut State) -> Result<(), String>,
) -> Reply {
    match locked(state).and_then(|mut guard| op(&mut guard)) {
        Ok(()) => ok_flag(),
        Err(error) => fail(&error),
    }
}
