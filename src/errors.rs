use thiserror::Error;

/// A single field-level validation error from the API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ErrorField {
    pub field:   String,
    pub rule:    String,
    pub message: String,
}

/// Every SDK method returns this on failure.
/// Use `is_code()` for programmatic error handling — never match on message.
///
/// # Example
/// ```rust
/// match client.charge.submit_otp(input).await {
///     Err(e) if e.is_code(codes::CHARGE_INVALID_OTP) => {
///         // resend OTP
///     }
///     Err(e) => eprintln!("{}", e),
///     Ok(resp) => { /* success */ }
/// }
/// ```
#[derive(Debug, Error)]
pub enum PayfakeError {
    #[error("PayfakeError [{code}] {message}")]
    Api {
        /// X-Payfake-Code header value — use this for programmatic handling.
        code: String,
        /// Human-readable error message — log this, don't match on it.
        message: String,
        /// Field-level validation errors (populated when code is VALIDATION_ERROR).
        fields: Vec<ErrorField>,
        /// HTTP status code of the failed response.
        http_status: u16,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl PayfakeError {
    /// Returns true if this is an Api error with the given code.
    pub fn is_code(&self, code: &str) -> bool {
        matches!(self, PayfakeError::Api { code: c, .. } if c == code)
    }

    /// Returns the error code if this is an Api error.
    pub fn code(&self) -> Option<&str> {
        match self {
            PayfakeError::Api { code, .. } => Some(code.as_str()),
            _ => None,
        }
    }
}

/// Response code constants — use these instead of raw strings.
pub mod codes {
    // Auth
    pub const EMAIL_TAKEN:          &str = "AUTH_EMAIL_TAKEN";
    pub const INVALID_CREDENTIALS:  &str = "AUTH_INVALID_CREDENTIALS";
    pub const UNAUTHORIZED:         &str = "AUTH_UNAUTHORIZED";
    pub const TOKEN_EXPIRED:        &str = "AUTH_TOKEN_EXPIRED";
    pub const TOKEN_INVALID:        &str = "AUTH_TOKEN_INVALID";

    // Transaction
    pub const TRANSACTION_NOT_FOUND:  &str = "TRANSACTION_NOT_FOUND";
    pub const REFERENCE_TAKEN:        &str = "TRANSACTION_REFERENCE_TAKEN";
    pub const INVALID_AMOUNT:         &str = "TRANSACTION_INVALID_AMOUNT";
    pub const ALREADY_VERIFIED:       &str = "TRANSACTION_ALREADY_VERIFIED";

    // Charge
    pub const CHARGE_FAILED:              &str = "CHARGE_FAILED";
    pub const CHARGE_SUCCESSFUL:          &str = "CHARGE_SUCCESSFUL";
    pub const CHARGE_SEND_PIN:            &str = "CHARGE_SEND_PIN";
    pub const CHARGE_SEND_OTP:            &str = "CHARGE_SEND_OTP";
    pub const CHARGE_SEND_BIRTHDAY:       &str = "CHARGE_SEND_BIRTHDAY";
    pub const CHARGE_SEND_ADDRESS:        &str = "CHARGE_SEND_ADDRESS";
    pub const CHARGE_OPEN_URL:            &str = "CHARGE_OPEN_URL";
    pub const CHARGE_PAY_OFFLINE:         &str = "CHARGE_PAY_OFFLINE";
    pub const CHARGE_INVALID_OTP:         &str = "CHARGE_INVALID_OTP";
    pub const INSUFFICIENT_FUNDS:         &str = "CHARGE_INSUFFICIENT_FUNDS";
    pub const DO_NOT_HONOR:               &str = "CHARGE_DO_NOT_HONOR";
    pub const MOMO_TIMEOUT:               &str = "CHARGE_MOMO_TIMEOUT";
    pub const MOMO_PROVIDER_UNAVAILABLE:  &str = "CHARGE_MOMO_PROVIDER_UNAVAILABLE";

    // Customer
    pub const CUSTOMER_NOT_FOUND:   &str = "CUSTOMER_NOT_FOUND";
    pub const CUSTOMER_EMAIL_TAKEN: &str = "CUSTOMER_EMAIL_TAKEN";

    // Generic
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const INTERNAL_ERROR:   &str = "INTERNAL_ERROR";
    pub const NOT_FOUND:        &str = "NOT_FOUND";
    pub const RATE_LIMITED:     &str = "RATE_LIMIT_EXCEEDED";
}
