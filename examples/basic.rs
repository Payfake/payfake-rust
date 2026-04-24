use payfake::types::*;
use payfake::{codes, Client, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(Config {
        secret_key: "sk_test_your_key_here".to_string(),
        base_url: Some("http://localhost:8080".to_string()),
        timeout: None,
    });

    // Register
    let token = match client
        .auth
        .register(RegisterInput {
            business_name: "Acme Store".to_string(),
            email: "dev@acme.com".to_string(),
            password: "secret123".to_string(),
        })
        .await
    {
        Ok(resp) => {
            println!("Registered: {}", resp.merchant.id);
            resp.access_token
        }
        Err(e) if e.is_code(codes::EMAIL_TAKEN) => {
            println!("Email taken — logging in");
            let resp = client
                .auth
                .login(LoginInput {
                    email: "dev@acme.com".to_string(),
                    password: "secret123".to_string(),
                })
                .await?;
            resp.access_token
        }
        Err(e) => return Err(e.into()),
    };

    //  Get keys
    let keys = client.auth.get_keys(&token).await?;
    println!("Secret key: {}...", &keys.secret_key[..20]);

    let authed = Client::new(Config {
        secret_key: keys.secret_key.clone(),
        base_url: Some("http://localhost:8080".to_string()),
        timeout: None,
    });

    // Initialize
    let tx = authed
        .transaction
        .initialize(InitializeInput {
            email: "customer@example.com".to_string(),
            amount: 10000,
            ..Default::default()
        })
        .await?;
    println!("\nReference:         {}", tx.reference);
    println!("Authorization URL: {}", tx.authorization_url);

    // Full local Verve card flow
    println!("\n── Card flow (local Verve) ──");

    // 5061xxxxxxxxxxxxxxxx = local Ghana Verve card → send_pin
    // 4111xxxxxxxxxxxx     = international Visa       → open_url (3DS)
    let step1 = authed
        .charge
        .card(ChargeCardInput {
            email: "customer@example.com".to_string(),
            reference: Some(tx.reference.clone()),
            card: CardDetails {
                number: "5061000000000000".to_string(),
                cvv: "123".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "2026".to_string(),
            },
            ..Default::default()
        })
        .await?;
    println!("Step 1: {}", step1.status); // send_pin

    let step2 = authed
        .charge
        .submit_pin(SubmitPINInput {
            reference: tx.reference.clone(),
            pin: "1234".to_string(),
        })
        .await?;
    println!("Step 2: {}", step2.status); // send_otp

    // Get OTP from logs, no real phone needed during testing
    let otp_logs = authed
        .control
        .get_otp_logs(&token, Some(&tx.reference), 1, 10)
        .await?;
    let otp = otp_logs
        .first()
        .map(|l| l.otp_code.clone())
        .unwrap_or_default();
    println!("OTP:    {}", otp);

    let step3 = authed
        .charge
        .submit_otp(SubmitOTPInput {
            reference: tx.reference.clone(),
            otp: otp.clone(),
        })
        .await?;
    println!("Step 3: {}", step3.status); // success

    //  Verify
    let verified = authed.transaction.verify(&tx.reference).await?;
    println!("\nVerified:          {}", verified.status);
    println!("Gateway response:  {}", verified.gateway_response);
    if let Some(auth) = &verified.authorization {
        println!("Auth code:         {}", auth.authorization_code);
    }

    //  MoMo flow
    println!("\n── MoMo flow ──");

    let tx2 = authed
        .transaction
        .initialize(InitializeInput {
            email: "momo@example.com".to_string(),
            amount: 5000,
            ..Default::default()
        })
        .await?;

    let momo1 = authed
        .charge
        .mobile_money(ChargeMomoInput {
            email: "momo@example.com".to_string(),
            reference: Some(tx2.reference.clone()),
            mobile_money: MomoDetails {
                phone: "+233241234567".to_string(),
                provider: "mtn".to_string(),
            },
            ..Default::default()
        })
        .await?;
    println!("MoMo step 1: {}", momo1.status); // send_otp

    let momo_logs = authed
        .control
        .get_otp_logs(&token, Some(&tx2.reference), 1, 10)
        .await?;
    let momo2 = authed
        .charge
        .submit_otp(SubmitOTPInput {
            reference: tx2.reference.clone(),
            otp: momo_logs
                .first()
                .map(|l| l.otp_code.clone())
                .unwrap_or_default(),
        })
        .await?;
    println!("MoMo step 2: {}", momo2.status); // pay_offline

    // Poll until resolved
    println!("Polling for resolution...");
    for i in 0..10 {
        let result = authed
            .transaction
            .public_verify(&tx2.reference, &tx2.access_code)
            .await?;
        let flow = result
            .charge
            .as_ref()
            .map(|c| c.flow_status.as_str())
            .unwrap_or("–");
        println!("  poll {}: status={} flow={}", i + 1, result.status, flow);
        if result.status == "success" || result.status == "failed" {
            println!("Resolved: {}", result.status);
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Scenario testing
    println!("\n── Scenario testing ──");

    authed
        .control
        .update_scenario(
            &token,
            UpdateScenarioInput {
                force_status: Some("failed".to_string()),
                error_code: Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
                ..Default::default()
            },
        )
        .await?;
    println!("Scenario: force insufficient funds");

    let tx3 = authed
        .transaction
        .initialize(InitializeInput {
            email: "fail@example.com".to_string(),
            amount: 10000,
            ..Default::default()
        })
        .await?;

    match authed
        .charge
        .card(ChargeCardInput {
            email: "fail@example.com".to_string(),
            reference: Some(tx3.reference.clone()),
            card: CardDetails {
                number: "5061000000000000".to_string(),
                cvv: "123".to_string(),
                expiry_month: "12".to_string(),
                expiry_year: "2026".to_string(),
            },
            ..Default::default()
        })
        .await
    {
        Err(e) => {
            println!("Charge failed as expected: {:?}", e.code());
            if e.is_code(codes::INSUFFICIENT_FUNDS) {
                println!("Correctly identified as insufficient funds");
            }
        }
        Ok(_) => println!("Expected failure but got success"),
    }

    authed.control.reset_scenario(&token).await?;
    println!("Scenario reset");

    //  Stats
    let stats = authed.control.get_stats(&token).await?;
    let total = stats["transactions"]["total"].as_i64().unwrap_or(0);
    let rate = stats["transactions"]["success_rate"]
        .as_f64()
        .unwrap_or(0.0);
    println!("\nStats: total={} success_rate={:.1}%", total, rate);

    Ok(())
}
