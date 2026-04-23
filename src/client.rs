use reqwest::{header, Client as ReqwestClient, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

use crate::errors::{ErrorField, PayfakeError};
use crate::types::{Envelope, ValidationRule};

const DEFAULT_BASE_URL: &str = "https://api.payfake.co";

/// Shared inner client — reference-counted so namespaces can hold cheap clones.
pub(crate) struct Inner {
    pub(crate) base_url: String,
    pub(crate) secret_key: String,
    pub(crate) http: ReqwestClient,
}

impl Inner {
    /// Authenticated with secret key — Paystack-compatible endpoints.
    pub(crate) async fn do_sk<B, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<R, PayfakeError>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        self.request(method, path, body, Some(&self.secret_key))
            .await
    }

    /// Authenticated with JWT — Payfake-specific endpoints.
    pub(crate) async fn do_jwt<B, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        token: &str,
    ) -> Result<R, PayfakeError>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        self.request(
            method,
            path,
            body,
            if token.is_empty() { None } else { Some(token) },
        )
        .await
    }

    /// Unauthenticated — public checkout endpoints.
    pub(crate) async fn do_public<B, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<R, PayfakeError>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        self.request(method, path, body, None).await
    }

    async fn request<B, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        token: Option<&str>,
    ) -> Result<R, PayfakeError>
    where
        B: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        let mut builder = self
            .http
            .request(method, &url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "application/json");

        if let Some(t) = token {
            builder = builder.bearer_auth(t);
        }

        if let Some(b) = body {
            builder = builder.json(b);
        }

        let resp: Response = builder.send().await?;
        let payfake_code = resp
            .headers()
            .get("X-Payfake-Code")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("UNKNOWN_ERROR")
            .to_string();
        let http_status = resp.status().as_u16();

        let envelope: Envelope = resp.json().await?;

        // status is a boolean in the Paystack/Payfake envelope.
        // false means an error regardless of HTTP status code.
        if !envelope.status {
            let fields: Vec<ErrorField> = envelope
                .errors
                .into_iter()
                .flat_map(|(field_name, rules)| {
                    rules.into_iter().map(move |rule| ErrorField {
                        field: field_name.clone(),
                        rule: rule.rule,
                        message: rule.message,
                    })
                })
                .collect();

            return Err(PayfakeError::Api {
                code: payfake_code,
                message: envelope.message,
                fields,
                http_status,
            });
        }

        let result: R = serde_json::from_value(envelope.data)?;
        Ok(result)
    }
}
