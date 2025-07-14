// ngrok.rs
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use serde::Deserialize;
use std::path::Path;
use std::env;

#[derive(Deserialize)]
struct TunnelInfo {
    public_url: String,
}

#[derive(Deserialize)]
struct NgrokApiResponse {
    tunnels: Vec<TunnelInfo>,
}

async fn ensure_ngrok_installed() -> Result<(), Box<dyn std::error::Error>> {
    if Command::new("ngrok").arg("--version").output().is_err() {
        println!("[NGROK] Downloading ngrok...");
        
        #[cfg(target_os = "windows")]
        {
            let ngrok_url = "https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-windows-amd64.zip";
            let download_status = Command::new("powershell")
                .args([
                    "-Command",
                    &format!("Invoke-WebRequest -Uri '{}' -OutFile 'ngrok.exe'", ngrok_url)
                ])
                .status()?;
            
            if !download_status.success() {
                return Err("Failed to download ngrok".into());
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            Command::new("curl")
                .args(["-O", "https://bin.equinox.io/c/bNyj1mQVY4c/ngrok-v3-stable-linux-amd64.tgz"])
                .status()?;
            Command::new("tar")
                .args(["xvf", "ngrok-*.tgz"])
                .status()?;
        }

        println!("[NGROK] Downloaded successfully. Please ensure ngrok is in your PATH.");
    }
    Ok(())
}

pub async fn install_ngrok_if_missing() -> Result<(), Box<dyn std::error::Error>> {
    ensure_ngrok_installed().await
}

pub async fn start_ngrok(port: u16) -> Option<String> {
    let ngrok = Command::new("ngrok")
        .args(["http", &port.to_string()])
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    reqwest::get("http://localhost:4040/api/tunnels")
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?
        .get("tunnels")?
        .as_array()?
        .first()?
        .get("public_url")?
        .as_str()
        .map(|s| s.to_string())
}

pub fn stop_ngrok() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "ngrok.exe"])
            .status();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("pkill")
            .arg("ngrok")
            .status();
    }
}