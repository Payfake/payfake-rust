use reqwest::Method;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{AuthResponse, KeysResponse, LoginInput, MerchantProfile, RegisterInput};

/// Wraps /api/v1/auth endpoints.
/// Payfake-specific, no Paystack equivalent.
/// Use the returned access_token with control and merchant methods.
pub struct AuthNamespace(pub(crate) Arc<Inner>);

impl AuthNamespace {
    /// Register a new merchant account.
    pub async fn register(&self, input: RegisterInput) -> Result<AuthResponse, PayfakeError> {
        self.0
            .do_jwt::<_, AuthResponse>(Method::POST, "/api/v1/auth/register", Some(&input), "")
            .await
    }

    /// Login and get an access token.
    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, PayfakeError> {
        self.0
            .do_jwt::<_, AuthResponse>(Method::POST, "/api/v1/auth/login", Some(&input), "")
            .await
    }

    /// Get the currently authenticated merchant profile.
    pub async fn me(&self, token: &str) -> Result<MerchantProfile, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/auth/me", None, token)
            .await
    }

    /// Get the merchant's public and secret keys.
    pub async fn get_keys(&self, token: &str) -> Result<KeysResponse, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/auth/keys", None, token)
            .await
    }

    /// Rotate the merchant's key pair.
    /// All requests using the old secret key fail immediately after this call.
    pub async fn regenerate_keys(&self, token: &str) -> Result<KeysResponse, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(
                Method::POST,
                "/api/v1/auth/keys/regenerate",
                None,
                token,
            )
            .await
    }
}
