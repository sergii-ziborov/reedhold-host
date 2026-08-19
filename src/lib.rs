//! Sync JSON HTTP host. Protocol kernel stays in `reedhold-api`.

#![forbid(unsafe_code)]

mod account;
mod ads;
mod body;
mod chain;
mod durable;
mod recovery;
mod rep;
mod reply;
mod rooms;
mod routes;
mod sealed;
mod social;
mod state;
mod sync;
mod talk;
mod work;

pub mod http;

pub use http::serve;
pub use reply::Reply;
pub use routes::dispatch;
pub use state::State;

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod sandbox_tests;
#[cfg(test)]
mod social_tests;
