//! Sync HTTP host. Protocol kernel stays in `reedhold-api`.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let bind = std::env::var("REEDHOLD_HOST").unwrap_or_else(|_| "127.0.0.1:4783".to_owned());
    match reedhold_host::serve(&bind) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("reedhold-host: {error}");
            ExitCode::FAILURE
        }
    }
}
