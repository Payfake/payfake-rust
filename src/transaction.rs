use reqwest::Method;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{
    InitializeInput, InitializeResponse, PublicTransactionResponse, PublicVerifyResponse,
    Transaction, TransactionList,
};

/// Wraps /transaction endpoints.
/// Matches https://api.paystack.co/transaction exactly.
/// Auth: Bearer sk_test_xxx
pub struct TransactionNamespace(pub(crate) Arc<Inner>);

impl TransactionNamespace {
    /// Create a new pending transaction.
    /// Returns authorization_url, access_code and reference.
    pub async fn initialize(
        &self,
        input: InitializeInput,
    ) -> Result<InitializeResponse, PayfakeError> {
        self.0
            .do_sk(Method::POST, "/transaction/initialize", Some(&input))
            .await
    }

    /// Verify a transaction by reference.
    /// Always call this before delivering value.
    pub async fn verify(&self, reference: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/transaction/verify/{}", reference);
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    /// Fetch a transaction by ID.
    pub async fn fetch(&self, id: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/transaction/{}", id);
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    /// List transactions with pagination.
    /// status: "success" | "failed" | "pending" | "abandoned"
    pub async fn list(
        &self,
        page: i32,
        per_page: i32,
        status: Option<&str>,
    ) -> Result<TransactionList, PayfakeError> {
        let mut path = format!("/transaction?page={}&perPage={}", page, per_page);
        if let Some(s) = status {
            path.push_str(&format!("&status={}", s));
        }
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    /// Refund (reverse) a successful transaction.
    pub async fn refund(&self, id: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/transaction/{}/refund", id);
        self.0
            .do_sk::<serde_json::Value, _>(Method::POST, &path, None)
            .await
    }

    /// Load transaction details for the checkout page using the access code.
    /// No secret key required. Call this on checkout page mount.
    pub async fn public_fetch(
        &self,
        access_code: &str,
    ) -> Result<PublicTransactionResponse, PayfakeError> {
        let path = format!("/api/v1/public/transaction/{}", access_code);
        self.0
            .do_public::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    /// Poll transaction status for MoMo pay_offline state.
    /// No secret key required. Poll every 3 seconds until status is success or failed.
    ///
    /// ```rust
    /// loop {
    ///     let r = client.transaction.public_verify(&reference).await?;
    ///     if r.status == "success" || r.status == "failed" { break; }
    ///     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    /// }
    /// ```
    pub async fn public_verify(
        &self,
        reference: &str,
        access_code: &str,
    ) -> Result<PublicVerifyResponse, PayfakeError> {
        let path = format!(
            "/api/v1/public/transaction/verify/{}?access_code={}",
            reference, access_code
        );

        self.0
            .do_public::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }
}
