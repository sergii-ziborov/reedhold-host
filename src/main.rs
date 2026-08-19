//! Sync HTTP host. Protocol kernel stays in `reedhold-api`.

#![forbid(unsafe_code)]

mod body;
mod http;
mod routes;

use crate::http::serve;
use std::process::ExitCode;

fn main() -> ExitCode {
    let bind = std::env::var("REEDHOLD_HOST").unwrap_or_else(|_| "127.0.0.1:4783".to_owned());
    match serve(&bind) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("reedhold-host: {error}");
            ExitCode::FAILURE
        }
    }
}
