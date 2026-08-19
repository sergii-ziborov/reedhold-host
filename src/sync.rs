//! Epoch relay lottery. Company host is never required.

use crate::body::{PlanBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use reedhold_api::sync_plan;

pub(crate) fn prior_commit(value: Option<&str>) -> String {
    value.filter(|text| !text.is_empty()).map_or_else(|| "00".repeat(32), str::to_owned)
}

pub(crate) fn company_hex(value: Option<&str>) -> Option<&str> {
    value.filter(|text| !text.is_empty())
}

pub(crate) fn plan(body: &str) -> Reply {
    let parsed = match parse_json::<PlanBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let prior = prior_commit(parsed.prior_commit.as_deref());
    let company = company_hex(parsed.company.as_deref());
    match sync_plan(parsed.epoch, &prior, &parsed.candidates, company, parsed.relay_count) {
        Ok(view) => json(&view),
        Err(error) => fail(&error.to_string()),
    }
}
