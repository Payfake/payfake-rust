# payfake

Official Rust SDK for [Payfake](https://payfake.co) — a Paystack-compatible payment
simulator for African developers.

```toml
[dependencies]
payfake = "1.0"
tokio = { version = "1", features = ["full"] }
```

---

## Quick Start

```rust
use payfake::{Client, Config};

let client = Client::new(Config {
    secret_key: "sk_test_xxx".to_string(),
    base_url:   None,   // defaults to https://api.payfake.co
    timeout:    None,
});

// Self-hosted:
let client = Client::new(Config {
    secret_key: "sk_test_xxx".to_string(),
    base_url:   Some("http://localhost:8080".to_string()),
    timeout:    None,
});
```

Change one config value to switch to real Paystack:

```rust
// Development
let client = Client::new(Config {
    secret_key: std::env::var("PAYSTACK_SECRET_KEY").unwrap(),
    base_url:   Some(std::env::var("PAYSTACK_BASE_URL").unwrap()),
    timeout:    None,
});
```

---

## Full Card Flow

```rust
use payfake::types::*;

// Initialize
let tx = client.transaction.initialize(InitializeInput {
    email:  "customer@example.com".to_string(),
    amount: 10000,  // GHS 100.00 — amounts in pesewas
    ..Default::default()
}).await?;

// Charge — local Verve card (5061xxxxxxxxxxxxxxxxx)
let step1 = client.charge.card(ChargeCardInput {
    email:       "customer@example.com".to_string(),
    access_code: Some(tx.access_code.clone()),
    card: CardDetails {
        number:       "5061000000000000".to_string(),
        cvv:          "123".to_string(),
        expiry_month: "12".to_string(),
        expiry_year:  "2026".to_string(),
    },
    ..Default::default()
}).await?;
// step1.status == "send_pin"

// Submit PIN
let step2 = client.charge.submit_pin(SubmitPINInput {
    reference: tx.reference.clone(),
    pin:       "1234".to_string(),
}).await?;
// step2.status == "send_otp"

// Get OTP from logs (no real phone needed)
let logs = client.control.get_otp_logs(&token, Some(&tx.reference), 1, 10).await?;
let otp  = logs[0].otp_code.clone();

// Submit OTP
let step3 = client.charge.submit_otp(SubmitOTPInput {
    reference: tx.reference.clone(),
    otp,
}).await?;
// step3.status == "success"

// Verify
let verified = client.transaction.verify(&tx.reference).await?;
// verified.status == "success"
// verified.gateway_response == "Approved"
// verified.authorization.unwrap().authorization_code
```

---

## Charge Flow Status Reference

| Status | Meaning | Next Call |
|--------|---------|-----------|
| `send_pin` | Enter card PIN | `charge.submit_pin` |
| `send_otp` | Enter OTP | `charge.submit_otp` |
| `send_birthday` | Enter date of birth | `charge.submit_birthday` |
| `send_address` | Enter billing address | `charge.submit_address` |
| `open_url` | Complete 3DS — open `url` field | Navigate checkout to `url` |
| `pay_offline` | Approve USSD prompt | Poll `transaction.public_verify` |
| `success` | Payment complete | Call `transaction.verify` |
| `failed` | Payment declined | Read `gateway_response` |

---

## Mobile Money

```rust
let step1 = client.charge.mobile_money(ChargeMomoInput {
    email:       "customer@example.com".to_string(),
    access_code: Some(tx.access_code.clone()),
    mobile_money: MomoDetails {
        phone:    "+233241234567".to_string(),
        provider: "mtn".to_string(),  // mtn | vodafone | airteltigo
    },
    ..Default::default()
}).await?;
// step1.status == "send_otp"

let logs  = client.control.get_otp_logs(&token, Some(&tx.reference), 1, 10).await?;
let step2 = client.charge.submit_otp(SubmitOTPInput {
    reference: tx.reference.clone(),
    otp:       logs[0].otp_code.clone(),
}).await?;
// step2.status == "pay_offline"

// Poll every 3 seconds
loop {
    let result = client.transaction.public_verify(&tx.reference).await?;
    if result.status == "success" || result.status == "failed" { break; }
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}
```

---

## Scenario Testing

```rust
use payfake::types::UpdateScenarioInput;

// Login to get JWT
let resp  = client.auth.login(LoginInput { email: "...", password: "..." }).await?;
let token = resp.access_token;

// Force failure
client.control.update_scenario(&token, UpdateScenarioInput {
    force_status: Some("failed".to_string()),
    error_code:   Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
    ..Default::default()
}).await?;

// Reset
client.control.reset_scenario(&token).await?;
```

---

## Error Handling

```rust
use payfake::{PayfakeError, codes};

match client.charge.submit_otp(input).await {
    Err(e) if e.is_code(codes::CHARGE_INVALID_OTP) => {
        // OTP expired — resend
        client.charge.resend_otp(ResendOTPInput {
            reference: reference.to_string(),
        }).await?;
    }
    Err(PayfakeError::Api { code, message, fields, http_status }) => {
        eprintln!("code={} message={} status={}", code, message, http_status);
        for f in &fields {
            eprintln!("  {}: {} ({})", f.field, f.message, f.rule);
        }
    }
    Err(e) => return Err(e.into()),
    Ok(resp) => println!("status: {}", resp.status),
}
```

---

## Webhook Verification

```rust
use hmac::{Hmac, Mac};
use sha2::Sha512;
use hex;

fn verify_webhook(body: &[u8], signature: &str, secret_key: &str) -> bool {
    let mut mac = Hmac::<Sha512>::new_from_slice(secret_key.as_bytes()).unwrap();
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}
```

Add to `Cargo.toml`:
```toml
hmac = "0.12"
sha2 = "0.10"
hex  = "0.4"
```

---

## License

MIT
