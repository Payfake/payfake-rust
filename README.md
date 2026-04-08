# payfake-rust

Official Rust SDK for [Payfake API](https://github.com/payfake/payfake-api), a self-hostable African payment simulator that mirrors the Paystack API exactly. Test every payment scenario without touching real money.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
payfake = { git = "https://github.com/payfake/payfake-rust" }
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use payfake::{Client, Config};
use payfake::types::{InitializeInput, ChargeCardInput};
use payfake::errors::codes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Config {
        secret_key: "sk_test_xxx".to_string(),
        base_url: None,       // defaults to http://localhost:8080
        timeout_secs: None,   // defaults to 30 seconds
    });

    // Initialize a transaction
    let tx = client.transaction.initialize(InitializeInput {
        email: "customer@example.com".to_string(),
        amount: 10000, // GHS 100.00 — amounts in smallest unit (pesewas)
        currency: Some("GHS".to_string()),
        ..Default::default()
    }).await?;

    println!("Access code: {}", tx.access_code);
    println!("Auth URL: {}", tx.authorization_url);

    // Charge a card
    let charge = client.charge.card(ChargeCardInput {
        access_code: Some(tx.access_code),
        card_number: "4111111111111111".to_string(),
        card_expiry: "12/26".to_string(),
        cvv: "123".to_string(),
        email: "customer@example.com".to_string(),
        ..Default::default()
    }).await?;

    println!("Status: {}", charge.transaction.status);

    // Verify the transaction
    let verified = client.transaction.verify(&tx.reference).await?;
    println!("Verified: {}", verified.status);

    Ok(())
}
```

## Namespaces

| Namespace | Access | Description |
|-----------|--------|-------------|
| `client.auth` | Public + JWT | Register, login, key management |
| `client.transaction` | Secret key | Initialize, verify, list, refund |
| `client.charge` | Secret key | Card, mobile money, bank transfer |
| `client.customer` | Secret key | Create, list, fetch, update |
| `client.control` | JWT | Scenarios, webhooks, logs, force outcomes |

## Error Handling

Every failed API call returns `Err(PayfakeError)`. Use pattern matching
for programmatic handling, idiomatic Rust, no exceptions:

```rust
use payfake::errors::codes;

match client.transaction.initialize(input).await {
    Ok(tx) => println!("Initialized: {}", tx.reference),
    Err(e) if e.is_code(codes::REFERENCE_TAKEN) => {
        // duplicate reference, verify the existing transaction
        let tx = client.transaction.verify(&reference).await?;
    }
    Err(e) if e.is_code(codes::INVALID_AMOUNT) => {
        eprintln!("Amount must be greater than zero");
    }
    Err(e) => return Err(e.into()),
}
```

`PayfakeError` variants:

```rust
PayfakeError::Api {
    code,        // Payfake response code, stable, use for matching
    message,     // Human-readable, don't parse programmatically
    fields,      // Vec<ApiErrorField>, field-level validation errors
    http_status, // u16 HTTP status code
}
PayfakeError::Http(reqwest::Error)   // network error, timeout, DNS failure
PayfakeError::Parse(serde_json::Error) // unexpected response format
PayfakeError::InvalidInput(String)   // missing required SDK input
```

## Scenario Control

```rust
// Login first to get a JWT
let resp = client.auth.login(LoginInput {
    email: "dev@acme.com".to_string(),
    password: "secret123".to_string(),
}).await?;
let token = resp.token;

// 30% failure rate with 1 second delay
client.control.update_scenario(&token, UpdateScenarioInput {
    failure_rate: Some(0.3),
    delay_ms: Some(1000),
    ..Default::default()
}).await?;

// Force a specific transaction to fail
client.control.force_transaction(&token, &reference, ForceTransactionInput {
    status: "failed".to_string(),
    error_code: Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
}).await?;

// Reset everything back to defaults
client.control.reset_scenario(&token).await?;
```

## Mobile Money

MoMo charges are async, always return `pending` immediately.
The final outcome arrives via webhook after the simulated delay:

```rust
let charge = client.charge.mobile_money(ChargeMomoInput {
    access_code: Some(tx.access_code),
    phone: "+233241234567".to_string(),
    provider: "mtn".to_string(), // mtn | vodafone | airteltigo
    email: "customer@example.com".to_string(),
    ..Default::default()
}).await?;

// charge.transaction.status is always "pending" here
// implement a webhook handler for the final outcome
```

## Partial Updates

Rust's `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`
implements the partial-update pattern cleanly — fields you don't set
are absent from the JSON body and the API leaves them unchanged:

```rust
// Only update failure_rate, delay_ms and force_status stay as they are
client.control.update_scenario(&token, UpdateScenarioInput {
    failure_rate: Some(0.5),
    ..Default::default() // all other fields are None → not sent
}).await?;
```

## Requirements

- Rust 1.75+
- tokio async runtime
- A running [Payfake API](https://github.com/payfake/payfake-api) server

## License

MIT
