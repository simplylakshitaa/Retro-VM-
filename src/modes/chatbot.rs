// In chatbot.rs (replace the current content with this)
use macroquad::prelude::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

const FONT_SIZE: f32 = 24.0;
const LINE_HEIGHT: f32 = 28.0;

#[derive(Clone)]
struct ChatMessage {
    sender: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct Chatbot {
    messages: Vec<ChatMessage>,
    input: String,
    blink_timer: f32,
    show_cursor: bool,
    booting: bool,
    boot_start: Instant,
    client: Client,
    scroll_offset: usize,
}

impl Chatbot {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            blink_timer: 0.0,
            show_cursor: true,
            booting: true,
            boot_start: Instant::now(),
            client: Client::new(),
            scroll_offset: 0,
        }
    }

    fn today_as_1996(&self) -> String {
        use chrono::Local;
        let now = Local::now();
        now.format("%B %d,").to_string() + " 1996"
    }

    fn get_ollama_response(&self, prompt: &str) -> String {
        if prompt.trim().to_lowercase() == "bye" {
            return "ja le ra hai apun".into();
        }

        let fake_date = self.today_as_1996();
        let retro_prompt = format!(
            "The current date is {}.\nYou're a short, witty, sarcastic assistant stuck in 1996. Your replies must be no longer than 2 short sentences. \
Use 90s slang, attitude, and sass. Act like you're chatting on a green CRT terminal. Never mention anything after 1996.\n\nUser: {}\nAI:",
            fake_date, prompt
        );

        let body = OllamaRequest {
            model: "mistral",
            prompt: &retro_prompt,
            stream: false,
        };

        let res = self.client
            .post("http://localhost:11434/api/generate")
            .json(&body)
            .send();

        match res {
            Ok(r) => {
                let text = r.text().unwrap_or_else(|_| "ERROR: Failed to read text.".into());
                println!("🔍 Raw response from Ollama:\n{text}");

                match serde_json::from_str::<OllamaResponse>(&text) {
                    Ok(parsed) => parsed.response.trim().to_string(),
                    Err(e) => {
                        println!("❌ JSON parse error: {e}");
                        "⚠️ Response parsing failed. Might be invalid JSON.".into()
                    }
                }
            }
            Err(_) => "⚠️ Yo! Can't reach Ollama. Dial-up fried?".into(),
        }
    }

    pub async fn update(&mut self) -> bool {
        let dt = get_frame_time();
        
        if let Some(key) = get_char_pressed() {
            if !key.is_control() {
                self.input.push(key);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.input.pop();
        }

        if is_key_pressed(KeyCode::Enter) && !self.input.trim().is_empty() {
            let prompt = self.input.trim().to_string();

            self.messages.push(ChatMessage {
                sender: "You".into(),
                content: prompt.clone(),
            });

            self.input.clear();

            let reply = self.get_ollama_response(&prompt);
            self.messages.push(ChatMessage {
                sender: "AI_BOT".into(),
                content: reply,
            });

            if prompt.to_lowercase() == "bye" {
                return true;
            }
        }

        if is_key_pressed(KeyCode::PageUp) {
            self.scroll_offset = self.scroll_offset.saturating_sub(5);
        }

        if is_key_pressed(KeyCode::PageDown) {
            self.scroll_offset += 5;
            let max_visible_lines = ((screen_height() - 120.0) / LINE_HEIGHT).floor() as usize;
            if self.scroll_offset + max_visible_lines > self.wrapped_lines().len() {
                self.scroll_offset = self.wrapped_lines().len().saturating_sub(max_visible_lines);
            }
        }

        false
    }

    fn wrapped_lines(&self) -> Vec<String> {
        let max_width = screen_width() - 100.0;
        let mut wrapped_lines: Vec<String> = vec![];

        for msg in &self.messages {
            let full_text = format!("{}: {}", msg.sender, msg.content);
            let mut line = String::new();
            for word in full_text.split_whitespace() {
                let test_line = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", line, word)
                };
                let width = measure_text(&test_line, None, FONT_SIZE as u16, 1.0).width;
                if width < max_width {
                    line = test_line;
                } else {
                    wrapped_lines.push(line);
                    line = word.to_string();
                }
            }
            if !line.is_empty() {
                wrapped_lines.push(line);
            }
            wrapped_lines.push(String::new());
        }

        wrapped_lines
    }

    pub fn draw(&mut self) {
        clear_background(BLACK);
        draw_rectangle_lines(40.0, 40.0, screen_width() - 80.0, screen_height() - 100.0, 2.0, GREEN);

        let chat_top = 60.0;
        let chat_bottom = screen_height() - 60.0;
        let max_visible_lines = ((chat_bottom - chat_top) / LINE_HEIGHT).floor() as usize;

        if self.booting {
            let elapsed = self.boot_start.elapsed();
            if elapsed > Duration::from_secs(4) {
                self.booting = false;
                self.messages.push(ChatMessage {
                    sender: "AI_BOT".into(),
                    content: "Hola Amigos Kaise ho theek Ho".into(),
                });
            } else {
                let y = chat_top;
                let boot_msg = match elapsed.as_secs() {
                    0 => "Booting ChatTerm v1.95...",
                    1 => "Loading memory modules...",
                    2 => "Establishing dial-up connection...",
                    3 => "✅ Connected to AI_BOT via 56K modem",
                    _ => "",
                };
                draw_text(boot_msg, 50.0, y, FONT_SIZE, GREEN);
                return;
            }
        }

        let wrapped_lines = self.wrapped_lines();
        if self.scroll_offset + max_visible_lines > wrapped_lines.len() {
            self.scroll_offset = wrapped_lines.len().saturating_sub(max_visible_lines);
        }

        let mut y = chat_top;
        for line in wrapped_lines.iter().skip(self.scroll_offset).take(max_visible_lines) {
            draw_text(line, 50.0, y, FONT_SIZE, GREEN);
            y += LINE_HEIGHT;
        }

        // Input box
        draw_rectangle(40.0, screen_height() - 50.0, screen_width() - 80.0, 30.0, DARKGRAY);
        draw_text(&format!("> {}", self.input), 50.0, screen_height() - 30.0, FONT_SIZE, WHITE);

        // Cursor blink
        self.blink_timer += get_frame_time();
        if self.blink_timer > 0.5 {
            self.show_cursor = !self.show_cursor;
            self.blink_timer = 0.0;
        }

        if self.show_cursor {
            let cursor_x = 50.0 + measure_text(&format!("> {}", self.input), None, FONT_SIZE as u16, 1.0).width;
            draw_text("_", cursor_x, screen_height() - 30.0, FONT_SIZE, WHITE);
        }
    }
}