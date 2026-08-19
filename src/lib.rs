//! Sync JSON HTTP host. Protocol kernel stays in `reedhold-api`.

#![forbid(unsafe_code)]

mod account;
mod body;
mod recovery;
mod reply;
mod routes;
mod sealed;
mod state;
mod sync;

pub mod http;

pub use http::serve;
pub use reply::Reply;
pub use routes::dispatch;
pub use state::State;

#[cfg(test)]
mod api_tests;
