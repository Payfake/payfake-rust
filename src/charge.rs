use reqwest::Method;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{
    ChargeBankInput, ChargeCardInput, ChargeFlowResponse, ChargeMomoInput, ResendOTPInput,
    SubmitAddressInput, SubmitBirthdayInput, SubmitOTPInput, SubmitPINInput,
};

/// Wraps /charge endpoints.
/// Matches https://api.paystack.co/charge exactly.
/// All methods call POST /charge, channel detected from body object.
/// Auth: Bearer sk_test_xxx
pub struct ChargeNamespace(pub(crate) Arc<Inner>);

impl ChargeNamespace {
    /// Initiate a card charge via POST /charge.
    ///
    /// Local Ghana cards (Verve: 5061, 5062, 5063, 6500, 6501):
    ///   returns status "send_pin" → call submit_pin
    ///
    /// International cards (Visa 4xxx, Mastercard 5xxx):
    ///   returns status "open_url" + url → checkout navigates to url
    pub async fn card(&self, input: ChargeCardInput) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0.do_sk(Method::POST, "/charge", Some(&input)).await
    }

    /// Initiate a MoMo charge via POST /charge.
    /// Returns status "send_otp" → call submit_otp.
    /// After OTP returns "pay_offline" → poll transaction.public_verify.
    pub async fn mobile_money(
        &self,
        input: ChargeMomoInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0.do_sk(Method::POST, "/charge", Some(&input)).await
    }

    /// Initiate a bank transfer charge via POST /charge.
    /// Returns status "send_birthday" → call submit_birthday.
    pub async fn bank(&self, input: ChargeBankInput) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0.do_sk(Method::POST, "/charge", Some(&input)).await
    }

    /// Submit card PIN after status "send_pin".
    /// Returns status "send_otp" — OTP sent to registered phone.
    /// Read OTP from control.get_otp_logs() during testing.
    pub async fn submit_pin(
        &self,
        input: SubmitPINInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/charge/submit_pin", Some(&input))
            .await
    }

    /// Submit OTP after status "send_otp".
    /// Card/bank: returns "success" or "failed".
    /// MoMo: returns "pay_offline" — poll transaction.public_verify.
    pub async fn submit_otp(
        &self,
        input: SubmitOTPInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/charge/submit_otp", Some(&input))
            .await
    }

    /// Submit date of birth after status "send_birthday".
    /// Returns status "send_otp" on success.
    pub async fn submit_birthday(
        &self,
        input: SubmitBirthdayInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/charge/submit_birthday", Some(&input))
            .await
    }

    /// Submit billing address after status "send_address".
    /// Returns "success" or "failed".
    pub async fn submit_address(
        &self,
        input: SubmitAddressInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/charge/submit_address", Some(&input))
            .await
    }

    /// Request a fresh OTP when the customer hasn't received one.
    /// Invalidates the previous OTP. Returns status "send_otp".
    pub async fn resend_otp(
        &self,
        input: ResendOTPInput,
    ) -> Result<ChargeFlowResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/charge/resend_otp", Some(&input))
            .await
    }

    /// Fetch the current state of a charge by transaction reference.
    pub async fn fetch(&self, reference: &str) -> Result<ChargeFlowResponse, PayfakeError> {
        let path = format!("/charge/{}", reference);
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    /// Complete the simulated 3DS verification.
    /// Called by the checkout app after the customer confirms on the 3DS page.
    pub async fn simulate_3ds(&self, reference: &str) -> Result<ChargeFlowResponse, PayfakeError> {
        let path = format!("/api/v1/public/simulate/3ds/{}", reference);
        self.0
            .do_public::<serde_json::Value, _>(Method::POST, &path, None)
            .await
    }
}
