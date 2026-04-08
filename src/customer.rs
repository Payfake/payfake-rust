use std::sync::Arc;
use reqwest::Method;

use crate::client::ClientInner;
use crate::errors::PayfakeError;
use crate::types::{
    CreateCustomerInput, UpdateCustomerInput,
    Customer, CustomerList, TransactionList, ListOptions,
};

/// Wraps the /customer endpoints.
/// All methods require the secret key set on the client.
pub struct CustomerNamespace {
    inner: Arc<ClientInner>,
}

impl CustomerNamespace {
    pub(crate) fn new(inner: Arc<ClientInner>) -> Self {
        Self { inner }
    }

    /// Create a new customer under the merchant account.
    /// Returns PayfakeError::Api with code CUSTOMER_EMAIL_TAKEN
    /// if a customer with this email already exists.
    pub async fn create(&self, input: CreateCustomerInput) -> Result<Customer, PayfakeError> {
        self.inner
            .request::<_, Customer>(
                Method::POST,
                "/api/v1/customer",
                Some(&input),
                None,
            )
            .await
    }

    /// List customers with optional pagination.
    pub async fn list(&self, opts: ListOptions) -> Result<CustomerList, PayfakeError> {
        let path = format!(
            "/api/v1/customer?page={}&per_page={}",
            opts.page, opts.per_page
        );
        self.inner
            .request::<serde_json::Value, CustomerList>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }

    /// Fetch a customer by their code (CUS_xxxxxxxx).
    pub async fn get(&self, code: &str) -> Result<Customer, PayfakeError> {
        let path = format!("/api/v1/customer/{}", code);
        self.inner
            .request::<serde_json::Value, Customer>(
                Method::GET,
                &path,
                None,
                None,
            )
            .await
    }

    /// Partially update a customer.
    /// Fields set to None in UpdateCustomerInput are not sent
    /// the API treats absent fields as "no change".
    pub async fn update(
        &self,
        code: &str,
        input: UpdateCustomerInput,
    ) -> Result<Customer, PayfakeError> {
        let path = format!("/api/v1/customer/{}", code);
        self.inner
            .request::<_, Customer>(Method::PUT, &path, Some(&input), None)
            .await
    }

    /// Fetch paginated transactions for a specific customer.
    pub async fn transactions(
        &self,
        code: &str,
        opts: ListOptions,
    ) -> Result<TransactionList, PayfakeError> {
        let path = format!(
            "/api/v1/customer/{}/transactions?page={}&per_page={}",
            code, opts.page, opts.per_page
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
}
