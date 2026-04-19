use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{
    ChargeCardInput, ChargeMomoInput, ChargeBankInput,
    ChargeFlowResponse, ChargeData,
    SubmitPINInput, SubmitOTPInput, SubmitBirthdayInput,
    SubmitAddressInput, ResendOTPInput,
};

/// Wraps all charge flow endpoints.
/// Every method returns ChargeFlowResponse, read status to determine next step.
pub struct ChargeNamespace {
    inner: Arc<ClientInner>,
}

impl ChargeNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Initiate a card charge.
    ///
    /// Local Verve cards (5061, 5062, 5063, 6500, 6501):
    ///   returns status "send_pin" → call submit_pin next
    ///
    /// International Visa/Mastercard:
    ///   returns status "open_url" → navigate to three_ds_url
    pub async fn card(&self, input: ChargeCardInput) -> Result<ChargeFlowResponse, PayfakeError> {
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/card",
            Some(&input),
            None,
        ).await
    }

    /// Initiate a mobile money charge.
    /// Always returns status "send_otp" first, customer verifies phone.
    /// After OTP returns "pay_offline", poll transaction for webhook outcome.
    pub async fn mobile_money(
        &self,
        input: ChargeMomoInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/mobile_money",
            Some(&input),
            None,
        ).await
    }

    /// Initiate a bank transfer charge.
    /// Returns status "send_birthday", customer enters DOB first.
    pub async fn bank(
        &self,
        input: ChargeBankInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        if input.access_code.is_none() && input.reference.is_none() {
            return Err(PayfakeError::InvalidInput(
                "either access_code or reference is required".to_string(),
            ));
        }
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/bank",
            Some(&input),
            None,
        ).await
    }

    /// Submit card PIN after status "send_pin".
    /// Returns status "send_otp", OTP sent to registered phone.
    /// Read OTP from client.control.get_otp_logs(token, Some(reference)).
    pub async fn submit_pin(
        &self,
        input: SubmitPINInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/submit_pin",
            Some(&input),
            None,
        ).await
    }

    /// Submit OTP after status "send_otp".
    /// Card/bank: returns "success" or "failed".
    /// MoMo: returns "pay_offline", poll transaction for final outcome.
    pub async fn submit_otp(
        &self,
        input: SubmitOTPInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/submit_otp",
            Some(&input),
            None,
        ).await
    }

    /// Submit date of birth after status "send_birthday".
    /// Returns status "send_otp" on success.
    pub async fn submit_birthday(
        &self,
        input: SubmitBirthdayInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/submit_birthday",
            Some(&input),
            None,
        ).await
    }

    /// Submit billing address after status "send_address".
    /// Returns "success" or "failed".
    pub async fn submit_address(
        &self,
        input: SubmitAddressInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/submit_address",
            Some(&input),
            None,
        ).await
    }

    /// Request a new OTP when the customer hasn't received one.
    /// Invalidates the previous OTP. Returns "send_otp" with fresh OTP.
    /// Read new OTP from client.control.get_otp_logs(token, Some(reference)).
    pub async fn resend_otp(
        &self,
        input: ResendOTPInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.inner.request::<_, ChargeFlowResponse>(
            Method::POST,
            "/api/v1/charge/resend_otp",
            Some(&input),
            None,
        ).await
    }

    /// Complete the simulated 3DS verification.
    /// Called after the customer confirms on the checkout app's 3DS page.
    /// Returns "success" or "failed" based on the scenario config.
    pub async fn simulate_3ds(
        &self,
        reference: &str,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        let path = format!("/api/v1/public/simulate/3ds/{}", reference);
        self.inner.request::<serde_json::Value, ChargeFlowResponse>(
            Method::POST,
            &path,
            None,
            None,
        ).await
    }

    /// Fetch the current state of a charge by transaction reference.
    pub async fn fetch(&self, reference: &str) -> Result<ChargeData, PayfakeError> {
        let path = format!("/api/v1/charge/{}", reference);
        self.inner.request::<serde_json::Value, ChargeData>(
            Method::GET,
            &path,
            None,
            None,
        ).await
    }
}
