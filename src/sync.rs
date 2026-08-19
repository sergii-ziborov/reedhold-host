//! Epoch relay lottery. Company host is never required.

use crate::body::{PlanBody, parse_json};
use crate::reply::{Reply, bad, fail, json};
use reedhold_api::sync_plan;

pub(crate) fn plan(body: &str) -> Reply {
    let parsed = match parse_json::<PlanBody>(body) {
        Ok(value) => value,
        Err(error) => return bad(&error),
    };
    let prior =
        parsed.prior_commit.filter(|value| !value.is_empty()).unwrap_or_else(|| "00".repeat(32));
    let company = parsed.company.as_deref().filter(|value| !value.is_empty());
    match sync_plan(parsed.epoch, &prior, &parsed.candidates, company, parsed.relay_count) {
        Ok(view) => json(&view),
        Err(error) => fail(&error.to_string()),
    }
}
