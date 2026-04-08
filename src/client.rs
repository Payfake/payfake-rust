

use reqwest::{Client as HttpClient, Method};
use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{ApiErrorField, PayfakeError};
use crate::types::ApiResponse;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";

/// Configuration for the Payfake SDK client.
pub struct Config {
    /// Merchant secret key (sk_test_xxx).
    /// Required for all transaction, charge and customer endpoints.
    pub secret_key: String,
    /// Payfake server URL. Defaults to http://localhost:8080.
    pub base_url: Option<String>,
    /// Request timeout in seconds. Defaults to 30.
    pub timeout_secs: Option<u64>,
}

/// The root Payfake SDK client.
///
/// All API namespaces are accessible as fields:
/// `client.transaction`, `client.charge`, `client.customer` etc.
///
/// The client is cheaply cloneable, it wraps an Arc internally
/// via reqwest's Client which uses a connection pool. Clone it
/// freely to share across tasks or threads.
///
/// # Example
///
/// ```rust
/// use payfake::{Client, Config};
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new(Config {
///         secret_key: "sk_test_xxx".to_string(),
///         base_url: None,
///         timeout_secs: None,
///     });
///
///     let tx = client.transaction.initialize(...).await.unwrap();
/// }
/// ```
pub struct Client {
    pub(crate) base_url: String,
    pub(crate) secret_key: String,
    pub(crate) http: HttpClient,

    pub auth:        crate::auth::AuthNamespace,
    pub transaction: crate::transaction::TransactionNamespace,
    pub charge:      crate::charge::ChargeNamespace,
    pub customer:    crate::customer::CustomerNamespace,
    pub control:     crate::control::ControlNamespace,
}

impl Client {
    /// Create a new Payfake SDK client.
    pub fn new(cfg: Config) -> Self {
        let base_url = cfg.base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let timeout = std::time::Duration::from_secs(
            cfg.timeout_secs.unwrap_or(30)
        );

        // reqwest::Client manages a connection pool internally
        // building it is expensive so we do it once here and reuse
        // it for every request. This is the standard pattern in Rust
        // HTTP code: one Client per application, not per request.
        let http = HttpClient::builder()
            .timeout(timeout)
            // Use rustls (pure Rust TLS) instead of native-tls (OpenSSL).
            // Avoids linking against OpenSSL which simplifies cross-compilation
            // to targets like ARM (common for African server deployments).
            .use_rustls_tls()
            .build()
            .expect("failed to build HTTP client");

        // We use Arc<Inner> via a helper struct so namespaces can hold
        // a reference to shared client state without lifetime issues.
        // Each namespace stores a ClientRef which is just a wrapper
        // around the shared state they need.
        let inner = std::sync::Arc::new(ClientInner {
            base_url: base_url.clone(),
            secret_key: cfg.secret_key.clone(),
            http: http.clone(),
        });

        Self {
            base_url,
            secret_key: cfg.secret_key,
            http,
            auth:        crate::auth::AuthNamespace::new(inner.clone()),
            transaction: crate::transaction::TransactionNamespace::new(inner.clone()),
            charge:      crate::charge::ChargeNamespace::new(inner.clone()),
            customer:    crate::customer::CustomerNamespace::new(inner.clone()),
            control:     crate::control::ControlNamespace::new(inner),
        }
    }
}

/// ClientInner holds the shared state all namespaces need.
/// Wrapped in Arc so it can be shared across namespace structs
/// without lifetime annotations, Arc handles the memory safety.
pub(crate) struct ClientInner {
    pub base_url:   String,
    pub secret_key: String,
    pub http:       HttpClient,
}

impl ClientInner {
    /// Execute an HTTP request with secret key authentication.
    /// This is the core request method, all namespace methods call this.
    ///
    /// Type parameter T is the target type to deserialize the response
    /// data field into. The caller specifies T and serde handles
    /// the deserialization automatically.
    pub(crate) async fn request<B, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        token: Option<&str>,
    ) -> Result<T, PayfakeError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);

        // Use the provided JWT token if given, otherwise use the secret key.
        // Control endpoints use JWT; transaction/charge/customer use secret key.
        let auth_value = token.unwrap_or(&self.secret_key);

        let mut req = self.http
            .request(method, &url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if !auth_value.is_empty() {
            req = req.bearer_auth(auth_value);
        }

        // Attach the body if one was provided.
        // We use serde_json::to_value first to avoid a double-serialization
        // issue, reqwest's .json() method handles the serialization itself.
        if let Some(b) = body {
            req = req.json(b);
        }

        // .send() returns a Result<Response, reqwest::Error>.
        // The ? operator propagates the error, reqwest::Error implements
        // Into<PayfakeError> via the #[from] attribute on the Http variant.
        let resp = req.send().await?;
        let status = resp.status().as_u16();

        // Parse the full envelope, we need it even on failure to extract
        // the error code and message from the body.
        let envelope: ApiResponse = resp.json().await?;

        if envelope.status != "success" {
            return Err(PayfakeError::Api {
                code:        envelope.code,
                message:     envelope.message,
                fields:      envelope.errors,
                http_status: status,
            });
        }

        // Re-serialize the data field and deserialize into T.
        // envelope.data is Option<serde_json::Value>, we need to
        // convert that raw JSON value into the concrete type T.
        // serde_json::from_value does exactly this, it interprets
        // the Value as type T using T's Deserialize implementation.
        let data = envelope.data.unwrap_or(serde_json::Value::Null);
        let result = serde_json::from_value::<T>(data)?;

        Ok(result)
    }
}
