use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{
    InitializeInput, InitializeResponse,
    Transaction, TransactionList, ListOptions,
};

/// Wraps the /transaction endpoints.
/// All methods require the secret key set on the client.
pub struct TransactionNamespace {
    inner: Arc<ClientInner>,
}

impl TransactionNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Create a new pending transaction.
    /// Returns the authorization URL for the payment popup and the
    /// access code the popup uses to identify the transaction.
    ///
    /// No money moves here, this just creates the record and
    /// returns the tokens the frontend needs to open the popup.
    pub async fn initialize(
        &self,
        input: InitializeInput,
    ) -> Result<InitializeResponse, PayfakeError> {
        self.inner
            .request::<_, InitializeResponse>(
                Method::POST,
                "/api/v1/transaction/initialize",
                Some(&input),
                None,
            )
            .await
    }

    /// Verify a transaction by reference.
    /// Call this after the payment popup closes to confirm the outcome.
    /// A status of "success" means the charge went through.
    pub async fn verify(&self, reference: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/api/v1/transaction/verify/{}", reference);
        self.inner
            .request::<serde_json::Value, Transaction>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }

    /// Fetch a single transaction by ID.
    pub async fn get(&self, id: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/api/v1/transaction/{}", id);
        self.inner
            .request::<serde_json::Value, Transaction>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }

    /// List transactions with optional pagination.
    /// Defaults to page=1, per_page=50.
    pub async fn list(&self, opts: ListOptions) -> Result<TransactionList, PayfakeError> {
        let path = format!(
            "/api/v1/transaction?page={}&per_page={}",
            opts.page, opts.per_page
        );
        self.inner
            .request::<serde_json::Value, TransactionList>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }

    /// Refund a successful transaction.
    /// Only transactions with status "success" can be refunded.
    /// Returns the updated transaction with status "reversed".
    pub async fn refund(&self, id: &str) -> Result<Transaction, PayfakeError> {
        let path = format!("/api/v1/transaction/{}/refund", id);
        self.inner
            .request::<serde_json::Value, Transaction>(
                Method::POST,
                &path,
                None,
                None,
            )
            .await
    }

    /// Load transaction details for the checkout page.
    /// No secret key required — authenticated via access code in the URL.
    /// Returns amount, currency, merchant branding, customer email
    /// and current charge flow status.
    ///
    /// Call on checkout page mount to hydrate the payment form.
    ///
    /// ```rust
    /// let tx = client.transaction.public_fetch(&access_code).await?;
    /// println!("Pay {} {} {:.2}", tx.merchant_name, tx.currency, tx.amount as f64 / 100.0);
    /// ```
    pub async fn public_fetch(
        &self,
        access_code: &str,
    ) -> Result<PublicTransactionResponse, PayfakeError> {
        let path = format!("/api/v1/public/transaction/{}", access_code);
        self.inner.request::<serde_json::Value, PublicTransactionResponse>(
            Method::GET,
            &path,
            None,
            None,
        ).await
    }

    /// Poll transaction status for MoMo pay_offline state.
    /// No secret key required.
    /// Stop polling when status is "success" or "failed".
    ///
    /// ```rust
    /// loop {
    ///     let result = client.transaction.public_verify(&reference).await?;
    ///     if result.status == "success" || result.status == "failed" {
    ///         break;
    ///     }
    ///     tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    /// }
    /// ```
    pub async fn public_verify(
        &self,
        reference: &str,
    ) -> Result<PublicVerifyResponse, PayfakeError> {
        let path = format!("/api/v1/public/transaction/verify/{}", reference);
        self.inner.request::<serde_json::Value, PublicVerifyResponse>(
            Method::GET,
            &path,
            None,
            None,
        ).await
    }
}
