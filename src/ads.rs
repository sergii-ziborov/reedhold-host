//! Attention-market sandbox. No user-id targeting.

use crate::body::parse_json;
use crate::reply::{Reply, bad};
use crate::state::{self, State};
use reedhold_api::MarketSession;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Deserialize)]
struct PostBody {
    advertiser: String,
    campaign: String,
    payload: String,
    topic: String,
    bucket_min: u8,
    bucket_max: u8,
    budget: u64,
    expiry: u64,
}

#[derive(Deserialize)]
struct RegisterBody {
    id: String,
    strength: u32,
}

#[derive(Deserialize)]
struct BidBody {
    advertiser: String,
    campaign: String,
    topic: String,
    bucket: u8,
    epoch: u64,
    price: u64,
}

#[derive(Deserialize)]
struct BookBody {
    topic: String,
    bucket: u8,
    epoch: u64,
}

#[derive(Deserialize)]
struct CampaignBody {
    campaign: String,
}

#[derive(Deserialize)]
struct StrengthBody {
    strength: u32,
}

#[derive(Serialize)]
struct SelectOut {
    campaign: Option<String>,
}

#[derive(Serialize)]
struct FloorOut {
    floor: u32,
}

#[derive(Serialize)]
struct BucketOut {
    bucket: u8,
}

pub(crate) fn open(state: &Mutex<State>) -> Reply {
    state::mutate_ok(state, |host| {
        host.ads = Some(MarketSession::open());
        Ok(())
    })
}

pub(crate) fn post(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<PostBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        market_mut(host)?
            .post(
                &parsed.advertiser,
                &parsed.campaign,
                &parsed.payload,
                &parsed.topic,
                parsed.bucket_min,
                parsed.bucket_max,
                parsed.budget,
                parsed.expiry,
            )
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn register(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<RegisterBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        market_mut(host)?.register(&parsed.id, parsed.strength).map_err(|error| error.to_string())
    })
}

pub(crate) fn bid(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<BidBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate_ok(state, |host| {
        market_mut(host)?
            .bid(
                &parsed.advertiser,
                &parsed.campaign,
                &parsed.topic,
                parsed.bucket,
                parsed.epoch,
                parsed.price,
            )
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn clear(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<BookBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        market_mut(host)?
            .clear(&parsed.topic, parsed.bucket, parsed.epoch)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn select(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<BookBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        let campaign = market(host)?
            .select(&parsed.topic, parsed.bucket, parsed.epoch)
            .map_err(|error| error.to_string())?;
        Ok(SelectOut { campaign })
    })
}

pub(crate) fn hide(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<CampaignBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::mutate(state, |host| {
        let floor = market_mut(host)?.hide(&parsed.campaign).map_err(|error| error.to_string())?;
        Ok(FloorOut { floor })
    })
}

pub(crate) fn settle(state: &Mutex<State>, body: &str) -> Reply {
    let parsed = match parse_json::<BookBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    state::inspect(state, |host| {
        market(host)?
            .settle(&parsed.topic, parsed.bucket, parsed.epoch)
            .map_err(|error| error.to_string())
    })
}

pub(crate) fn bucket(body: &str) -> Reply {
    let parsed = match parse_json::<StrengthBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    crate::reply::json(&BucketOut { bucket: MarketSession::bucket(parsed.strength) })
}

fn market(host: &State) -> Result<&MarketSession, String> {
    host.ads.as_ref().ok_or_else(|| "market is not open".to_owned())
}

fn market_mut(host: &mut State) -> Result<&mut MarketSession, String> {
    host.ads.as_mut().ok_or_else(|| "market is not open".to_owned())
}
