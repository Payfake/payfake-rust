use payfake::{Client, Config, PayfakeError};
use payfake::errors::codes;
use payfake::types::{
    RegisterInput, LoginInput,
    InitializeInput, ChargeCardInput, ChargeMomoInput,
    UpdateScenarioInput, ForceTransactionInput,
    ListOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // STEP 1: Create auth client (no secret key needed for auth endpoints)
    let auth_client = Client::new(Config {
        secret_key: "".to_string(), // Not required for auth
        base_url: Some("http://localhost:8080".to_string()),
        timeout_secs: None,
    });

    // STEP 2: Register or Login to get auth token
    let token = match auth_client.auth.register(RegisterInput {
        business_name: "Acme Store".to_string(),
        email: "dev@acme.com".to_string(),
        password: "secret123".to_string(),
    }).await {
        Ok(resp) => {
            println!("Registered: {}", resp.merchant.id);
            resp.token
        }
        Err(e) if e.is_code(codes::EMAIL_TAKEN) => {
            let resp = auth_client.auth.login(LoginInput {
                email: "dev@acme.com".to_string(),
                password: "secret123".to_string(),
            }).await?;
            println!("Logged in as: {}", resp.merchant.email);
            resp.token
        }
        Err(e) => return Err(e.into()),
    };

    // STEP 3: Get actual API keys using auth token
    let keys = auth_client.auth.get_keys(&token).await?;
    println!("\nAPI Keys retrieved:");
    println!("Public Key: {}", keys.public_key);
    println!("Secret Key: {}...", &keys.secret_key[..15]);

    // STEP 4: Create authenticated client with real secret key
    let client = Client::new(Config {
        secret_key: keys.secret_key,
        base_url: Some("http://localhost:8080".to_string()),
        timeout_secs: None,
    });

    // STEP 5: Initialize a transaction
    let tx = client.transaction.initialize(InitializeInput {
        email: "customer@example.com".to_string(),
        amount: 10000,
        currency: Some("GHS".to_string()),
        ..Default::default()
    }).await?;

    println!("\nTransaction initialized");
    println!("Reference:   {}", tx.reference);
    println!("Access code: {}", tx.access_code);
    println!("Auth URL:    {}", tx.authorization_url);

    // STEP 6: Charge a card
    match client.charge.card(ChargeCardInput {
        access_code: Some(tx.access_code.clone()),
        card_number: "4111111111111111".to_string(),
        card_expiry: "12/26".to_string(),
        cvv: "123".to_string(),
        email: "customer@example.com".to_string(),
        ..Default::default()
    }).await {
        Ok(charge) => {
            println!("\nCharge status: {}", charge.transaction.status);
        }
        Err(e) if e.is_code(codes::CHARGE_FAILED) => {
            println!("\nCharge failed — check scenario config");
            if let Some(field) = e.fields().first() {
                println!("Reason: {}", field.message);
            }
        }
        Err(e) => return Err(e.into()),
    }

    // STEP 7: Verify transaction
    let verified = client.transaction.verify(&tx.reference).await?;
    println!("Verified status: {}", verified.status);

    // STEP 8: Mobile Money flow
    let tx2 = client.transaction.initialize(InitializeInput {
        email: "momo@example.com".to_string(),
        amount: 5000,
        ..Default::default()
    }).await?;

    let momo = client.charge.mobile_money(ChargeMomoInput {
        access_code: Some(tx2.access_code),
        phone: "+233241234567".to_string(),
        provider: "mtn".to_string(),
        email: "momo@example.com".to_string(),
        ..Default::default()
    }).await?;

    println!("\nMoMo status: {}", momo.transaction.status);

    // STEP 9: Control panel operations (using auth token, not secret key)
    let scenario = auth_client.control.update_scenario(&token, UpdateScenarioInput {
        failure_rate: Some(0.3),
        delay_ms: Some(1000),
        ..Default::default()
    }).await?;
    println!("\nScenario updated — failure rate: {}", scenario.failure_rate);

    // Force a specific transaction to fail
    let tx3 = client.transaction.initialize(InitializeInput {
        email: "force@example.com".to_string(),
        amount: 2000,
        ..Default::default()
    }).await?;

    let forced = auth_client.control.force_transaction(
        &token,
        &tx3.reference,
        ForceTransactionInput {
            status: "failed".to_string(),
            error_code: Some("CHARGE_INSUFFICIENT_FUNDS".to_string()),
        },
    ).await?;
    println!("Forced status: {}", forced.status);

    // Reset scenario
    auth_client.control.reset_scenario(&token).await?;
    println!("Scenario reset");

    // STEP 10: Get recent logs
    match auth_client.control.get_logs(&token, ListOptions {
        page: 1,
        per_page: 5,
    }).await {
        Ok(logs) => {
            println!("\nRecent requests: {}", logs.len());
            for log in &logs {
                println!("  {} {} → {}", log.method, log.path, log.status_code);
            }
        }
        Err(e) if e.is_code("LOGS_EMPTY") => {
            println!("\nNo logs found yet (expected for new merchant)");
        }
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
