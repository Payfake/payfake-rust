

use serde::{Deserialize, Serialize};
use std::collections::HashMap;


// Shared

/// The envelope every Payfake API response follows.
/// The SDK unwraps this internally, callers never see it,
/// they get the typed data or a PayfakeError.
///
/// We use serde's untagged enum trick for the data field
/// it can be any JSON value, we re-deserialize it into the
/// concrete target type after confirming the status is "success".
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    pub status: String,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub errors: Vec<crate::errors::ApiErrorField>,
    #[serde(default)]
    pub code: String,
}

/// Pagination metadata included in all list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub pages: i64,
}

/// Options for paginated list endpoints.
/// Both fields have sensible defaults, page=1, per_page=50.
#[derive(Debug, Clone)]
pub struct ListOptions {
    pub page: i32,
    pub per_page: i32,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self { page: 1, per_page: 50 }
    }
}


// Auth

/// Input for merchant registration.
#[derive(Debug, Serialize)]
pub struct RegisterInput {
    pub business_name: String,
    pub email: String,
    pub password: String,
}

/// Input for merchant login.
#[derive(Debug, Serialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

/// Merchant summary returned in auth responses.
/// SecretKey is intentionally absent, never returned in auth responses.
#[derive(Debug, Clone, Deserialize)]
pub struct MerchantData {
    pub id: String,
    pub business_name: String,
    pub email: String,
    pub public_key: String,
}

/// Returned after successful registration or login.
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub merchant: MerchantData,
    pub token: String,
}

/// The merchant's current key pair.
#[derive(Debug, Deserialize)]
pub struct KeysResponse {
    pub public_key: String,
    pub secret_key: String,
}


// Transaction

/// Input for transaction initialization.
/// Only email and amount are required, everything else has
/// sensible defaults or is optional.
#[derive(Debug, Serialize, Default)]
pub struct InitializeInput {
    pub email: String,
    /// Amount in the smallest currency unit (pesewas for GHS).
    /// 10000 = GHS 100.00
    pub amount: i64,
    /// Defaults to "GHS" on the server if not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// If not provided the server generates one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Returned after successful transaction initialization.
#[derive(Debug, Deserialize)]
pub struct InitializeResponse {
    /// The URL to open for the payment popup.
    pub authorization_url: String,
    /// The token the popup sends with the charge request.
    pub access_code: String,
    pub reference: String,
}

/// A full transaction record.
#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub reference: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub channel: String,
    pub fees: i64,
    pub access_code: String,
    pub callback_url: String,
    pub created_at: String,
    pub paid_at: Option<String>,
    pub customer: CustomerSummary,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Minimal customer data embedded in transaction responses.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CustomerSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub customer_code: String,
}

/// A paginated list of transactions.
#[derive(Debug, Deserialize)]
pub struct TransactionList {
    pub transactions: Vec<Transaction>,
    pub meta: PaginationMeta,
}


// Charge


/// Input for a direct card charge.
/// Provide either access_code (popup flow) or reference (direct API flow).
#[derive(Debug, Serialize, Default)]
pub struct ChargeCardInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub card_number: String,
    pub card_expiry: String,
    pub cvv: String,
    pub email: String,
}

/// Input for a mobile money charge.
#[derive(Debug, Serialize, Default)]
pub struct ChargeMomoInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub phone: String,
    /// One of: "mtn", "vodafone", "airteltigo"
    pub provider: String,
    pub email: String,
}

/// Input for a bank transfer charge.
#[derive(Debug, Serialize, Default)]
pub struct ChargeBankInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub bank_code: String,
    pub account_number: String,
    pub email: String,
}

/// Charge details returned alongside the transaction in charge responses.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChargeData {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub card_brand: String,
    #[serde(default)]
    pub card_last4: String,
    #[serde(default)]
    pub momo_phone: String,
    #[serde(default)]
    pub momo_provider: String,
}

/// Returned after any charge attempt.
#[derive(Debug, Deserialize)]
pub struct ChargeResponse {
    pub transaction: Transaction,
    pub charge: ChargeData,
}


// Customer

/// Input for creating a new customer.
#[derive(Debug, Serialize, Default)]
pub struct CreateCustomerInput {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Input for partially updating a customer.
/// All fields are Option<T>, None means "don't touch this field".
/// skip_serializing_if ensures None fields are absent from the JSON body
/// so the API treats them as "no change requested".
#[derive(Debug, Serialize, Default)]
pub struct UpdateCustomerInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// A full customer record.
#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: String,
    pub customer_code: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A paginated list of customers.
#[derive(Debug, Deserialize)]
pub struct CustomerList {
    pub customers: Vec<Customer>,
    pub meta: PaginationMeta,
}


// Control

/// The merchant's current scenario configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioConfig {
    pub id: String,
    pub merchant_id: String,
    /// Probability of charge failure, between 0.0 (never) and 1.0 (always).
    pub failure_rate: f64,
    /// Artificial delay applied to every charge in milliseconds.
    pub delay_ms: i32,
    /// If set, every charge returns exactly this status regardless of failure_rate.
    pub force_status: String,
    /// Error code returned when force_status is "failed".
    pub error_code: String,
}

/// Input for updating the scenario configuration.
/// All fields are Option<T>, None means "don't change this field".
#[derive(Debug, Serialize, Default)]
pub struct UpdateScenarioInput {
    /// Must be between 0.0 and 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_rate: Option<f64>,
    /// Must be between 0 and 30000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<i32>,
    /// Must be one of: "success", "failed", "abandoned", or "" to clear.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// Input for forcing a transaction to a specific terminal state.
#[derive(Debug, Serialize)]
pub struct ForceTransactionInput {
    /// Must be one of: "success", "failed", "abandoned"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// A webhook event record.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event: String,
    pub transaction_id: String,
    pub delivered: bool,
    pub attempts: i32,
    pub created_at: String,
    pub last_attempt_at: Option<String>,
}

/// A single webhook delivery attempt.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookAttempt {
    pub id: String,
    pub status_code: i32,
    pub response_body: String,
    pub succeeded: bool,
    pub attempted_at: String,
}

/// A request/response introspection log entry.
#[derive(Debug, Clone, Deserialize)]
pub struct RequestLog {
    pub id: String,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub request_body: String,
    pub response_body: String,
    pub duration_ms: i64,
    pub request_id: String,
    pub logged_at: String,
}



//
// Charge flow v0.2.0
//

/// Input for submitting a card PIN.
#[derive(Debug, Serialize, Default)]
pub struct SubmitPINInput {
    pub reference: String,
    pub pin: String,
}

/// Input for submitting an OTP.
/// Works for card, MoMo and bank flows.
#[derive(Debug, Serialize, Default)]
pub struct SubmitOTPInput {
    pub reference: String,
    pub otp: String,
}

/// Input for submitting a date of birth.
#[derive(Debug, Serialize, Default)]
pub struct SubmitBirthdayInput {
    pub reference: String,
    /// Format: YYYY-MM-DD
    pub birthday: String,
}

/// Input for submitting a billing address.
#[derive(Debug, Serialize, Default)]
pub struct SubmitAddressInput {
    pub reference: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

/// Input for requesting a new OTP.
#[derive(Debug, Serialize, Default)]
pub struct ResendOTPInput {
    pub reference: String,
}

/// Returned by every charge step endpoint.
/// Read status to decide what the checkout page renders next.
///
/// Status values:
/// - `send_pin`      → show PIN input form
/// - `send_otp`      → show OTP input form
/// - `send_birthday` → show date of birth form
/// - `send_address`  → show address form
/// - `open_url`      → navigate to three_ds_url
/// - `pay_offline`   → show approve on phone screen, poll transaction
/// - `success`       → show success screen
/// - `failed`        → show failure screen
#[derive(Debug, Deserialize)]
pub struct ChargeFlowResponse {
    pub status: String,
    pub reference: String,
    #[serde(default)]
    pub display_text: String,
    /// Populated only when status is "open_url".
    /// Navigate to this URL in the checkout app's 3DS page.
    #[serde(default)]
    pub three_ds_url: String,
    pub transaction: Option<Transaction>,
    pub charge: Option<ChargeData>,
}

/// A generated OTP stored for developer inspection.
/// OTPs are never returned in API responses — read them here
/// during testing instead of needing a real phone.
#[derive(Debug, Clone, Deserialize)]
pub struct OTPLog {
    pub id: String,
    pub merchant_id: String,
    pub reference: String,
    pub channel: String,
    /// The actual OTP code, use this in submit_otp during testing.
    pub otp_code: String,
    pub step: String,
    /// True if this OTP was successfully submitted.
    pub used: bool,
    pub expires_at: String,
    pub created_at: String,
}


#[derive(Debug, Clone, Deserialize)]
pub struct MerchantProfile {
    pub id: String,
    pub business_name: String,
    pub email: String,
    pub public_key: String,
    #[serde(default)]
    pub webhook_url: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateProfileInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebhookConfig {
    pub webhook_url: String,
    pub is_set: bool,
}

#[derive(Debug, Deserialize)]
pub struct WebhookTestResult {
    pub webhook_url: String,
    pub success: bool,
    pub status_code: i32,
}
