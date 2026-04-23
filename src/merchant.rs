use reqwest::Method;
use serde_json::json;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{MerchantProfile, UpdateProfileInput, WebhookConfig};

/// Wraps /api/v1/merchant endpoints.
/// Payfake-specific, no Paystack equivalent.
/// Auth: Bearer JWT (from auth.login)
pub struct MerchantNamespace(pub(crate) Arc<Inner>);

impl MerchantNamespace {
    pub async fn get_profile(&self, token: &str) -> Result<MerchantProfile, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/merchant", None, token)
            .await
    }

    pub async fn update_profile(
        &self,
        token: &str,
        input: UpdateProfileInput,
    ) -> Result<MerchantProfile, PayfakeError> {
        self.0
            .do_jwt(Method::PUT, "/api/v1/merchant", Some(&input), token)
            .await
    }

    pub async fn get_webhook_url(&self, token: &str) -> Result<WebhookConfig, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/merchant/webhook", None, token)
            .await
    }

    pub async fn update_webhook_url(
        &self,
        token: &str,
        webhook_url: &str,
    ) -> Result<serde_json::Value, PayfakeError> {
        self.0
            .do_jwt(
                Method::POST,
                "/api/v1/merchant/webhook",
                Some(&json!({ "webhook_url": webhook_url })),
                token,
            )
            .await
    }

    pub async fn test_webhook(&self, token: &str) -> Result<serde_json::Value, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(
                Method::POST,
                "/api/v1/merchant/webhook/test",
                None,
                token,
            )
            .await
    }
}
