use macroquad::prelude::*;
use std::fs::{File, remove_file};
use rfd::FileDialog;
use chrono::Utc;

pub struct Notepad {
    text: String,
    status: String,
    cursor_position: usize,
    save_button_rect: Rect,
    open_button_rect: Rect,
}

impl Notepad {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            status: "Welcome to Retro Notepad!".to_string(),
            cursor_position: 0,
            save_button_rect: Rect::new(20.0, 440.0, 100.0, 30.0),
            open_button_rect: Rect::new(130.0, 440.0, 100.0, 30.0),
        }
    }

    pub fn update(&mut self) -> bool {
        // Handle text input
        while let Some(key) = get_char_pressed() {
            if key as u32 == 8 {
                if self.cursor_position > 0 {
                    self.text.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            } else if key.is_ascii() && !key.is_control() {
                self.text.insert(self.cursor_position, key);
                self.cursor_position += 1;
            }
        }

        // Handle cursor movement
        if is_key_pressed(KeyCode::Left) && self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
        if is_key_pressed(KeyCode::Right) && self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }

        // Handle buttons
        let mouse_pos = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) {
            if self.save_button_rect.contains(vec2(mouse_pos.0, mouse_pos.1)) {
                self.save_encrypted();
            } else if self.open_button_rect.contains(vec2(mouse_pos.0, mouse_pos.1)) {
                self.open_file();
            }
        }

        // Exit on ESC
        is_key_pressed(KeyCode::Escape)
    }

    pub fn draw(&mut self) {
        clear_background(BLACK);
        
        // Draw UI elements
        draw_text("✎ RETRO NOTEPAD", 20.0, 40.0, 24.0, GREEN);
        draw_rectangle(20.0, 70.0, 560.0, 350.0, Color::new(0.1, 0.1, 0.1, 1.0));
        draw_text(&self.text, 25.0, 90.0, 16.0, GREEN);

        // Draw cursor
        if (get_time() * 2.0).sin() > 0.0 {
            let cursor_x = 25.0 + measure_text(&self.text[..self.cursor_position], None, 16, 1.0).width;
            draw_rectangle(cursor_x, 75.0, 2.0, 16.0, GREEN);
        }

        // Draw buttons
        self.draw_button(&self.save_button_rect, "💾 Save");
        self.draw_button(&self.open_button_rect, "📂 Open");
        
        // Status
        draw_text(&self.status, 20.0, 490.0, 16.0, GREEN);
    }

    fn draw_button(&self, rect: &Rect, text: &str) {
        let hovered = rect.contains(mouse_position().into());
        draw_rectangle(rect.x, rect.y, rect.w, rect.h,
            if hovered { DARKGRAY } else { Color::new(0.2, 0.2, 0.2, 1.0) });
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, GREEN);
        draw_text(text, rect.x + 15.0, rect.y + 20.0, 16.0, GREEN);
    }

    fn xor_encrypt(&self, data: &str, key: &str) -> Vec<u8> {
        data.bytes()
            .zip(key.bytes().cycle())
            .map(|(b, k)| b ^ k)
            .collect()
    }

    fn xor_decrypt(&self, data: &[u8], key: &str) -> String {
        data.iter()
            .zip(key.bytes().cycle())
            .map(|(&b, k)| (b ^ k) as char)
            .collect()
    }

    fn save_encrypted(&mut self) {
        let _ = std::fs::create_dir_all("secrets");
        let filename = format!("secrets/secret_{}.enc", Utc::now().timestamp());
        let encrypted = self.xor_encrypt(&self.text, "my_secret_key_123");
        
        if let Err(e) = std::fs::write(&filename, encrypted) {
            self.status = format!("Failed to save: {}", e);
        } else {
            self.status = format!("Saved to {}", filename);
        }
    }

    fn open_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Encrypted", &["enc"])
            .pick_file() 
        {
            if let Ok(data) = std::fs::read(&path) {
                self.text = self.xor_decrypt(&data, "my_secret_key_123");
                self.status = format!("Loaded from {}", path.display());
                self.cursor_position = self.text.len();
            } else {
                self.status = "Failed to open file".to_string();
            }
        }
    }
}