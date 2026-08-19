//! HTTP reply helpers.

use crate::body::{error_json, write_json};

/// HTTP reply.
pub struct Reply {
    /// Status code.
    pub status: u16,
    /// JSON body.
    pub body: String,
}

/// Encode `value` as a 200 JSON reply.
#[must_use]
pub fn json<T: serde::Serialize>(value: &T) -> Reply {
    match write_json(value) {
        Ok(body) => Reply { status: 200, body },
        Err(error) => fail(&error),
    }
}

/// Literal JSON 200.
#[must_use]
pub fn ok(body: &str) -> Reply {
    Reply { status: 200, body: body.to_owned() }
}

/// Client error.
#[must_use]
pub fn bad(message: &str) -> Reply {
    Reply { status: 400, body: error_json(message) }
}

/// Protocol / session error.
#[must_use]
pub fn fail(message: &str) -> Reply {
    Reply { status: 409, body: error_json(message) }
}
