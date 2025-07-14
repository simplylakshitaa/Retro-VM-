use macroquad::prelude::*;
use macroquad::audio::*;
use crate::modes::floppy_disk::FloppyDiskGame;
use crate::modes::chess_GAME::{ChessGame, GameStatus};
use crate::modes::math_question::MathQuestion;
use crate::modes::notepad::Notepad;
use crate::modes::chatbot::Chatbot;
use crate::modes::hacker::HackerMode;
use std::fs::{File, remove_file};
use std::process::Command;
use chrono::Utc;

pub enum AppState {
    Booting,
    Desktop,
    Chatbot(Chatbot),
    FloppyDisk(FloppyDiskGame),
    Chess(ChessGame),
    MathQuestion(MathQuestion),
    Vedic(MathQuestion),
    HackerMode(HackerMode, MathQuestion),
    Notepad(Notepad),
    PasswordScreen,
    WelcomeScreen,
    VirusMode,
}

#[derive(Clone)]
struct AppIcon {
    rect: Rect,
    label: String,
    texture: Texture2D,
    primary_color: Color,
    hover_color: Color,
    accent_color: Color,
    app_type: AppType,
}

#[derive(Clone)]
enum AppType {
    Chatbot,
    FloppyDisk,
    Chess,
    Vedic,
    Notepad,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    color: Color,
    size: f32,
}

pub struct NormalMode {
    pub state: AppState,
    boot_start: f64,
    anime_effect_timer: f32,
    icons: Vec<AppIcon>,
    password_input: String,
    virus_text: String,
    welcome_timer: f32,
    esc_pressed: bool,
    desktop_particles: Vec<Particle>,
    time_display: String,
    taskbar_height: f32,
    grid_animation: f32,
    background_texture: Texture2D,
}

impl NormalMode {
    pub async fn new() -> Self {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let taskbar_height = 60.0;
        
        // Load textures
        let background_texture = load_texture("assets/background.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load background texture"));
        
        // Load icon textures
        let chatbot_icon = load_texture("assets/chatbot_icon.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load chatbot icon"));
        let floppy_icon = load_texture("assets/floppy_icon.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load floppy icon"));
        let chess_icon = load_texture("assets/chess_icon.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load chess icon"));
        let vedic_icon = load_texture("assets/vedic_icon.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load vedic icon"));
        let notepad_icon = load_texture("assets/notepad_icon.png")
            .await
            .unwrap_or_else(|_| panic!("Failed to load notepad icon"));

        // Icon layout configuration
        let icon_size = 100.0;
        let icon_spacing = 40.0;
        let cols = 3;
        let rows = 2;

        // Calculate grid dimensions
        let grid_width = cols as f32 * icon_size + (cols as f32 - 1.0) * icon_spacing;
        let grid_height = rows as f32 * icon_size + (rows as f32 - 1.0) * icon_spacing;
        let grid_start_x = (screen_w - grid_width) / 2.0;
        let grid_start_y = (screen_h - grid_height) / 2.0;

        let icons = vec![
            AppIcon {
                rect: Rect::new(grid_start_x, grid_start_y, icon_size, icon_size),
                label: "CHATBOT".to_string(),
                texture: chatbot_icon,
                primary_color: Color::from_rgba(45, 140, 255, 255),
                hover_color: Color::from_rgba(65, 160, 255, 255),
                accent_color: Color::from_rgba(25, 120, 235, 255),
                app_type: AppType::Chatbot,
            },
            AppIcon {
                rect: Rect::new(grid_start_x + icon_size + icon_spacing, grid_start_y, icon_size, icon_size),
                label: "FLOPPY DISK".to_string(),
                texture: floppy_icon,
                primary_color: Color::from_rgba(156, 39, 176, 255),
                hover_color: Color::from_rgba(176, 59, 196, 255),
                accent_color: Color::from_rgba(136, 19, 156, 255),
                app_type: AppType::FloppyDisk,
            },
            AppIcon {
                rect: Rect::new(grid_start_x + 2.0 * (icon_size + icon_spacing), grid_start_y, icon_size, icon_size),
                label: "CHESS".to_string(),
                texture: chess_icon,
                primary_color: Color::from_rgba(76, 175, 80, 255),
                hover_color: Color::from_rgba(96, 195, 100, 255),
                accent_color: Color::from_rgba(56, 155, 60, 255),
                app_type: AppType::Chess,
            },
            AppIcon {
                rect: Rect::new(grid_start_x, grid_start_y + icon_size + icon_spacing, icon_size, icon_size),
                label: "VEDIC MATH".to_string(),
                texture: vedic_icon,
                primary_color: Color::from_rgba(255, 152, 0, 255),
                hover_color: Color::from_rgba(255, 172, 20, 255),
                accent_color: Color::from_rgba(235, 132, 0, 255),
                app_type: AppType::Vedic,
            },
            AppIcon {
                rect: Rect::new(grid_start_x + icon_size + icon_spacing, grid_start_y + icon_size + icon_spacing, icon_size, icon_size),
                label: "NOTEPAD".to_string(),
                texture: notepad_icon,
                primary_color: Color::from_rgba(96, 125, 139, 255),
                hover_color: Color::from_rgba(116, 145, 159, 255),
                accent_color: Color::from_rgba(76, 105, 119, 255),
                app_type: AppType::Notepad,
            },
        ];

        Self {
            state: AppState::Booting,
            boot_start: get_time(),
            anime_effect_timer: 0.0,
            icons,
            password_input: String::new(),
            virus_text: String::new(),
            welcome_timer: 0.0,
            esc_pressed: false,
            desktop_particles: Vec::new(),
            time_display: String::new(),
            taskbar_height,
            grid_animation: 0.0,
            background_texture,
        }
    }

    pub async fn update(&mut self) -> bool {
        let dt = get_frame_time();
        self.anime_effect_timer += dt;
        self.grid_animation += dt * 2.0;

        // Update particles
        self.desktop_particles.retain_mut(|p| {
            p.pos += p.vel * dt;
            p.life -= dt;
            p.color.a = (p.life / 2.0).min(1.0);
            p.life > 0.0
        });

        // Add new particles occasionally
        if self.desktop_particles.len() < 50 && rand::gen_range(0.0, 1.0) < 0.3 * dt {
            self.desktop_particles.push(Particle {
                pos: vec2(rand::gen_range(0.0, screen_width()), screen_height()),
                vel: vec2(rand::gen_range(-20.0, 20.0), rand::gen_range(-50.0, -10.0)),
                life: rand::gen_range(1.0, 3.0),
                color: Color::from_rgba(100, 200, 255, 50),
                size: rand::gen_range(1.0, 3.0),
            });
        }

        // Update time display
        self.time_display = format!("{}", Utc::now().format("%H:%M:%S"));

        self.esc_pressed = is_key_pressed(KeyCode::Escape);

        let mut next_state = None;

        match std::mem::replace(&mut self.state, AppState::Booting) {
            AppState::Booting => {
                if get_time() - self.boot_start > 3.0 {
                    next_state = Some(AppState::PasswordScreen);
                } else {
                    next_state = Some(AppState::Booting);
                }
            }

            AppState::PasswordScreen => {
                while let Some(c) = get_char_pressed() {
                    if c == '\u{8}' && !self.password_input.is_empty() {
                        self.password_input.pop();
                    } else if c.is_ascii() && !c.is_control() {
                        self.password_input.push(c);
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    if self.password_input.trim() == "hola amigo!" {
                        next_state = Some(AppState::WelcomeScreen);
                    } else {
                        self.activate_virus().await;
                        next_state = Some(AppState::VirusMode);
                    }
                } else {
                    next_state = Some(AppState::PasswordScreen);
                }
            }

            AppState::WelcomeScreen => {
                self.welcome_timer += dt;
                if self.welcome_timer > 2.0 {
                    next_state = Some(AppState::Desktop);
                } else {
                    next_state = Some(AppState::WelcomeScreen);
                }
            }

            AppState::VirusMode => {
                while let Some(c) = get_char_pressed() {
                    if c == '\u{8}' && !self.virus_text.is_empty() {
                        self.virus_text.pop();
                    } else if c.is_ascii() && !c.is_control() {
                        self.virus_text.push(c);
                    }
                }

                if self.virus_text.trim().to_lowercase() == "stop" {
                    let _ = remove_file("VIRUS.txt");
                    std::process::exit(0);
                    return true;
                }
                
                next_state = Some(AppState::VirusMode);
            }

            AppState::Desktop => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let mouse = mouse_position();
                    let mouse_vec = vec2(mouse.0, mouse.1);
                    
                    for icon in &self.icons {
                        if icon.rect.contains(mouse_vec) {
                            match icon.app_type {
                                AppType::Chatbot => next_state = Some(AppState::Chatbot(Chatbot::new())),
                                AppType::FloppyDisk => next_state = Some(AppState::FloppyDisk(FloppyDiskGame::new().await)),
                                AppType::Chess => next_state = Some(AppState::Chess(ChessGame::new().await)),
                                AppType::Vedic => next_state = Some(AppState::Vedic(MathQuestion::new(0))),
                                AppType::Notepad => next_state = Some(AppState::Notepad(Notepad::new())),
                            }
                            break;
                        }
                    }
                    
                    if next_state.is_none() {
                        next_state = Some(AppState::Desktop);
                    }
                } else if self.esc_pressed {
                    return true;
                } else {
                    next_state = Some(AppState::Desktop);
                }
            }

            AppState::Chatbot(mut chatbot) => {
                if self.esc_pressed || chatbot.update().await {
                    next_state = Some(AppState::Desktop);
                } else {
                    next_state = Some(AppState::Chatbot(chatbot));
                }
            }

            AppState::Chess(mut game) => {
                match game.update() {
                    GameStatus::Checkmate => next_state = Some(AppState::Desktop),
                    GameStatus::Exit => return true,
                    _ => {
                        if self.esc_pressed {
                            next_state = Some(AppState::Desktop);
                        } else {
                            next_state = Some(AppState::Chess(game));
                        }
                    }
                }
            }

            AppState::MathQuestion(mut question) => {
                if question.update() {
                    next_state = Some(AppState::Desktop);
                } else if let Some(hacker_mode) = question.hacker_mode.take() {
                    next_state = Some(AppState::HackerMode(hacker_mode, question));
                } else if self.esc_pressed {
                    next_state = Some(AppState::Desktop);
                } else {
                    next_state = Some(AppState::MathQuestion(question));
                }
            }

            AppState::HackerMode(mut hacker_mode, question) => {
                hacker_mode.update();
                next_state = if self.esc_pressed || !hacker_mode.triggered {
                    Some(AppState::MathQuestion(question))
                } else {
                    Some(AppState::HackerMode(hacker_mode, question))
                };
            }

            AppState::Notepad(mut notepad) => {
                let should_exit = notepad.update();
                next_state = if self.esc_pressed || should_exit {
                    Some(AppState::Desktop)
                } else {
                    Some(AppState::Notepad(notepad))
                };
            }

            AppState::FloppyDisk(mut game) => {
                if is_key_pressed(KeyCode::R) {
                    next_state = Some(AppState::FloppyDisk(FloppyDiskGame::new().await));
                } else {
                    game.update(dt);
                    
                    if self.esc_pressed {
                        next_state = Some(AppState::Desktop);
                    } else {
                        next_state = Some(AppState::FloppyDisk(game));
                    }
                }
            }

            AppState::Vedic(mut question) => {
                if question.update() {
                    next_state = Some(AppState::Desktop);
                } else if self.esc_pressed {
                    next_state = Some(AppState::Desktop);
                } else {
                    next_state = Some(AppState::Vedic(question));
                }
            }

            other => next_state = Some(other),
        }

        if let Some(new_state) = next_state {
            self.state = new_state;
        }

        false
    }

    async fn activate_virus(&mut self) {
        let _ = std::fs::write("VIRUS.txt", "VIRUS ACTIVATED! Type 'stop' to exit");
        
        // Remove sound-related code
        
        // Modified thread spawning code
        std::thread::spawn(|| {
            for i in 0..10 {
                // On Windows, we need to use "notepad.exe" explicitly
                let _ = Command::new("notepad.exe").spawn();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    }

    pub fn draw(&mut self) {
        match &mut self.state {
            AppState::Booting => self.draw_booting(),
            AppState::PasswordScreen => self.draw_password_screen(),
            AppState::WelcomeScreen => self.draw_welcome_screen(),
            AppState::VirusMode => self.draw_virus_mode(),
            AppState::Desktop => self.draw_desktop(),
            AppState::FloppyDisk(game) => game.draw(),
            AppState::Chess(game) => game.draw(),
            AppState::MathQuestion(question) => question.draw(),
            AppState::Vedic(question) => question.draw(),
            AppState::HackerMode(hacker_mode, _) => hacker_mode.draw_hacker_ui(),
            AppState::Notepad(notepad) => notepad.draw(),
            AppState::Chatbot(chatbot) => chatbot.draw(),
        }
    }

    fn draw_booting(&self) {
        // Animated gradient background
        let gradient_offset = (self.anime_effect_timer * 0.5).sin() * 0.2 + 0.8;
        clear_background(Color::from_rgba(
            (20.0 * gradient_offset) as u8,
            (25.0 * gradient_offset) as u8,
            (45.0 * gradient_offset) as u8,
            255
        ));

        // Draw grid pattern
        for i in 0..80 {
            for j in 0..60 {
                let x = i as f32 * 16.0;
                let y = j as f32 * 12.0;
                let alpha = (0.1 + 0.05 * ((x + y + self.anime_effect_timer * 100.0) * 0.01).sin()).max(0.0).min(0.3);
                draw_rectangle(x, y, 1.0, 1.0, Color::from_rgba(0, 255, 150, (alpha * 255.0) as u8));
            }
        }

        let pulse = 0.7 + 0.3 * (self.anime_effect_timer * 3.0).sin();
        let spinner_chars = ["◐", "◓", "◑", "◒"];
        let spinner_idx = ((self.anime_effect_timer * 8.0) as usize) % spinner_chars.len();

        // Main boot text with glow effect
        let boot_text = "RETRO VM INITIALIZING";
        let text_width = measure_text(boot_text, None, 36, 1.0).width;
        let text_x = (screen_width() - text_width) / 2.0;
        let text_y = screen_height() / 2.0 - 50.0;

        // Glow effect
        for offset in [-2.0, -1.0, 1.0, 2.0] {
            draw_text(boot_text, text_x + offset, text_y, 36.0, 
                     Color::from_rgba(0, 255, 150, (30.0 * pulse) as u8));
        }
        
        draw_text(boot_text, text_x, text_y, 36.0, 
                 Color::from_rgba(0, 255, 150, (255.0 * pulse) as u8));
        
        // Animated spinner
        draw_text(
            spinner_chars[spinner_idx],
            screen_width() / 2.0 - 20.0,
            screen_height() / 2.0 + 20.0,
            48.0,
            Color::from_rgba(0, 255, 150, (255.0 * pulse) as u8)
        );

        // Progress bar
        let bar_width = 400.0;
        let bar_height = 8.0;
        let bar_x = (screen_width() - bar_width) / 2.0;
        let bar_y = screen_height() / 2.0 + 80.0;
        let progress = ((get_time() - self.boot_start) / 3.0).min(1.0) as f32;

        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::from_rgba(40, 40, 60, 255));
        draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, 
                      Color::from_rgba(0, 255, 150, 255));
    }

    fn draw_password_screen(&self) {
        // Animated matrix-style background
        clear_background(Color::from_rgba(0, 0, 0, 255));
        
        // Security-themed UI
        let panel_width = 500.0;
        let panel_height = 300.0;
        let panel_x = (screen_width() - panel_width) / 2.0;
        let panel_y = (screen_height() - panel_height) / 2.0;

        // Panel background with glow
        draw_rectangle(panel_x - 2.0, panel_y - 2.0, panel_width + 4.0, panel_height + 4.0, 
                      Color::from_rgba(0, 255, 150, 80));
        draw_rectangle(panel_x, panel_y, panel_width, panel_height, 
                      Color::from_rgba(10, 20, 30, 240));

        // Title
        let title = "SECURE ACCESS REQUIRED";
        let title_width = measure_text(title, None, 28, 1.0).width;
        draw_text(title, panel_x + (panel_width - title_width) / 2.0, panel_y + 60.0, 28.0, 
                 Color::from_rgba(0, 255, 150, 255));

        // Lock icon
        draw_text("🔒", panel_x + panel_width / 2.0 - 20.0, panel_y + 120.0, 40.0, 
                 Color::from_rgba(255, 255, 255, 255));

        // Input field
        let input_width = 300.0;
        let input_height = 40.0;
        let input_x = panel_x + (panel_width - input_width) / 2.0;
        let input_y = panel_y + 160.0;

        draw_rectangle(input_x, input_y, input_width, input_height, 
                      Color::from_rgba(20, 30, 40, 255));
        draw_rectangle_lines(input_x, input_y, input_width, input_height, 2.0, 
                            Color::from_rgba(0, 255, 150, 255));

        // Password dots
        let dots = "•".repeat(self.password_input.len());
        draw_text(&dots, input_x + 10.0, input_y + 28.0, 24.0, WHITE);

        // Blinking cursor
        if (get_time() * 2.0).sin() > 0.0 {
            let cursor_x = input_x + 10.0 + measure_text(&dots, None, 24, 1.0).width;
            draw_rectangle(cursor_x, input_y + 8.0, 2.0, 24.0, 
                          Color::from_rgba(0, 255, 150, 255));
        }

        // Instructions
        let instruction = "Enter access code and press ENTER";
        let inst_width = measure_text(instruction, None, 18, 1.0).width;
        draw_text(instruction, panel_x + (panel_width - inst_width) / 2.0, panel_y + 240.0, 18.0, 
                 Color::from_rgba(150, 150, 150, 255));
    }

    fn draw_welcome_screen(&self) {
        clear_background(Color::from_rgba(0, 20, 40, 255));
        
        let welcome_text = "WELCOME TO RETRO VM";
        let text_width = measure_text(welcome_text, None, 40, 1.0).width;
        let pulse = 0.7 + 0.3 * (self.welcome_timer * 4.0).sin();
        
        draw_text(welcome_text, (screen_width() - text_width) / 2.0, screen_height() / 2.0, 40.0, 
                 Color::from_rgba(0, 255, 150, (255.0 * pulse) as u8));
        
        let sub_text = "Kaise ho Theek ho!";
        let sub_width = measure_text(sub_text, None, 24, 1.0).width;
        draw_text(sub_text, (screen_width() - sub_width) / 2.0, screen_height() / 2.0 + 60.0, 24.0, 
                 Color::from_rgba(255, 255, 255, 200));
    }

    fn draw_virus_mode(&self) {
        clear_background(Color::from_rgba(139, 0, 0, 255));
        
        // Flashing effect
        let flash = if (get_time() * 10.0).sin() > 0.5 { 255 } else { 200 };
        
        draw_text("⚠️ CRITICAL SYSTEM ERROR ⚠️", 100.0, 150.0, 32.0, 
                 Color::from_rgba(255, 255, 0, flash));
        draw_text("VIRUS DETECTED - SYSTEM COMPROMISED", 80.0, 200.0, 28.0, 
                 Color::from_rgba(255, 255, 255, 255));
        draw_text("Type 'stop' to terminate...", 150.0, 300.0, 24.0, 
                 Color::from_rgba(255, 255, 255, 255));
        
        // Input field
        let input_bg = Rect::new(100.0, 350.0, 400.0, 40.0);
        draw_rectangle(input_bg.x, input_bg.y, input_bg.w, input_bg.h, 
                      Color::from_rgba(0, 0, 0, 200));
        draw_text(&self.virus_text, input_bg.x + 10.0, input_bg.y + 28.0, 24.0, WHITE);
    }

    fn draw_desktop(&mut self) {
        // Draw background texture
        draw_texture(
            &self.background_texture,
            0.0,
            0.0,
            WHITE
        );

        // Draw particles
        for particle in &self.desktop_particles {
            draw_circle(particle.pos.x, particle.pos.y, particle.size, particle.color);
        }

        // Draw title bar
        self.draw_title_bar();

        // Draw icons with professional styling
        self.draw_professional_icons();

        // Draw taskbar
        self.draw_taskbar();
    }

    fn draw_title_bar(&self) {
        let title_height = 50.0;
        draw_rectangle(0.0, 0.0, screen_width(), title_height, 
                      Color::from_rgba(20, 30, 50, 230));
        
        let title = "RETRO VM DESKTOP ENVIRONMENT";
        let title_width = measure_text(title, None, 24, 1.0).width;
        draw_text(title, (screen_width() - title_width) / 2.0, 32.0, 24.0, 
                 Color::from_rgba(200, 220, 255, 255));
    }

    fn draw_professional_icons(&self) {
        let mouse = mouse_position();
        let mouse_vec = vec2(mouse.0, mouse.1);
        
        for (i, icon) in self.icons.iter().enumerate() {
            let is_hovered = icon.rect.contains(mouse_vec);
            let hover_scale = if is_hovered { 1.05 } else { 1.0 };
            let animation_offset = (self.anime_effect_timer * 2.0 + i as f32 * 0.5).sin() * 2.0;
            
            // Icon background with modern styling
            let bg_rect = Rect::new(
                icon.rect.x - 5.0,
                icon.rect.y - 5.0 + animation_offset,
                icon.rect.w + 10.0,
                icon.rect.h + 10.0
            );
            
            // Drop shadow
            draw_rectangle(bg_rect.x + 3.0, bg_rect.y + 3.0, bg_rect.w, bg_rect.h, 
                          Color::from_rgba(0, 0, 0, 60));
            
            // Main background
            let bg_color = if is_hovered { icon.hover_color } else { icon.primary_color };
            draw_rectangle(bg_rect.x, bg_rect.y, bg_rect.w, bg_rect.h, bg_color);
            
            // Glass effect border
            draw_rectangle_lines(bg_rect.x, bg_rect.y, bg_rect.w, bg_rect.h, 2.0, 
                               Color::from_rgba(255, 255, 255, 80));
            
            // Inner glow
            draw_rectangle_lines(bg_rect.x + 1.0, bg_rect.y + 1.0, bg_rect.w - 2.0, bg_rect.h - 2.0, 1.0, 
                                icon.accent_color);
            
            // Draw icon texture
            let texture_size = vec2(icon.rect.w * 0.6 * hover_scale, icon.rect.h * 0.6 * hover_scale);
            let texture_pos = vec2(
                icon.rect.x + (icon.rect.w - texture_size.x) / 2.0,
                icon.rect.y + 20.0 + animation_offset
            );
            
            draw_texture_ex(
                &icon.texture,
                texture_pos.x,
                texture_pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(texture_size),
                    source: None,
                    rotation: 0.0,
                    flip_x: false,
                    flip_y: false,
                    pivot: None,
                }
            );
            
            // Label with clean typography
            let label_width = measure_text(&icon.label, None, 14, 1.0).width;
            let label_x = icon.rect.x + (icon.rect.w - label_width) / 2.0;
            let label_y = icon.rect.y + icon.rect.h - 15.0 + animation_offset;
            
            // Label background
            draw_rectangle(label_x - 5.0, label_y - 18.0, label_width + 10.0, 20.0, 
                            Color::from_rgba(0, 0, 0, 120));
            
            draw_text(&icon.label, label_x, label_y, 14.0, WHITE);
            
            // Hover effect - subtle pulse
            if is_hovered {
                let pulse = 0.3 + 0.2 * (self.anime_effect_timer * 6.0).sin();
                draw_rectangle_lines(bg_rect.x - 2.0, bg_rect.y - 2.0, bg_rect.w + 4.0, bg_rect.h + 4.0, 2.0, 
                                   Color::from_rgba(255, 255, 255, (pulse * 255.0) as u8));
            }
        }
    }

    fn draw_taskbar(&self) {
        let taskbar_y = screen_height() - self.taskbar_height;
        
        // Taskbar background with gradient
        for i in 0..self.taskbar_height as i32 {
            let alpha = 200 - (i as f32 / self.taskbar_height * 50.0) as u8;
            draw_rectangle(0.0, taskbar_y + i as f32, screen_width(), 1.0, 
                        Color::from_rgba(10, 20, 35, alpha));
        }
        
        // Taskbar border
        draw_rectangle(0.0, taskbar_y, screen_width(), 2.0, 
                    Color::from_rgba(0, 150, 255, 150));
        
        // System info section
        let info_text = format!("RETRO VM | {} | ESC to exit", self.time_display);
        let info_width = measure_text(&info_text, None, 16, 1.0).width;
        draw_text(&info_text, 20.0, taskbar_y + 25.0, 16.0, 
                    Color::from_rgba(200, 220, 255, 255));
        
        // Status indicators
        let status_x = screen_width() - 200.0;
        
        // CPU indicator (animated)
        let cpu_activity = 0.3 + 0.7 * (self.anime_effect_timer * 3.0).sin().abs();
        draw_text("CPU:", status_x, taskbar_y + 25.0, 14.0, 
                    Color::from_rgba(150, 150, 150, 255));
        
        let cpu_bar_width = 60.0;
        let cpu_bar_height = 8.0;
        let cpu_bar_x = status_x + 35.0;
        let cpu_bar_y = taskbar_y + 15.0;
        
        draw_rectangle(cpu_bar_x, cpu_bar_y, cpu_bar_width, cpu_bar_height, 
                    Color::from_rgba(40, 40, 60, 255));
        draw_rectangle(cpu_bar_x, cpu_bar_y, cpu_bar_width * cpu_activity, cpu_bar_height, 
                    Color::from_rgba(0, 255, 150, 255));
        
        // Memory indicator
        let mem_activity = 0.6;
        draw_text("MEM:", status_x, taskbar_y + 45.0, 14.0, 
                    Color::from_rgba(150, 150, 150, 255));
        
        let mem_bar_y = taskbar_y + 35.0;
        draw_rectangle(cpu_bar_x, mem_bar_y, cpu_bar_width, cpu_bar_height, 
                    Color::from_rgba(40, 40, 60, 255));
        draw_rectangle(cpu_bar_x, mem_bar_y, cpu_bar_width * mem_activity, cpu_bar_height, 
                    Color::from_rgba(255, 150, 0, 255));
    }
}