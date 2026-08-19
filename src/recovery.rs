//! k-of-n Shamir split / combine.

use crate::body::{CombineBody, CreateOut, ShareIn, SplitBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::State;
use reedhold_api::{ShareView, session_from_shares};
use std::sync::Mutex;

pub(crate) fn split(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<SplitBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    match mutate(state, |session| session.split_recovery(parsed.threshold, parsed.total)) {
        Ok(shares) => json(&shares),
        Err(error) => fail(&error),
    }
}

pub(crate) fn combine(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<CombineBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let shares: Vec<ShareView> = parsed.shares.into_iter().map(into_share).collect();
    match session_from_shares(&shares, parsed.threshold, &parsed.password, &parsed.device_secret) {
        Ok((session, manifest)) => {
            let account = session.view();
            if let Ok(mut lock) = state.lock() {
                lock.session = Some(session);
            }
            json(&CreateOut { account, manifest })
        }
        Err(error) => fail(&error.to_string()),
    }
}

fn into_share(share: ShareIn) -> ShareView {
    ShareView { index: share.index, body_hex: share.body_hex }
}

fn mutate<T, E: ToString>(
    state: &Mutex<State>,
    op: impl FnOnce(&mut reedhold_api::Session) -> Result<T, E>,
) -> Result<T, String> {
    let mut guard = state.lock().map_err(|_| "lock".to_owned())?;
    guard.with_mut(op)
}
