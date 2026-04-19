//! # payfake
//!
//! Official Rust SDK for [Payfake](https://github.com/GordenArcher/payfake) —
//! a self-hostable African payment simulator that mirrors the Paystack API
//! exactly. Test every payment scenario without touching real money.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use payfake::{Client, Config};
//! use payfake::types::{InitializeInput, ChargeCardInput};
//! use payfake::errors::codes;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new(Config {
//!         secret_key: "sk_test_xxx".to_string(),
//!         base_url: None,
//!         timeout_secs: None,
//!     });
//!
//!     // Initialize a transaction
//!     let tx = client.transaction.initialize(InitializeInput {
//!         email: "customer@example.com".to_string(),
//!         amount: 10000,
//!         currency: Some("GHS".to_string()),
//!         ..Default::default()
//!     }).await?;
//!
//!     // Charge a card
//!     let charge = client.charge.card(ChargeCardInput {
//!         access_code: Some(tx.access_code),
//!         card_number: "4111111111111111".to_string(),
//!         card_expiry: "12/26".to_string(),
//!         cvv: "123".to_string(),
//!         email: "customer@example.com".to_string(),
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("Status: {}", charge.transaction.status);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod charge;
pub mod client;
pub mod control;
pub mod customer;
pub mod errors;
pub mod transaction;
pub mod types;

pub use types::{
    InitializeInput, InitializeResponse,
    ChargeCardInput, ChargeMomoInput, ChargeBankInput,
    CreateCustomerInput, UpdateCustomerInput,
    UpdateScenarioInput, ForceTransactionInput,
    ListOptions,
    // new v0.2.0
    SubmitPINInput, SubmitOTPInput, SubmitBirthdayInput,
    SubmitAddressInput, ResendOTPInput,
    ChargeFlowResponse, OTPLog,
};

pub use types::{
    MerchantProfile, UpdateProfileInput,
    WebhookConfig, WebhookTestResult,
};

pub use types::{
    PublicTransactionResponse, PublicVerifyResponse,
    PublicChargeStatus, PublicMerchant, PublicCustomer,
};

// Re-export the most commonly used items at the crate root
// so callers can do `use payfake::Client` instead of
// `use payfake::client::Client`. Convenience re-exports
// are standard practice in Rust library crates.
pub use client::{Client, Config};
pub use errors::PayfakeError;
