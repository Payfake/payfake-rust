use reqwest::Method;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{
    CustomerList, ForceTransactionInput, LogList, OTPLog, ScenarioConfig, Transaction,
    TransactionList, UpdateScenarioInput, WebhookAttempt, WebhookList,
};

/// Wraps /api/v1/control endpoints.
/// Payfake-specific — no Paystack equivalent.
/// Auth: Bearer JWT (from auth.login)
pub struct ControlNamespace(pub(crate) Arc<Inner>);

impl ControlNamespace {
    pub async fn get_stats(&self, token: &str) -> Result<serde_json::Value, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/control/stats", None, token)
            .await
    }

    pub async fn get_scenario(&self, token: &str) -> Result<ScenarioConfig, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, "/api/v1/control/scenario", None, token)
            .await
    }

    /// Update scenario config.
    ///
    /// ```rust
    /// // Force all charges to fail
    /// client.control.update_scenario(token, UpdateScenarioInput {
    ///     force_status: Some("failed".to_string()),
    ///     error_code:   Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
    ///     ..Default::default()
    /// }).await?;
    ///
    /// // 30% random failure with 2 second delay
    /// client.control.update_scenario(token, UpdateScenarioInput {
    ///     failure_rate: Some(0.3),
    ///     delay_ms:     Some(2000),
    ///     ..Default::default()
    /// }).await?;
    /// ```
    pub async fn update_scenario(
        &self,
        token: &str,
        input: UpdateScenarioInput,
    ) -> Result<ScenarioConfig, PayfakeError> {
        self.0
            .do_jwt(Method::PUT, "/api/v1/control/scenario", Some(&input), token)
            .await
    }

    /// Reset scenario to defaults. All charges succeed with no delay.
    pub async fn reset_scenario(&self, token: &str) -> Result<ScenarioConfig, PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, _>(
                Method::POST,
                "/api/v1/control/scenario/reset",
                None,
                token,
            )
            .await
    }

    pub async fn list_transactions(
        &self,
        token: &str,
        page: i32,
        per_page: i32,
        status: Option<&str>,
        search: Option<&str>,
    ) -> Result<TransactionList, PayfakeError> {
        let mut path = format!(
            "/api/v1/control/transactions?page={}&perPage={}",
            page, per_page
        );
        if let Some(s) = status {
            path.push_str(&format!("&status={}", s));
        }
        if let Some(q) = search {
            path.push_str(&format!("&search={}", q));
        }
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await
    }

    pub async fn list_customers(
        &self,
        token: &str,
        page: i32,
        per_page: i32,
    ) -> Result<CustomerList, PayfakeError> {
        let path = format!(
            "/api/v1/control/customers?page={}&perPage={}",
            page, per_page
        );
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await
    }

    pub async fn force_transaction(
        &self,
        token: &str,
        reference: &str,
        input: ForceTransactionInput,
    ) -> Result<Transaction, PayfakeError> {
        let path = format!("/api/v1/control/transactions/{}/force", reference);
        self.0
            .do_jwt(Method::POST, &path, Some(&input), token)
            .await
    }

    pub async fn list_webhooks(
        &self,
        token: &str,
        page: i32,
        per_page: i32,
    ) -> Result<WebhookList, PayfakeError> {
        let path = format!(
            "/api/v1/control/webhooks?page={}&perPage={}",
            page, per_page
        );
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await
    }

    pub async fn retry_webhook(&self, token: &str, id: &str) -> Result<(), PayfakeError> {
        let path = format!("/api/v1/control/webhooks/{}/retry", id);
        self.0
            .do_jwt::<serde_json::Value, serde_json::Value>(Method::POST, &path, None, token)
            .await?;
        Ok(())
    }

    pub async fn get_webhook_attempts(
        &self,
        token: &str,
        id: &str,
    ) -> Result<Vec<WebhookAttempt>, PayfakeError> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: Vec<WebhookAttempt>,
        }
        let path = format!("/api/v1/control/webhooks/{}/attempts", id);
        let r: Resp = self
            .0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await?;
        Ok(r.data)
    }

    pub async fn get_logs(
        &self,
        token: &str,
        page: i32,
        per_page: i32,
    ) -> Result<LogList, PayfakeError> {
        let path = format!("/api/v1/control/logs?page={}&perPage={}", page, per_page);
        self.0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await
    }

    pub async fn clear_logs(&self, token: &str) -> Result<(), PayfakeError> {
        self.0
            .do_jwt::<serde_json::Value, serde_json::Value>(
                Method::DELETE,
                "/api/v1/control/logs",
                None,
                token,
            )
            .await?;
        Ok(())
    }

    /// Get OTP codes generated during charge flows.
    /// Primary way to read OTPs during testing without a real phone.
    ///
    /// ```rust
    /// let logs = client.control.get_otp_logs(token, Some(&reference), 1, 10).await?;
    /// let otp  = &logs[0].otp_code;
    /// ```
    ///
    /// Pass `Some(reference)` to filter for a specific transaction.
    /// Pass `None` to list all OTP logs paginated.
    pub async fn get_otp_logs(
        &self,
        token: &str,
        reference: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<Vec<OTPLog>, PayfakeError> {
        let path = if let Some(r) = reference {
            format!("/api/v1/control/otp-logs?reference={}", r)
        } else {
            format!(
                "/api/v1/control/otp-logs?page={}&perPage={}",
                page, per_page
            )
        };

        // Handle both flat array and paginated { data: [], meta: {} } shapes
        let raw: serde_json::Value = self
            .0
            .do_jwt::<serde_json::Value, _>(Method::GET, &path, None, token)
            .await?;

        if let Some(arr) = raw.as_array() {
            return Ok(serde_json::from_value(serde_json::Value::Array(
                arr.clone(),
            ))?);
        }
        if let Some(data) = raw.get("data") {
            return Ok(serde_json::from_value(data.clone())?);
        }
        Ok(vec![])
    }
}
