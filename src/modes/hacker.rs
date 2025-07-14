use macroquad::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use ::rand::Rng;
use std::process::Command;
use tokio::runtime::Runtime;
use crate::modes::hackersmod::{ngrok, server, sitegen, webhook};
use std::collections::HashMap;
use ::rand::thread_rng;
static START_TIME: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

pub struct PhishingState {
    pub active: bool,
    pub brand: String,
    pub webhook_url: String,
    pub ngrok_url: Arc<Mutex<Option<String>>>,
}

pub struct HackerMode {
    pub triggered: bool,
    terminal_lines: Vec<String>,
    current_line_index: usize,
    char_index: usize,
    input_buffer: String,
    last_typing_time: Instant,
    blink_on: bool,
    phishing: PhishingState,
    scroll_offset: f32,
    max_scroll: f32,
}

impl Default for HackerMode {
    fn default() -> Self {
        Self {
            triggered: false,
            terminal_lines: vec![],
            current_line_index: 0,
            char_index: 0,
            input_buffer: String::new(),
            last_typing_time: Instant::now(),
            blink_on: true,
            phishing: PhishingState {
                active: false,
                brand: String::new(),
                webhook_url: String::new(),
                ngrok_url: Arc::new(Mutex::new(None)),
            },
            scroll_offset: 0.0,
            max_scroll: 0.0,
        }
    }
}

impl HackerMode {
    pub fn update(&mut self) {
        if !self.triggered && is_key_pressed(KeyCode::G)
            && is_key_down(KeyCode::LeftControl)
            && is_key_down(KeyCode::LeftAlt) {
            self.trigger();
        }
    }

    pub fn trigger(&mut self) {
        self.triggered = true;
        self.add_terminal_line("=== SECURITY TRAINING TERMINAL ===");
        self.add_terminal_line("WARNING: For authorized training only");
        self.add_terminal_line("----------------------------------");
        self.add_terminal_line("Loading modules:");
        self.add_terminal_line("- Security Awareness Trainer");
        self.add_terminal_line("- Phishing Simulation Kit");
        self.add_terminal_line("- Network Defense Tools");
        self.add_terminal_line("----------------------------------");
        self.add_terminal_line("System ready. Type 'help' for commands");
        *START_TIME.lock().unwrap() = Some(Instant::now());
        self.last_typing_time = Instant::now();
    }

    pub fn draw_hacker_ui(&mut self) {
        clear_background(BLACK);

        if !self.triggered {
            return;
        }

        // Calculate max scroll based on content height
        let content_height = self.terminal_lines.len() as f32 * 24.0;
        let visible_height = screen_height() - 100.0;
        self.max_scroll = (content_height - visible_height).max(0.0);

        // Handle scroll input
        if mouse_wheel().1 != 0.0 {
            self.scroll_offset = (self.scroll_offset - mouse_wheel().1 * 20.0)
                .clamp(0.0, self.max_scroll);
        }

        // Draw scroll bar if needed
        if self.max_scroll > 0.0 {
            let scroll_bar_width = 10.0;
            let scroll_bar_height = visible_height * (visible_height / content_height);
            let scroll_pos = (self.scroll_offset / self.max_scroll) * (visible_height - scroll_bar_height);
            
            draw_rectangle(
                screen_width() - scroll_bar_width - 5.0,
                5.0 + scroll_pos,
                scroll_bar_width,
                scroll_bar_height,
                Color::new(0.2, 0.8, 0.2, 0.5),
            );
        }

        // Render terminal text with scroll offset
        let mut y = 20.0 - self.scroll_offset;
        for (i, line) in self.terminal_lines.iter().enumerate() {
            if i <= self.current_line_index && y + 24.0 > 0.0 && y < screen_height() {
                let render_line = if i == self.current_line_index {
                    line.chars().take(self.char_index).collect::<String>()
                } else {
                    line.clone()
                };

                draw_text_ex(
                    &render_line,
                    20.0,
                    y,
                    TextParams {
                        font: None,
                        font_size: 20,
                        color: GREEN,
                        ..Default::default()
                    },
                );
            }
            y += 24.0;
        }

        // Update typing animation
        if self.last_typing_time.elapsed() >= Duration::from_millis(10) {
            if self.current_line_index < self.terminal_lines.len() {
                self.char_index += 1;
                if self.char_index > self.terminal_lines[self.current_line_index].len() {
                    self.current_line_index += 1;
                    self.char_index = 0;
                    if ::rand::thread_rng().gen_range(0..10) == 0 {
                        self.add_terminal_line("@##$%&*^GL!TCH~ERROR###");
                    }
                }
            }
            self.last_typing_time = Instant::now();
        }

        // Draw input prompt
        let input_y = screen_height() - 50.0;
        draw_line(20.0, input_y - 10.0, screen_width() - 20.0, input_y - 10.0, 1.0, GREEN);
        
        if self.last_typing_time.elapsed() >= Duration::from_millis(500) {
            self.blink_on = !self.blink_on;
            self.last_typing_time = Instant::now();
        }

        let cursor = if self.blink_on { "█" } else { " " };
        let prompt = format!("$ {}{}", self.input_buffer, cursor);
        draw_text_ex(
            &prompt,
            20.0,
            input_y,
            TextParams {
                font: None,
                font_size: 20,
                color: GREEN,
                ..Default::default()
            },
        );

        self.handle_input();
    }

    fn handle_input(&mut self) {
        while let Some(key) = get_char_pressed() {
            self.input_buffer.push(key);
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.input_buffer.pop();
        }

        if is_key_pressed(KeyCode::Enter) {
            let input = self.input_buffer.trim().to_lowercase();
            self.process_command(&input);
            self.input_buffer.clear();
            self.scroll_offset = self.max_scroll;
        }
    }

    fn process_command(&mut self, input: &str) {
        self.add_terminal_line(&format!("$ {}", input));
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.as_slice() {
            ["help"] => self.show_help(),
            ["clear"] => self.clear_terminal(),
            ["exit"] => self.exit_hacker_mode(),
            ["phish", "start", brand, business, ..] => {
                let logo = parts.get(3);
                let webhook = parts.get(4);
                self.start_phishing(brand, business, logo, webhook);
            },
            ["phish", "stop"] => self.stop_phishing(),
            ["phish", "status"] => self.phishing_status(),
            ["check_browser", target] => self.scan_browser_extension(target),
            ["weak_ssids", ssid] => self.check_ssid_strength(ssid),
            ["footprint"] => self.run_footprint_scan(),
            _ => self.add_terminal_line("Unknown command. Type 'help' for options"),
        }
    }

    fn show_help(&mut self) {
        self.add_terminal_line("Security Training Commands:");
        self.add_terminal_line("  phish start <brand> <business> [logo] [webhook] - Start simulation");
        self.add_terminal_line("  phish stop               - Stop current simulation");
        self.add_terminal_line("  phish status             - Show simulation status");
        self.add_terminal_line("  check_browser <file>     - Analyze browser extension");
        self.add_terminal_line("  weak_ssids <ssid>        - Check SSID vulnerability");
        self.add_terminal_line("  footprint                - Run digital footprint scan");
        self.add_terminal_line("  clear                    - Clear terminal");
        self.add_terminal_line("  exit                     - Exit security terminal");
    }

    fn clear_terminal(&mut self) {
        self.terminal_lines.clear();
        self.current_line_index = 0;
        self.char_index = 0;
        self.scroll_offset = 0.0;
        self.add_terminal_line("Terminal cleared");
    }

    fn exit_hacker_mode(&mut self) {
        if self.phishing.active {
            self.stop_phishing();
        }
        self.triggered = false;
        self.add_terminal_line("Security terminal session ended");
    }
    fn start_phishing(&mut self, brand: &str, business: &str, logo: Option<&&str>, webhook: Option<&&str>) {
        const PORT: u16 = 8081;
        
        self.add_terminal_line(&format!("🚀 Starting SECURITY TRAINING simulation for {}...", brand));
        self.add_terminal_line("⚠️ WARNING: For authorized training only");
        self.add_terminal_line("⚠️ Legal and ethical use required");

        if brand.trim().is_empty() || business.trim().is_empty() {
            self.add_terminal_line("❌ Error: Brand and business name cannot be empty");
            return;
        }

        self.phishing.brand = brand.to_string();
        self.phishing.webhook_url = webhook.unwrap_or(&"").to_string();
        self.phishing.active = true;

        let html = sitegen::generate_html(
            brand, 
            business, 
            logo.unwrap_or(&""), 
            "default"
        );
        
        let webhook_url = self.phishing.webhook_url.clone();
        let ngrok_url = Arc::clone(&self.phishing.ngrok_url);

        // Spawn a new thread for the server operations
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            
            // Clone the Arc for the ngrok task
            let ngrok_url_for_task = Arc::clone(&ngrok_url);
            
            rt.block_on(async {
                // Convert the error to a thread-safe type
                let server_result = async {
                    server::start_server(html, PORT, webhook_url)
                        .await
                        .map_err(|e| e.to_string())
                };

                // Start server
                let server = tokio::spawn(server_result);
                
                // Start ngrok after a short delay
                let ngrok = tokio::spawn(async move {  // Note: 'move' here
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if let Some(url) = ngrok::start_ngrok(PORT).await {
                        let mut lock = ngrok_url_for_task.lock().unwrap();
                        *lock = Some(url);
                    }
                });

                // Run both tasks concurrently
                match tokio::try_join!(server, ngrok) {
                    Ok((server_res, _)) => {
                        if let Err(e) = server_res {
                            eprintln!("Server error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Error in phishing simulation: {}", e),
                }
            });
        });

        self.add_terminal_line(&format!("✅ Training page available at http://localhost:{}", PORT));
    }
    fn stop_phishing(&mut self) {
        if !self.phishing.active {
            self.add_terminal_line("ℹ️ No active security training to stop");
            return;
        }

        self.phishing.active = false;
        ngrok::stop_ngrok();
        self.phishing.ngrok_url = Arc::new(Mutex::new(None));
        self.add_terminal_line("🛑 Security training stopped");
        self.add_terminal_line("ℹ️ All resources cleaned up");
    }

    fn phishing_status(&mut self) {
        self.add_terminal_line("=== SECURITY TRAINING STATUS ===");
        self.add_terminal_line(&format!("Active: {}", self.phishing.active));

        if self.phishing.active {
            self.add_terminal_line(&format!("Brand: {}", self.phishing.brand));

            // Scope to drop the immutable borrow before mutably borrowing `self` again
            let public_url = {
                if let Ok(lock) = self.phishing.ngrok_url.lock() {
                    lock.clone() // Clone the Option<String>
                } else {
                    None
                }
            };

            if let Some(url) = public_url {
                self.add_terminal_line(&format!("Public URL: {}", url));
            } else {
                self.add_terminal_line("Public URL: Not available (localhost only)");
            }

            if !self.phishing.webhook_url.is_empty() {
                self.add_terminal_line("Webhook: Configured");
            } else {
                self.add_terminal_line("Webhook: Not configured");
            }
        }
    }


    fn scan_browser_extension(&mut self, target: &str) {
        self.add_terminal_line(&format!("🔍 Analyzing browser extension: {}", target));
        
        let output = Command::new("python")
            .arg("src/scripts/predict_extension.py")
            .arg(target)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    self.add_terminal_line(line);
                }
            }
            Err(e) => {
                self.add_terminal_line(&format!("❌ Error: {}", e));
            }
        }
    }

    fn check_ssid_strength(&mut self, ssid: &str) {
        self.add_terminal_line(&format!("📶 Checking SSID strength: {}", ssid));
        
        let output = Command::new("python")
            .arg("src/scripts/predict_ssid.py")
            .arg(ssid)
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.add_terminal_line(&stdout.trim());
            }
            Err(e) => {
                self.add_terminal_line(&format!("❌ Error: {}", e));
            }
        }
    }

    fn run_footprint_scan(&mut self) {
        self.add_terminal_line("🔍 Starting Digital Footprint Scan...");
        
        // Simulated data - in a real implementation, this would call actual APIs
        let email = "user@example.com";
        self.add_terminal_line("\n🛡️ Breach Lookup");
        self.add_terminal_line(&format!("Found 2 breaches for {}", email));
        self.add_terminal_line("- AdobeLeak (2019): Email, Password");
        self.add_terminal_line("- DataMonster (2021): Email, Name, Phone");

        self.add_terminal_line("\n🌐 Social Media Presence");
        let platforms = vec!["GitHub", "Twitter", "Reddit", "Instagram"];
        for site in platforms {
            self.add_terminal_line(&format!(
                "- {}: https://{}.com/username",
                site,
                site.to_lowercase()
            ));
        }

        self.add_terminal_line("\n📊 Final Risk Score: 83/100");
        self.add_terminal_line("ℹ️ This is simulated data for training purposes");
    }

    fn add_terminal_line(&mut self, text: &str) {
        self.terminal_lines.push(text.to_string());
    }
}