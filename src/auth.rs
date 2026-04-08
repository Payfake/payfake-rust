use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{AuthResponse, KeysResponse, LoginInput, RegisterInput};

/// Wraps the /auth endpoints.
///
/// Register and Login require no authentication.
/// GetKeys and RegenerateKeys require a JWT token from login().
pub struct AuthNamespace {
    inner: Arc<ClientInner>,
}

impl AuthNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Create a new merchant account.
    /// Returns merchant data and a JWT token on success.
    /// Returns PayfakeError::Api with code AUTH_EMAIL_TAKEN if
    /// the email is already registered.
    pub async fn register(&self, input: RegisterInput) -> Result<AuthResponse, PayfakeError> {
        self.inner
            .request::<_, AuthResponse>(
                Method::POST,
                "/api/v1/auth/register",
                Some(&input),
                None,
            )
            .await
    }

    /// Authenticate a merchant and return a JWT token.
    /// Store the token, you need it for all control and key endpoints.
    /// Tokens expire after JWT_EXPIRY_HOURS (default 24 hours).
    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, PayfakeError> {
        self.inner
            .request::<_, AuthResponse>(
                Method::POST,
                "/api/v1/auth/login",
                Some(&input),
                None,
            )
            .await
    }

    /// Fetch the merchant's current public and secret keys.
    /// Requires a valid JWT token from login().
    pub async fn get_keys(&self, token: &str) -> Result<KeysResponse, PayfakeError> {
        self.inner
            .request::<serde_json::Value, KeysResponse>(
                Method::GET,
                "/api/v1/auth/keys",
                None,
                Some(token),
            )
            .await
    }

    /// Generate a new key pair for the merchant.
    /// The old secret key is immediately invalid after this call.
    /// Update your environment variables before calling this.
    /// Requires a valid JWT token from login().
    pub async fn regenerate_keys(&self, token: &str) -> Result<KeysResponse, PayfakeError> {
        self.inner
            .request::<serde_json::Value, KeysResponse>(
                Method::POST,
                "/api/v1/auth/keys/regenerate",
                None,
                Some(token),
            )
            .await
    }
}
