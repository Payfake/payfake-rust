use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//  Internal envelope

/// Paystack-compatible response envelope.
/// status is boolean, true on success, false on any failure.
#[derive(Debug, Deserialize)]
pub(crate) struct Envelope {
    pub status: bool,
    pub message: String,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub errors: HashMap<String, Vec<ValidationRule>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ValidationRule {
    pub rule: String,
    pub message: String,
}

//  Pagination

/// Paystack-compatible pagination metadata.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PaginationMeta {
    pub total: i64,
    pub skipped: i32,
    pub per_page: i32,
    pub page: i32,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
}

//  Auth

#[derive(Debug, Serialize)]
pub struct RegisterInput {
    pub business_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    pub merchant: MerchantSummary,
    pub access_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MerchantSummary {
    pub id: String,
    pub business_name: String,
    pub email: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeysResponse {
    pub public_key: String,
    pub secret_key: String,
}

//  Transaction

#[derive(Debug, Serialize, Default)]
pub struct InitializeInput {
    pub email: String,
    pub amount: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub currency: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reference: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub callback_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Matches Paystack's initialize response exactly:
/// { "authorization_url": "...", "access_code": "...", "reference": "..." }
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeResponse {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

/// Full transaction object matching Paystack's shape.
#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    pub id: String,
    #[serde(default)]
    pub domain: String,
    pub status: String,
    pub reference: String,
    pub amount: i64,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub gateway_response: String,
    #[serde(default)]
    pub paid_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub fees: i64,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default)]
    pub customer: Option<CustomerSummary>,
    #[serde(default)]
    pub authorization: Option<Authorization>,
}

/// Payment instrument object, store authorization_code for recurring charges.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Authorization {
    #[serde(default)]
    pub authorization_code: String,
    #[serde(default)]
    pub bin: String,
    #[serde(default)]
    pub last4: String,
    #[serde(default)]
    pub exp_month: String,
    #[serde(default)]
    pub exp_year: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub card_type: String,
    #[serde(default)]
    pub bank: String,
    #[serde(default)]
    pub country_code: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub reusable: bool,
    #[serde(default)]
    pub signature: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionList {
    pub data: Vec<Transaction>,
    pub meta: PaginationMeta,
}

/// Returned by GET /api/v1/public/transaction/:access_code.
#[derive(Debug, Clone, Deserialize)]
pub struct PublicTransactionResponse {
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub reference: String,
    #[serde(default)]
    pub callback_url: String,
    #[serde(default)]
    pub access_code: String,
    #[serde(default)]
    pub merchant: Option<PublicMerchant>,
    #[serde(default)]
    pub customer: Option<PublicCustomer>,
    #[serde(default)]
    pub charge: Option<PublicCharge>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PublicMerchant {
    pub business_name: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PublicCustomer {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

/// Charge state embedded in public transaction responses.
/// Check flow_status during MoMo polling.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PublicCharge {
    #[serde(default)]
    pub flow_status: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub channel: String,
}

/// Returned by GET /api/v1/public/transaction/verify/:reference.
#[derive(Debug, Clone, Deserialize)]
pub struct PublicVerifyResponse {
    pub status: String,
    pub reference: String,
    pub amount: i64,
    pub currency: String,
    #[serde(default)]
    pub paid_at: Option<String>,
    #[serde(default)]
    pub charge: Option<PublicCharge>,
}

//  Charge

#[derive(Debug, Clone, Serialize, Default)]
pub struct CardDetails {
    pub number: String,
    pub cvv: String,
    pub expiry_month: String,
    pub expiry_year: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MomoDetails {
    pub phone: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BankDetails {
    pub code: String,
    pub account_number: String,
}

#[derive(Debug, Serialize, Default)]
pub struct ChargeCardInput {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub card: CardDetails,
}

#[derive(Debug, Serialize, Default)]
pub struct ChargeMomoInput {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub mobile_money: MomoDetails,
}

#[derive(Debug, Serialize, Default)]
pub struct ChargeBankInput {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    pub bank: BankDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitPINInput {
    pub reference: String,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitOTPInput {
    pub reference: String,
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitBirthdayInput {
    pub reference: String,
    /// Format: YYYY-MM-DD
    pub birthday: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitAddressInput {
    pub reference: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct ResendOTPInput {
    pub reference: String,
}

/// Returned by every charge step endpoint.
///
/// Read `status` to decide what the checkout page renders next:
/// - `"send_pin"`      → show PIN input, call `submit_pin`
/// - `"send_otp"`      → show OTP input, call `submit_otp`
/// - `"send_birthday"` → show DOB input, call `submit_birthday`
/// - `"send_address"`  → show address form, call `submit_address`
/// - `"open_url"`      → open `url` in checkout app 3DS page
/// - `"pay_offline"`   → show approve on phone screen, poll `public_verify`
/// - `"success"`       → payment complete, webhook fired
/// - `"failed"`        → payment declined
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChargeFlowResponse {
    pub status: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub display_text: String,
    /// Populated when status is "open_url" (3DS).
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub channel: String,
    #[serde(default)]
    pub gateway_response: String,
    #[serde(default)]
    pub authorization: Option<Authorization>,
}

/// A generated OTP stored for developer inspection.
/// OTPs are never returned in API responses, read them here during testing.
#[derive(Debug, Clone, Deserialize)]
pub struct OTPLog {
    pub id: String,
    pub merchant_id: String,
    pub reference: String,
    pub channel: String,
    /// The 6-digit OTP — use this in submit_otp during testing.
    pub otp_code: String,
    pub step: String,
    pub used: bool,
    pub expires_at: String,
    pub created_at: String,
}

//  Customer

#[derive(Debug, Serialize, Default)]
pub struct CreateCustomerInput {
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub first_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateCustomerInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: String,
    pub customer_code: String,
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub domain: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CustomerSummary {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub customer_code: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomerList {
    pub data: Vec<Customer>,
    pub meta: PaginationMeta,
}

//  Merchant
#[derive(Debug, Clone, Deserialize)]
pub struct MerchantProfile {
    pub id: String,
    pub business_name: String,
    pub email: String,
    pub public_key: String,
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateProfileInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    pub webhook_url: String,
    pub is_set: bool,
}

//  Control

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScenarioConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub merchant_id: String,
    pub failure_rate: f64,
    pub delay_ms: i32,
    #[serde(default)]
    pub force_status: String,
    #[serde(default)]
    pub error_code: String,
}

#[derive(Debug, Serialize, Default)]
pub struct UpdateScenarioInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ForceTransactionInput {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event: String,
    pub transaction_id: String,
    pub delivered: bool,
    pub attempts: i32,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookAttempt {
    pub id: String,
    pub status_code: i32,
    pub response_body: String,
    pub succeeded: bool,
    pub attempted_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookList {
    pub data: Vec<WebhookEvent>,
    pub meta: PaginationMeta,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct LogList {
    pub data: Vec<RequestLog>,
    pub meta: PaginationMeta,
}
