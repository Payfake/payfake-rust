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

    /// Get full merchant profile.
    pub async fn get_profile(&self, token: &str) -> Result<MerchantProfile, PayfakeError> {
        self.inner.request::<serde_json::Value, MerchantProfile>(
            Method::GET, "/api/v1/merchant", None, Some(token),
        ).await
    }

    /// Update merchant business name and/or webhook URL.
    pub async fn update_profile(
        &self,
        token: &str,
        input: UpdateProfileInput,
    ) -> Result<MerchantProfile, PayfakeError> {
        self.inner.request::<_, MerchantProfile>(
            Method::PUT, "/api/v1/merchant", Some(&input), Some(token),
        ).await
    }

    /// Get current webhook URL and config.
    pub async fn get_webhook_url(&self, token: &str) -> Result<WebhookConfig, PayfakeError> {
        self.inner.request::<serde_json::Value, WebhookConfig>(
            Method::GET, "/api/v1/merchant/webhook", None, Some(token),
        ).await
    }

    /// Set webhook URL.
    pub async fn update_webhook_url(&self, token: &str, webhook_url: &str) -> Result<(), PayfakeError> {
        #[derive(serde::Serialize)]
        struct Input<'a> { webhook_url: &'a str }
        self.inner.request::<_, serde_json::Value>(
            Method::POST, "/api/v1/merchant/webhook",
            Some(&Input { webhook_url }), Some(token),
        ).await?;
        Ok(())
    }

    /// Fire a test webhook. Rate limited to 5 per minute per merchant.
    pub async fn test_webhook(&self, token: &str) -> Result<WebhookTestResult, PayfakeError> {
        self.inner.request::<serde_json::Value, WebhookTestResult>(
            Method::POST, "/api/v1/merchant/webhook/test", None, Some(token),
        ).await
    }
}
