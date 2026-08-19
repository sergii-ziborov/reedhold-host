//! JSON read/write through `blazingly-json`.

use serde::Deserialize;

/// JSON object returned on errors.
#[derive(serde::Serialize)]
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

/// Split the unlocked seed.
#[derive(Deserialize)]
pub struct SplitBody {
    pub threshold: u8,
    pub total: u8,
}

/// One Shamir share from a client.
#[derive(Deserialize)]
pub struct ShareIn {
    pub index: u8,
    pub body_hex: String,
}

/// Combine k-of-n shares into a new session.
#[derive(Deserialize)]
pub struct CombineBody {
    pub password: String,
    pub device_secret: String,
    pub threshold: u8,
    pub shares: Vec<ShareIn>,
}

/// Seal plaintext under a conversation key.
#[derive(Deserialize)]
pub struct SealedBody {
    pub conversation_key: String,
    pub plaintext: String,
}

/// Open a sealed envelope hex.
#[derive(Deserialize)]
pub struct OpenBody {
    pub conversation_key: String,
    pub envelope_hex: String,
}

/// Draw today's transitional relays.
#[derive(Deserialize)]
pub struct PlanBody {
    pub epoch: u64,
    pub prior_commit: Option<String>,
    pub candidates: Vec<String>,
    pub company: Option<String>,
    pub relay_count: Option<u16>,
}

/// UTF-8 plaintext reply.
#[derive(serde::Serialize)]
pub struct PlainOut {
    pub plaintext: String,
}

/// Create-account reply.
#[derive(serde::Serialize)]
pub struct CreateOut {
    pub account: reedhold_api::AccountView,
    pub manifest: reedhold_api::ManifestView,
}

/// Decode JSON `T`.
///
/// # Errors
///
/// Returns a display string when the body is not valid JSON for `T`.
pub fn parse_json<T: for<'de> Deserialize<'de>>(raw: &str) -> Result<T, String> {
    blazingly_json::from_str(raw).map_err(|error| error.to_string())
}

/// Encode `value` as JSON.
///
/// # Errors
///
/// Returns a display string when encoding fails.
pub fn write_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    blazingly_json::to_string(value).map_err(|error| error.to_string())
}

/// Standard error object.
#[must_use]
pub fn error_json(message: &str) -> String {
    write_json(&ErrorBody { error: message.to_owned() })
        .unwrap_or_else(|_| "{\"error\":\"encode failed\"}".to_owned())
}
