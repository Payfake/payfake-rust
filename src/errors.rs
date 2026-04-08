

use serde::Deserialize;
use thiserror::Error;

/// A single field-level error returned by the Payfake API.
/// Populated when a request fails validation, points to the
/// exact field that caused the problem and explains why.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorField {
    pub field: String,
    pub message: String,
}

/// PayfakeError is returned by every SDK method on failure.
///
/// We use thiserror's #[derive(Error)] to generate the std::error::Error
/// implementation automatically, no manual impl blocks needed.
///
/// The #[error("...")] attribute defines what the Display impl shows
/// when you print the error or use it in format strings.
///
/// Use pattern matching on the variants for programmatic error handling:
///
/// ```rust
/// match err {
///     PayfakeError::Api { code, .. } if code == "AUTH_EMAIL_TAKEN" => {
///         // handle duplicate email
///     }
///     PayfakeError::Api { code, .. } if code == "CHARGE_FAILED" => {
///         // handle charge failure
///     }
///     _ => return Err(err),
/// }
/// ```
#[derive(Debug, Error)]
pub enum PayfakeError {
    /// The API returned an error response.
    /// code is the Payfake response code, stable across API versions.
    /// message is human-readable, don't parse it programmatically.
    /// fields contains field-level validation errors if any.
    /// http_status is the HTTP status code of the response.
    #[error("payfake [{code}] {message}")]
    Api {
        code: String,
        message: String,
        fields: Vec<ApiErrorField>,
        http_status: u16,
    },

    /// The HTTP request itself failed, network error, timeout, DNS failure etc.
    /// This wraps reqwest's error type. The original error is available via
    /// the source() method from std::error::Error.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The response body could not be parsed as JSON.
    /// This usually means the server returned something unexpected
    /// a proxy error page, a plain text error, or a server crash response.
    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),

    /// A required field was missing in the SDK call.
    /// Either access_code or reference must be provided for charge calls.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl PayfakeError {
    /// Check if this error matches a specific Payfake response code.
    ///
    /// ```rust
    /// if err.is_code("AUTH_EMAIL_TAKEN") {
    ///     // handle duplicate email
    /// }
    /// ```
    pub fn is_code(&self, code: &str) -> bool {
        match self {
            PayfakeError::Api { code: c, .. } => c == code,
            _ => false,
        }
    }

    /// Extract the response code if this is an API error.
    /// Returns None for network errors and parse errors.
    pub fn code(&self) -> Option<&str> {
        match self {
            PayfakeError::Api { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Extract field-level validation errors if any.
    /// Returns an empty slice for non-validation errors.
    pub fn fields(&self) -> &[ApiErrorField] {
        match self {
            PayfakeError::Api { fields, .. } => fields,
            _ => &[],
        }
    }
}

/// Common error code constants, use these instead of raw strings
/// so your code stays correct if codes ever get renamed.
pub mod codes {
    pub const EMAIL_TAKEN:            &str = "AUTH_EMAIL_TAKEN";
    pub const INVALID_CREDENTIALS:    &str = "AUTH_INVALID_CREDENTIALS";
    pub const UNAUTHORIZED:           &str = "AUTH_UNAUTHORIZED";
    pub const TOKEN_EXPIRED:          &str = "AUTH_TOKEN_EXPIRED";
    pub const TRANSACTION_NOT_FOUND:  &str = "TRANSACTION_NOT_FOUND";
    pub const REFERENCE_TAKEN:        &str = "TRANSACTION_REFERENCE_TAKEN";
    pub const INVALID_AMOUNT:         &str = "TRANSACTION_INVALID_AMOUNT";
    pub const CHARGE_FAILED:          &str = "CHARGE_FAILED";
    pub const CHARGE_PENDING:         &str = "CHARGE_PENDING";
    pub const CUSTOMER_NOT_FOUND:     &str = "CUSTOMER_NOT_FOUND";
    pub const CUSTOMER_EMAIL_TAKEN:   &str = "CUSTOMER_EMAIL_TAKEN";
    pub const VALIDATION_ERROR:       &str = "VALIDATION_ERROR";
    pub const INTERNAL_ERROR:         &str = "INTERNAL_ERROR";
}
