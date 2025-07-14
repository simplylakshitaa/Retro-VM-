use super::ModeUI;
use eframe::egui;
use egui::Rounding;
use eframe::egui::{Color32, FontId, Visuals};
use std::collections::HashMap;
use std::time::Instant;

#[derive(PartialEq, Eq, Hash, Clone, Default)]
enum AppState {
    #[default]
    Desktop,
    Finder,
    Calculator,
    About,
}

pub struct NormalMode {
    state: AppState,
    desktop_items: Vec<DesktopItem>,
    clicked_item: Option<usize>,
    last_click_time: Instant,
    calculator_value: String,
    calculator_memory: f64,
    calculator_last_op: Option<String>,
    window_positions: HashMap<AppState, egui::Pos2>,
    window_sizes: HashMap<AppState, egui::Vec2>,
}

struct DesktopItem {
    name: String,
    icon: char,
    position: egui::Pos2,
    app: AppState,
}

impl Default for NormalMode {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalMode {
    pub fn new() -> Self {
        let mut window_positions = HashMap::new();
        let mut window_sizes = HashMap::new();

        window_positions.insert(AppState::Finder, egui::Pos2::new(100.0, 100.0));
        window_sizes.insert(AppState::Finder, egui::Vec2::new(400.0, 300.0));

        window_positions.insert(AppState::Calculator, egui::Pos2::new(200.0, 200.0));
        window_sizes.insert(AppState::Calculator, egui::Vec2::new(200.0, 300.0));

        Self {
            state: AppState::Desktop,
            desktop_items: vec![
                DesktopItem {
                    name: "System Disk".into(),
                    icon: '💾',
                    position: egui::Pos2::new(50.0, 90.0),
                    app: AppState::Finder,
                },
                DesktopItem {
                    name: "Calculator".into(),
                    icon: '🧮',
                    position: egui::Pos2::new(50.0, 190.0),
                    app: AppState::Calculator,
                },
            ],
            clicked_item: None,
            last_click_time: Instant::now(),
            calculator_value: "0".to_string(),
            calculator_memory: 0.0,
            calculator_last_op: None,
            window_positions,
            window_sizes,
        }
    }

    fn draw_desktop_item(&mut self, ui: &mut egui::Ui, index: usize, item: &DesktopItem) -> egui::Response {
        let icon_size = 40.0;
        let text_height = 16.0;
        let padding = 6.0;
        let total_width = icon_size + padding * 2.0;
        let total_height = icon_size + text_height + padding * 3.0;

        let rect = egui::Rect::from_min_size(item.position, egui::vec2(total_width, total_height));
        let (rect, response) = ui.allocate_exact_size(rect.size(), egui::Sense::click_and_drag());

        if response.clicked() {
            let now = Instant::now();
            if now.duration_since(self.last_click_time).as_secs_f32() < 0.5 {
                self.state = item.app.clone();
            }
            self.last_click_time = now;
            self.clicked_item = Some(index);
        }

        if response.dragged() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                self.desktop_items[index].position = pos;
            }
        }

        let bg_color = if self.clicked_item == Some(index) {
            Color32::from_rgb(180, 200, 255)
        } else {
            Color32::TRANSPARENT
        };

        ui.painter().rect_filled(rect, 6.0, bg_color);
        
        // Draw icon and text
        let icon_pos = rect.center_top() + egui::vec2(0.0, padding);
        ui.painter().text(
            icon_pos,
            egui::Align2::CENTER_TOP,
            item.icon.to_string(),
            FontId::new(32.0, egui::FontFamily::Proportional),
            Color32::BLACK,
        );

        let text_pos = icon_pos + egui::vec2(0.0, icon_size + 4.0);
        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_TOP,
            item.name.clone(),
            FontId::new(13.0, egui::FontFamily::Proportional),
            Color32::BLACK,
        );

        response
    }

    fn draw_finder_window(&mut self, ctx: &egui::Context) {
        let window_size = *self.window_sizes.get(&AppState::Finder).unwrap_or(&egui::Vec2::new(400.0, 300.0));
        let window_pos = *self.window_positions.get(&AppState::Finder).unwrap_or(&egui::Pos2::new(100.0, 100.0));

        let window = egui::Window::new("System Disk")
            .id(egui::Id::new("finder_window"))
            .default_size(window_size)
            .current_pos(window_pos)
            .collapsible(false)
            .resizable(true)
            .title_bar(true);

        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Applications");
                    ui.separator();
                    ui.label("System");
                    ui.label("Utilities");
                });

                ui.vertical(|ui| {
                    ui.label("Contents:");
                    ui.separator();
                    for i in 0..10 {
                        ui.label(format!("File {}.txt", i));
                    }
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("❌").clicked() {
                    self.state = AppState::Desktop;
                }
            });
        });
    }

    fn draw_calculator(&mut self, ctx: &egui::Context) {
        let window_size = *self.window_sizes.get(&AppState::Calculator).unwrap_or(&egui::Vec2::new(200.0, 300.0));
        let window_pos = *self.window_positions.get(&AppState::Calculator).unwrap_or(&egui::Pos2::new(200.0, 200.0));

        let window = egui::Window::new("Calculator")
            .id(egui::Id::new("calculator"))
            .default_size(window_size)
            .current_pos(window_pos)
            .collapsible(false)
            .resizable(true)
            .title_bar(true);

        window.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Calculator");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.calculator_value);
                });
            });
            ui.separator();

            let button_size = egui::vec2(40.0, 30.0);

            // Calculator buttons
            ui.horizontal(|ui| {
                if ui.add_sized(button_size, egui::Button::new("C")).clicked() {
                    self.calculator_value = "0".to_string();
                }
                if ui.add_sized(button_size, egui::Button::new("±")).clicked() {
                    if let Ok(val) = self.calculator_value.parse::<f64>() {
                        self.calculator_value = (-val).to_string();
                    }
                }
                if ui.add_sized(button_size, egui::Button::new("%")).clicked() {
                    if let Ok(val) = self.calculator_value.parse::<f64>() {
                        self.calculator_value = (val / 100.0).to_string();
                    }
                }
                if ui.add_sized(button_size, egui::Button::new("÷")).clicked() {
                    self.perform_calculator_operation("÷");
                }
            });

            // Number buttons
            ui.horizontal(|ui| {
                if ui.add_sized(button_size, egui::Button::new("7")).clicked() {
                    self.calculator_input("7");
                }
                if ui.add_sized(button_size, egui::Button::new("8")).clicked() {
                    self.calculator_input("8");
                }
                if ui.add_sized(button_size, egui::Button::new("9")).clicked() {
                    self.calculator_input("9");
                }
                if ui.add_sized(button_size, egui::Button::new("×")).clicked() {
                    self.perform_calculator_operation("×");
                }
            });

            ui.horizontal(|ui| {
                if ui.add_sized(button_size, egui::Button::new("4")).clicked() {
                    self.calculator_input("4");
                }
                if ui.add_sized(button_size, egui::Button::new("5")).clicked() {
                    self.calculator_input("5");
                }
                if ui.add_sized(button_size, egui::Button::new("6")).clicked() {
                    self.calculator_input("6");
                }
                if ui.add_sized(button_size, egui::Button::new("-")).clicked() {
                    self.perform_calculator_operation("-");
                }
            });

            ui.horizontal(|ui| {
                if ui.add_sized(button_size, egui::Button::new("1")).clicked() {
                    self.calculator_input("1");
                }
                if ui.add_sized(button_size, egui::Button::new("2")).clicked() {
                    self.calculator_input("2");
                }
                if ui.add_sized(button_size, egui::Button::new("3")).clicked() {
                    self.calculator_input("3");
                }
                if ui.add_sized(button_size, egui::Button::new("+")).clicked() {
                    self.perform_calculator_operation("+");
                }
            });

            ui.horizontal(|ui| {
                if ui.add_sized(button_size, egui::Button::new("0")).clicked() {
                    self.calculator_input("0");
                }
                if ui.add_sized(button_size, egui::Button::new(".")).clicked() {
                    if !self.calculator_value.contains('.') {
                        self.calculator_input(".");
                    }
                }
                if ui.add_sized(egui::vec2(80.0, 30.0), egui::Button::new("=")).clicked() {
                    self.perform_calculator_operation("=");
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("❌").clicked() {
                    self.state = AppState::Desktop;
                }
            });
        });
    }

    fn draw_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About This Mac")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([300.0, 200.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Macintosh");
                    ui.label("System Software 1.0");
                    ui.label("");
                    ui.label("© 1984 Apple Computer, Inc.");
                    ui.label("All Rights Reserved");
                    ui.label("");
                    if ui.button("OK").clicked() {
                        self.state = AppState::Desktop;
                    }
                });
            });
    }

    fn calculator_input(&mut self, digit: &str) {
        if self.calculator_value == "0" && digit != "." {
            self.calculator_value = digit.to_string();
        } else {
            self.calculator_value.push_str(digit);
        }
    }

    fn perform_calculator_operation(&mut self, op: &str) {
        let current_value = self.calculator_value.parse::<f64>().unwrap_or(0.0);

        if let Some(last_op) = &self.calculator_last_op {
            match last_op.as_str() {
                "+" => self.calculator_memory += current_value,
                "-" => self.calculator_memory -= current_value,
                "×" => self.calculator_memory *= current_value,
                "÷" => {
                    if current_value.abs() > f64::EPSILON {
                        self.calculator_memory /= current_value;
                    } else {
                        self.calculator_value = "Error".to_string();
                        self.calculator_last_op = None;
                        return;
                    }
                }
                _ => {}
            }
        } else {
            self.calculator_memory = current_value;
        }

        if op == "=" {
            self.calculator_value = self.calculator_memory.to_string();
            self.calculator_last_op = None;
        } else {
            self.calculator_last_op = Some(op.to_string());
            self.calculator_value = "0".to_string();
        }
    }
}

impl ModeUI for NormalMode {
    fn ui(&mut self, ctx: &egui::Context) {
        // Set Mac System 1 style visuals
        let mut visuals = Visuals::light();
        
        // Window and background colors
        visuals.window_fill = Color32::from_rgb(192, 192, 192);
        visuals.panel_fill = Color32::from_rgb(192, 192, 192);
        visuals.faint_bg_color = Color32::from_rgb(192, 192, 192);
        
        // Widget colors
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(192, 192, 192);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(192, 192, 192);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(160, 160, 160);
        visuals.widgets.active.bg_fill = Color32::from_rgb(128, 128, 128);
        
        // Window shadow
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [2.0 as i8, 2.0 as i8],
            blur: 3.0 as u8,
            spread: 0.0,
            color: Color32::from_black_alpha(100),
        };
        
        // Rounding (now using f32 instead of u8)// Removed invalid field: visuals.window_rounding
        
        // Widget rounding
        visuals.widgets.noninteractive = visuals.widgets.noninteractive.rounding(Rounding::same(4.0));
        visuals.widgets.inactive = visuals.widgets.inactive.rounding(Rounding::same(4.0));
        visuals.widgets.hovered = visuals.widgets.hovered.rounding(Rounding::same(4.0));
        visuals.widgets.active = visuals.widgets.active.rounding(Rounding::same(4.0));
        
        // Selection colors
        visuals.selection.bg_fill = Color32::from_rgb(0, 0, 128);
        visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
        
        ctx.set_visuals(visuals);

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("🍎", |ui| {
                    if ui.button("About This Mac...").clicked() {
                        self.state = AppState::About;
                        ui.close_menu();
                    }
                });

                if self.state == AppState::Finder {
                    ui.menu_button("File", |_| {});
                    ui.menu_button("Edit", |_| {});
                    ui.menu_button("View", |_| {});
                }
            });
        });

        // Desktop background and icons
        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw desktop background
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                Color32::from_rgb(192, 192, 192),
            );

            // Draw desktop items
            for (i, item) in self.desktop_items.iter().enumerate() {
                let response = self.draw_desktop_item(ui, i, item);
                if response.double_clicked() {
                    self.state = item.app.clone();
                }
            }

            // Draw windows based on current state
            match &self.state {
                AppState::Finder => self.draw_finder_window(ctx),
                AppState::Calculator => self.draw_calculator(ctx),
                AppState::About => self.draw_about_window(ctx),
                AppState::Desktop => {}
            }
        });
    }
}

use pixels::{Pixels, SurfaceTexture};
use winit::event::Event;
use winit::keyboard::KeyCode;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const SCALE: u32 = 4;

struct Ghost {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    animation_frame: usize,
    animation_timer: u32,
    wiggle_offset: f32,
}

impl Ghost {
    fn new() -> Self {
        Self {
            x: WIDTH as f32 / 2.0,
            y: HEIGHT as f32 / 2.0,
            velocity_x: 0.8,
            velocity_y: 0.5,
            animation_frame: 0,
            animation_timer: 0,
            wiggle_offset: 0.0,
        }
    }

    fn update(&mut self) {
        // Movement
        self.x += self.velocity_x;
        self.y += self.velocity_y + self.wiggle_offset.sin() * 0.3;
        self.wiggle_offset += 0.1;

        // Bounce off walls
        if self.x < 10.0 || self.x > WIDTH as f32 - 10.0 {
            self.velocity_x *= -1.0;
        }
        if self.y < 10.0 || self.y > HEIGHT as f32 - 20.0 {
            self.velocity_y *= -1.0;
        }

        // Animation
        self.animation_timer += 1;
        if self.animation_timer >= 10 {
            self.animation_timer = 0;
            self.animation_frame = (self.animation_frame + 1) % 4;
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        let ghost_colors = [
            [0xFF, 0xFF, 0xFF, 0xFF], // White
            [0xEE, 0xEE, 0xEE, 0xFF], // Light gray
            [0xDD, 0xDD, 0xDD, 0xFF], // Medium gray
            [0xCC, 0xCC, 0xCC, 0xFF], // Dark gray
        ];

        let base_x = self.x as i32;
        let base_y = self.y as i32;

        // Ghost body (different frames create animation)
        match self.animation_frame {
            0 => self.draw_ghost_body(frame, base_x, base_y, ghost_colors[0]),
            1 => self.draw_ghost_body(frame, base_x, base_y + 1, ghost_colors[1]),
            2 => self.draw_ghost_body(frame, base_x, base_y, ghost_colors[2]),
            3 => self.draw_ghost_body(frame, base_x, base_y + 1, ghost_colors[3]),
            _ => {}
        }

        // Eyes (same for all frames)
        self.draw_eyes(frame, base_x, base_y);
    }

    fn draw_ghost_body(&self, frame: &mut [u8], x: i32, y: i32, color: [u8; 4]) {
        // Ghost body pattern (11x16 pixels)
        let ghost_pattern = [
            [0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1],
            [1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];

        for (dy, row) in ghost_pattern.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    let px = x + dx as i32;
                    let py = y + dy as i32;
                    
                    if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
                        let index = (py as usize * WIDTH as usize + px as usize) * 4;
                        frame[index..index + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    fn draw_eyes(&self, frame: &mut [u8], x: i32, y: i32) {
        // Eyes pattern (uses same coordinates as ghost body)
        let eye_color = [0x00, 0x00, 0x88, 0xFF]; // Dark blue
        
        // Left eye (relative to ghost body)
        self.set_pixel(frame, x + 3, y + 3, eye_color);
        self.set_pixel(frame, x + 4, y + 3, eye_color);
        self.set_pixel(frame, x + 3, y + 4, eye_color);
        self.set_pixel(frame, x + 4, y + 4, eye_color);
        
        // Right eye
        self.set_pixel(frame, x + 6, y + 3, eye_color);
        self.set_pixel(frame, x + 7, y + 3, eye_color);
        self.set_pixel(frame, x + 6, y + 4, eye_color);
        self.set_pixel(frame, x + 7, y + 4, eye_color);
    }

    fn set_pixel(&self, frame: &mut [u8], x: i32, y: i32, color: [u8; 4]) {
        if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
            let index = (y as usize * WIDTH as usize + x as usize) * 4;
            frame[index..index + 4].copy_from_slice(&color);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut input = WinitInputHelper::new();
    
    let window = WindowBuilder::new()
        .with_title("Ghost Pixel Animation")
        .with_inner_size(winit::dpi::LogicalSize::new(WIDTH * SCALE, HEIGHT * SCALE))
        .build(&event_loop)?;
    
    let surface_texture = SurfaceTexture::new(WIDTH * SCALE, HEIGHT * SCALE, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
    
    let mut ghost = Ghost::new();

    event_loop.run(move |event, elwt| {
        if let Event::WindowEvent { event, .. } = &event {
            if input.close_requested() || input.destroyed() {
                elwt.exit();
                return;
            }
            if input.key_pressed(KeyCode::Escape) {
                elwt.exit();
                return;            }
        }

        if input.update(&event) {
            // Clear screen with dark background
            for pixel in pixels.frame_mut().chunks_exact_mut(4) {
                pixel.copy_from_slice(&[0x10, 0x10, 0x20, 0xff]);
            }

            ghost.update();
            ghost.draw(pixels.frame_mut());
            
            if pixels.render().is_err() {
                elwt.exit();
            }
        }
    })?;
    
    Ok(())
}

