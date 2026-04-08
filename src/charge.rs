use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{
    ChargeCardInput, ChargeMomoInput, ChargeBankInput,
    ChargeResponse, ChargeData,
};

/// Wraps the /charge endpoints.
/// All methods require the secret key set on the client.
pub struct ChargeNamespace {
    inner: Arc<ClientInner>,
}

impl ChargeNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Charge a card directly.
    /// Provide either access_code (popup flow) or reference (direct API flow).
    ///
    /// Returns Err(PayfakeError::Api { code: "CHARGE_FAILED", .. }) on
    /// simulated charge failure, check err.is_code(codes::CHARGE_FAILED).
    pub async fn card(&self, input: ChargeCardInput) -> Result<ChargeResponse, PayfakeError> {
        // Validate that at least one of access_code or reference is set.
        // We catch this client-side to give a clear error before
        // wasting a network round trip on a request that will fail.
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner
            .request::<_, ChargeResponse>(
                Method::POST,
                "/api/v1/charge/card",
                Some(&input),
                None,
            )
            .await
    }

    /// Initiate a mobile money charge.
    ///
    /// Always returns immediately with status "pending", the final
    /// outcome arrives via webhook after the simulated approval window.
    /// Never assume pending means success. Always implement a webhook
    /// handler to receive the charge.success or charge.failed event.
    pub async fn mobile_money(
        &self,
        input: ChargeMomoInput,
    ) -> Result<ChargeResponse, PayfakeError> {
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner
            .request::<_, ChargeResponse>(
                Method::POST,
                "/api/v1/charge/mobile_money",
                Some(&input),
                None,
            )
            .await
    }

    /// Initiate a bank transfer charge.
    pub async fn bank(&self, input: ChargeBankInput) -> Result<ChargeResponse, PayfakeError> {
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner
            .request::<_, ChargeResponse>(
                Method::POST,
                "/api/v1/charge/bank",
                Some(&input),
                None,
            )
            .await
    }

    /// Fetch a charge by transaction reference.
    pub async fn fetch(&self, reference: &str) -> Result<ChargeData, PayfakeError> {
        let path = format!("/api/v1/charge/{}", reference);
        self.inner
            .request::<serde_json::Value, ChargeData>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }
}
