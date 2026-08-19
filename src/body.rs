//! JSON read/write through `blazingly-json`.

use serde::{Deserialize, Serialize};
use tiny_http::Request;

/// JSON object returned on errors.
#[derive(Serialize)]
pub struct ErrorBody {
    /// Human-readable failure.
    pub error: String,
}

/// Account create / restore fields.
#[derive(Deserialize)]
pub struct SecretBody {
    pub password: String,
    pub device_secret: String,
    pub manifest_hex: Option<String>,
}

/// Emit a signed event.
#[derive(Deserialize)]
pub struct EmitBody {
    pub kind: String,
    pub payload: String,
}

/// Verify a signed event hex.
#[derive(Deserialize)]
pub struct VerifyBody {
    pub event_hex: String,
}

/// Change the vault password.
#[derive(Deserialize)]
pub struct PasswordBody {
    pub password: String,
}

/// Read the request body as `T`.
///
/// # Errors
///
/// Returns a display string when the body is not valid JSON for `T`.
pub fn read_json<T: for<'de> Deserialize<'de>>(request: &mut Request) -> Result<T, String> {
    let mut raw = String::new();
    request.as_reader().read_to_string(&mut raw).map_err(|error| error.to_string())?;
    blazingly_json::from_str(&raw).map_err(|error| error.to_string())
}

/// Encode `value` as JSON.
///
/// # Errors
///
/// Returns a display string when encoding fails.
pub fn write_json<T: Serialize>(value: &T) -> Result<String, String> {
    blazingly_json::to_string(value).map_err(|error| error.to_string())
}

/// Standard error object.
#[must_use]
pub fn error_json(message: &str) -> String {
    write_json(&ErrorBody { error: message.to_owned() })
        .unwrap_or_else(|_| "{\"error\":\"encode failed\"}".to_owned())
}
