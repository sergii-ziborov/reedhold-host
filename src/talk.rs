//! DMs and small groups over the in-process fabric.

use crate::body::{PeerBody, PlanBody, parse_json};
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use crate::sync::{company_hex, prior_commit};
use reedhold_api::TalkNet;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize)]
struct DmBody {
    to: String,
    to_msg_pub: String,
    plaintext: String,
}

#[derive(Deserialize)]
struct NameBody {
    name: String,
}

#[derive(Deserialize)]
struct InviteBody {
    group: String,
    member: String,
    member_msg_pub: String,
}

#[derive(Deserialize)]
struct GroupTextBody {
    group: String,
    plaintext: String,
}

#[derive(Deserialize)]
struct RemoveBody {
    group: String,
    member: String,
}

#[derive(Serialize)]
struct DmOut {
    path: String,
    hop: Option<String>,
    conversation: String,
    text: String,
    from: String,
}

pub(crate) fn open(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<PlanBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        let prior = prior_commit(parsed.prior_commit.as_deref());
        let company = company_hex(parsed.company.as_deref());
        let mut talk =
            TalkNet::open(parsed.epoch, &prior, &parsed.candidates, company, parsed.relay_count)
                .map_err(|error| error.to_string())?;
        if let Some(session) = host.session.as_ref() {
            talk.online(&session.peer_hex()).map_err(|error| error.to_string())?;
        }
        host.talk = Some(talk);
        Ok(())
    })
}

pub(crate) fn online(state: &Mutex<State>, body: &str) -> Reply {
    peer_op(state, body, |talk, peer| talk.online(peer).map_err(|error| error.to_string()))
}

pub(crate) fn offline(state: &Mutex<State>, body: &str) -> Reply {
    peer_op(state, body, |talk, peer| talk.offline(peer).map_err(|error| error.to_string()))
}

pub(crate) fn block(state: &Mutex<State>, body: &str) -> Reply {
    peer_op(state, body, |talk, peer| talk.block(peer).map_err(|error| error.to_string()))
}

pub(crate) fn dm(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<DmBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        let from = session.peer_hex();
        let route = talk
            .dm(session, &parsed.to, &parsed.to_msg_pub, &parsed.plaintext)
            .map_err(|error| error.to_string())?;
        let conversation = reedhold_api::dm_conversation_hex(&from, &parsed.to)
            .map_err(|error| error.to_string())?;
        Ok(DmOut {
            path: route.path,
            hop: route.hop,
            conversation,
            text: parsed.plaintext.clone(),
            from,
        })
    })
}

pub(crate) fn create_group(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<NameBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        talk.create_circle(session, &parsed.name).map_err(|error| error.to_string())
    })
}

pub(crate) fn invite(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<InviteBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        talk.invite(session, &parsed.group, &parsed.member, &parsed.member_msg_pub)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn send(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<GroupTextBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        talk.send_circle(session, &parsed.group, &parsed.plaintext)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn remove(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<RemoveBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        talk.remove(session, &parsed.group, &parsed.member).map_err(|error| error.to_string())
    })
}

pub(crate) fn inbox(state: &Mutex<State>) -> Reply {
    state::mutate(state, |host| {
        crate::social::ensure_talk(host)?;
        let (talk, session) = host.talk_and_session().map_err(str::to_owned)?;
        talk.inbox(session).map_err(|error| error.to_string())
    })
}

fn peer_op(
    state: &Mutex<State>,
    body: &str,
    op: impl FnOnce(&mut TalkNet, &str) -> Result<(), String>,
) -> Reply {
    let parsed = match parse_json::<PeerBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        let talk = host.talk.as_mut().ok_or_else(|| "talk net is not open".to_owned())?;
        op(talk, &parsed.peer).map_err(|error| error.to_string())
    })
}
