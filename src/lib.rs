//! # payfake
//!
//! Official Rust SDK for [Payfake](https://payfake.co) — a Paystack-compatible
//! payment simulator for African developers.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use payfake::{Client, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new(Config {
//!         secret_key: "sk_test_xxx".to_string(),
//!         base_url:   None,   // defaults to https://api.payfake.co
//!         timeout:    None,
//!     });
//!
//!     let tx = client.transaction.initialize(payfake::types::InitializeInput {
//!         email:  "customer@example.com".to_string(),
//!         amount: 10000,
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("{}", tx.authorization_url);
//!     Ok(())
//! }
//! ```

mod client;

pub mod auth;
pub mod charge;
pub mod control;
pub mod customer;
pub mod errors;
pub mod merchant;
pub mod transaction;
pub mod types;

pub use errors::{codes, ErrorField, PayfakeError};
pub use types::*;

use client::Inner;
use std::sync::Arc;
use std::time::Duration;

/// Client configuration.
pub struct Config {
    /// Merchant sk_test_xxx key — required for Paystack-compatible endpoints.
    pub secret_key: String,
    /// Base URL. Defaults to `https://api.payfake.co`.
    /// For self-hosted: `Some("http://localhost:8080".to_string())`
    pub base_url: Option<String>,
    /// HTTP timeout. Defaults to 30 seconds.
    pub timeout: Option<Duration>,
}

/// Payfake SDK client.
///
/// All API namespaces hang off this struct:
/// - `client.auth`        — register, login, keys
/// - `client.transaction` — initialize, verify, fetch, list, refund
/// - `client.charge`      — card, mobile_money, bank, submit_*, simulate_3ds
/// - `client.customer`    — create, list, fetch, update
/// - `client.merchant`    — profile, webhook management
/// - `client.control`     — stats, scenarios, logs, OTP logs, force
pub struct Client {
    pub auth: auth::AuthNamespace,
    pub transaction: transaction::TransactionNamespace,
    pub charge: charge::ChargeNamespace,
    pub customer: customer::CustomerNamespace,
    pub merchant: merchant::MerchantNamespace,
    pub control: control::ControlNamespace,
}

impl Client {
    /// Create a new Payfake client.
    pub fn new(cfg: Config) -> Self {
        let base_url = cfg
            .base_url
            .unwrap_or_else(|| "https://api.payfake.co".to_string())
            .trim_end_matches('/')
            .to_string();

        let timeout = cfg.timeout.unwrap_or(Duration::from_secs(30));

        let http = reqwest::Client::builder()
            .timeout(timeout)
            .use_rustls_tls()
            .build()
            .expect("failed to build HTTP client");

        let inner = Arc::new(Inner {
            base_url,
            secret_key: cfg.secret_key,
            http,
        });

        Client {
            auth: auth::AuthNamespace(Arc::clone(&inner)),
            transaction: transaction::TransactionNamespace(Arc::clone(&inner)),
            charge: charge::ChargeNamespace(Arc::clone(&inner)),
            customer: customer::CustomerNamespace(Arc::clone(&inner)),
            merchant: merchant::MerchantNamespace(Arc::clone(&inner)),
            control: control::ControlNamespace(Arc::clone(&inner)),
        }
    }
}
