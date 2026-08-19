//! In-memory sessions for this process. Same shape as the MCP host.

use crate::reply::{Reply, fail, json, ok_flag};
use reedhold_api::{
    AccountView, ChainSession, DurableSession, MarketSession, RepSession, Session, TalkNet,
    WorkSession,
};
use std::sync::{Mutex, MutexGuard};

/// Unlocked identity plus optional sandbox overlays.
#[derive(Default)]
pub struct State {
    pub(crate) session: Option<Session>,
    pub(crate) talk: Option<TalkNet>,
    pub(crate) durable: Option<DurableSession>,
    pub(crate) chain: Option<ChainSession>,
    pub(crate) rep: Option<RepSession>,
    pub(crate) ads: Option<MarketSession>,
    pub(crate) work: Option<WorkSession>,
}

impl State {
    pub(crate) fn view(&self) -> Result<AccountView, &'static str> {
        self.session.as_ref().map(Session::view).ok_or("no unlocked session")
    }

    pub(crate) fn with<T, E: ToString>(
        &self,
        op: impl FnOnce(&Session) -> Result<T, E>,
    ) -> Result<T, String> {
        let session = self.session.as_ref().ok_or_else(|| "no unlocked session".to_owned())?;
        op(session).map_err(|error| error.to_string())
    }

    pub(crate) fn with_mut<T, E: ToString>(
        &mut self,
        op: impl FnOnce(&mut Session) -> Result<T, E>,
    ) -> Result<T, String> {
        let session = self.session.as_mut().ok_or_else(|| "no unlocked session".to_owned())?;
        op(session).map_err(|error| error.to_string())
    }

    pub(crate) fn talk_and_session(
        &mut self,
    ) -> Result<(&mut TalkNet, &mut Session), &'static str> {
        let talk = self.talk.as_mut().ok_or("talk net is not open")?;
        let session = self.session.as_mut().ok_or("no unlocked session")?;
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
