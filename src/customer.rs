use reqwest::Method;
use std::sync::Arc;

use crate::client::Inner;
use crate::errors::PayfakeError;
use crate::types::{
    CreateCustomerInput, Customer, CustomerList, TransactionList, UpdateCustomerInput,
};

/// Wraps /customer endpoints.
/// Matches https://api.paystack.co/customer exactly.
/// Auth: Bearer sk_test_xxx
pub struct CustomerNamespace(pub(crate) Arc<Inner>);

impl CustomerNamespace {
    pub async fn create(&self, input: CreateCustomerInput) -> Result<Customer, PayfakeError> {
        self.0.do_sk(Method::POST, "/customer", Some(&input)).await
    }

    pub async fn list(&self, page: i32, per_page: i32) -> Result<CustomerList, PayfakeError> {
        let path = format!("/customer?page={}&perPage={}", page, per_page);
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    pub async fn fetch(&self, code: &str) -> Result<Customer, PayfakeError> {
        let path = format!("/customer/{}", code);
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }

    pub async fn update(
        &self,
        code: &str,
        input: UpdateCustomerInput,
    ) -> Result<Customer, PayfakeError> {
        let path = format!("/customer/{}", code);
        self.0.do_sk(Method::PUT, &path, Some(&input)).await
    }

    pub async fn transactions(
        &self,
        code: &str,
        page: i32,
        per_page: i32,
    ) -> Result<TransactionList, PayfakeError> {
        let path = format!(
            "/customer/{}/transactions?page={}&perPage={}",
            code, page, per_page
        );
        self.0
            .do_sk::<serde_json::Value, _>(Method::GET, &path, None)
            .await
    }
}
