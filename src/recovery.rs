//! k-of-n Shamir split / combine.

use crate::body::{CombineBody, CreateOut, ShareIn, SplitBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use crate::state::{self, State};
use reedhold_api::{ShareView, session_from_shares};
use std::sync::Mutex;

pub(crate) fn split(state: &Mutex<State>, seat: &str, body: &str) -> Reply {
    let parsed = match parse_json::<SplitBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        host.with_mut(seat, |session| session.split_recovery(parsed.threshold, parsed.total))
    })
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
            let seat = match state.lock() {
                Ok(mut lock) => lock.issue_seat(session),
                Err(_) => return fail("lock"),
            };
            json(&CreateOut { account, manifest, seat })
        }
        Err(error) => fail(&error.to_string()),
    }
}

fn into_share(share: ShareIn) -> ShareView {
    ShareView { index: share.index, body_hex: share.body_hex }
}
