use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{
    ScenarioConfig, UpdateScenarioInput, ForceTransactionInput,
    WebhookEvent, WebhookAttempt, RequestLog,
    Transaction, ListOptions,
};

/// Wraps the /control endpoints, Payfake's power layer.
///
/// All methods require a JWT token from auth.login().
/// These are dashboard operations, not application-level API calls.
/// The token is passed explicitly to each method, the control namespace
/// intentionally doesn't store it, keeping auth state in the caller.
pub struct ControlNamespace {
    inner: Arc<ClientInner>,
}

impl ControlNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Fetch the current scenario config for the merchant.
    pub async fn get_scenario(&self, token: &str) -> Result<ScenarioConfig, PayfakeError> {
        self.inner
            .request::<serde_json::Value, ScenarioConfig>(
                Method::GET,
                "/api/v1/control/scenario",
                None,
                Some(token),
            )
            .await
    }

    /// Update the scenario config.
    /// Only non-None fields are sent to the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // 30% failure rate with 1 second delay
    /// client.control.update_scenario(token, UpdateScenarioInput {
    ///     failure_rate: Some(0.3),
    ///     delay_ms: Some(1000),
    ///     ..Default::default()
    /// }).await?;
    ///
    /// // Force all charges to fail with insufficient funds
    /// client.control.update_scenario(token, UpdateScenarioInput {
    ///     force_status: Some("failed".to_string()),
    ///     error_code: Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
    ///     ..Default::default()
    /// }).await?;
    /// ```
    pub async fn update_scenario(
        &self,
        token: &str,
        input: UpdateScenarioInput,
    ) -> Result<ScenarioConfig, PayfakeError> {
        self.inner
            .request::<_, ScenarioConfig>(
                Method::PUT,
                "/api/v1/control/scenario",
                Some(&input),
                Some(token),
            )
            .await
    }

    /// Reset scenario config to defaults.
    /// After reset: failure_rate=0, delay_ms=0, no forced status.
    /// All charges will succeed instantly.
    pub async fn reset_scenario(&self, token: &str) -> Result<ScenarioConfig, PayfakeError> {
        self.inner
            .request::<serde_json::Value, ScenarioConfig>(
                Method::POST,
                "/api/v1/control/scenario/reset",
                None,
                Some(token),
            )
            .await
    }

    /// List webhook events with delivery status.
    pub async fn list_webhooks(
        &self,
        token: &str,
        opts: ListOptions,
    ) -> Result<Vec<WebhookEvent>, PayfakeError> {
        // The API wraps the array in a { webhooks: [...] } object —
        // we need an intermediate struct to deserialize it correctly
        // before returning just the Vec to the caller.
        #[derive(serde::Deserialize)]
        struct WebhookListData {
            webhooks: Vec<WebhookEvent>,
        }

        let path = format!(
            "/api/v1/control/webhooks?page={}&per_page={}",
            opts.page, opts.per_page
        );
        let data = self.inner
            .request::<serde_json::Value, WebhookListData>(
                Method::GET,
                &path,
                None,
                Some(token),
            )
            .await?;

        Ok(data.webhooks)
    }

    /// Manually re-trigger delivery for a failed webhook event.
    /// Delivery happens asynchronously — this returns immediately
    /// after scheduling the retry, not after delivery completes.
    pub async fn retry_webhook(
        &self,
        token: &str,
        webhook_id: &str,
    ) -> Result<(), PayfakeError> {
        let path = format!("/api/v1/control/webhooks/{}/retry", webhook_id);
        // The retry endpoint returns null data, we use serde_json::Value
        // as T so the deserialization step is a no-op.
        self.inner
            .request::<serde_json::Value, serde_json::Value>(
                Method::POST,
                &path,
                None,
                Some(token),
            )
            .await?;
        Ok(())
    }

    /// Fetch all delivery attempts for a specific webhook event.
    pub async fn get_webhook_attempts(
        &self,
        token: &str,
        webhook_id: &str,
    ) -> Result<Vec<WebhookAttempt>, PayfakeError> {
        #[derive(serde::Deserialize)]
        struct AttemptsData {
            attempts: Vec<WebhookAttempt>,
        }

        let path = format!("/api/v1/control/webhooks/{}/attempts", webhook_id);
        let data = self.inner
            .request::<serde_json::Value, AttemptsData>(
                Method::GET,
                &path,
                None,
                Some(token),
            )
            .await?;

        Ok(data.attempts)
    }

    /// Force a pending transaction to a specific terminal state.
    ///
    /// This bypasses the scenario engine entirely, the outcome is
    /// always exactly what you specify. Only pending transactions
    /// can be forced.
    ///
    /// status must be one of: "success", "failed", "abandoned"
    pub async fn force_transaction(
        &self,
        token: &str,
        reference: &str,
        input: ForceTransactionInput,
    ) -> Result<Transaction, PayfakeError> {
        let path = format!("/api/v1/control/transactions/{}/force", reference);
        self.inner
            .request::<_, Transaction>(
                Method::POST,
                &path,
                Some(&input),
                Some(token),
            )
            .await
    }

    /// Fetch paginated request/response introspection logs.
    pub async fn get_logs(
        &self,
        token: &str,
        opts: ListOptions,
    ) -> Result<Vec<RequestLog>, PayfakeError> {
        #[derive(serde::Deserialize)]
        struct LogsData {
            logs: Vec<RequestLog>,
        }

        let path = format!(
            "/api/v1/control/logs?page={}&per_page={}",
            opts.page, opts.per_page
        );
        let data = self.inner
            .request::<serde_json::Value, LogsData>(
                Method::GET,
                &path,
                None,
                Some(token),
            )
            .await?;

        Ok(data.logs)
    }

    /// Permanently delete all logs for the merchant.
    pub async fn clear_logs(&self, token: &str) -> Result<(), PayfakeError> {
        self.inner
            .request::<serde_json::Value, serde_json::Value>(
                Method::DELETE,
                "/api/v1/control/logs",
                None,
                Some(token),
            )
            .await?;
        Ok(())
    }

    /// Fetch OTP codes generated during charge flows.
    /// Pass Some(reference) to filter for a specific transaction.
    ///
    /// This is the primary way to read OTPs during testing without a real phone:
    ///
    /// EXAMPLE:
    /// let logs = client.control.get_otp_logs(&token, Some("TXN_xxx"), ListOptions::default()).await?;
    /// let otp = &logs[0].otp_code;
    ///
    pub async fn get_otp_logs(
        &self,
        token: &str,
        reference: Option<&str>,
        opts: ListOptions,
    ) -> Result<Vec<crate::types::OTPLog>, PayfakeError> {
        #[derive(serde::Deserialize)]
        struct OTPLogsData {
            otp_logs: Vec<crate::types::OTPLog>,
        }

        let path = if let Some(r) = reference {
            format!("/api/v1/control/otp-logs?reference={}", r)
        } else {
            format!(
                "/api/v1/control/otp-logs?page={}&per_page={}",
                opts.page, opts.per_page
            )
        };

        let data = self.inner
            .request::<serde_json::Value, OTPLogsData>(
                Method::GET,
                &path,
                None,
                Some(token),
            )
            .await?;

        Ok(data.otp_logs)
    }
}
