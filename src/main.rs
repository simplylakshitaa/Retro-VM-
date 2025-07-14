mod modes;

use macroquad::prelude::*;
use modes::{NormalMode, ghost_an::GhostAnimation, ghost::GhostMode};

enum AppState {
    Menu,
    Normal(NormalMode),
    Ghost(GhostState),  
}

enum GhostState {
    Animation(GhostAnimation),
    Manager(GhostMode),
}

struct MenuTab {
    label: &'static str,
    icon: &'static str,
    position: Vec2,
    size: Vec2,
    active: bool,
    hover_animation: f32,
}

impl MenuTab {
    fn new(label: &'static str, icon: &'static str, position: Vec2, size: Vec2) -> Self {
        MenuTab {
            label,
            icon,
            position,
            size,
            active: false,
            hover_animation: 0.0,
        }
    }

    fn update(&mut self) {
        let mouse = mouse_position().into();
        let hovered = Rect::new(self.position.x, self.position.y, self.size.x, self.size.y).contains(mouse);
        
        let target = if hovered || self.active { 1.0 } else { 0.0 };
        self.hover_animation = lerp(self.hover_animation, target, 0.25);
    }

    fn draw(&self) -> bool {
        let mouse = mouse_position().into();
        let hovered = Rect::new(self.position.x, self.position.y, self.size.x, self.size.y).contains(mouse);
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

        let base_alpha = 0.08 + (self.hover_animation * 0.25);
        let border_alpha = 0.3 + (self.hover_animation * 0.4);
        let blur_alpha = 0.25 + (self.hover_animation * 0.15);
        
        for i in 0..3 {
            let offset = (i as f32 + 1.0) * 0.8;
            let alpha = blur_alpha * (0.4 - i as f32 * 0.1);
            draw_rectangle(
                self.position.x - offset,
                self.position.y - offset,
                self.size.x + offset * 2.0,
                self.size.y + offset * 2.0,
                Color::new(0.0, 0.0, 0.0, alpha),
            );
        }

        draw_rectangle(
            self.position.x - 1.0,
            self.position.y - 1.0,
            self.size.x + 2.0,
            self.size.y + 2.0,
            Color::new(0.05, 0.05, 0.2, 0.4),
        );

        let bg_color = if self.active {
            Color::new(0.95, 0.95, 1.0, base_alpha + 0.08)
        } else {
            Color::new(0.9, 0.9, 1.0, base_alpha)
        };

        let border_color = Color::new(0.95, 0.95, 1.0, border_alpha);

        draw_rectangle(
            self.position.x + 2.0,
            self.position.y + 2.0,
            self.size.x,
            self.size.y,
            Color::new(0.0, 0.0, 0.0, 0.2),
        );

        draw_rectangle(self.position.x, self.position.y, self.size.x, self.size.y, bg_color);
        
        draw_rectangle_lines(self.position.x, self.position.y, self.size.x, self.size.y, 2.0, border_color);
        
        if self.active {
            draw_rectangle_lines(
                self.position.x + 1.0, 
                self.position.y + 1.0, 
                self.size.x - 2.0, 
                self.size.y - 2.0, 
                1.0, 
                Color::new(1.0, 1.0, 1.0, 0.3)
            );
        }

        let icon_size = 24.0;
        let icon_alpha = 0.95 + (self.hover_animation * 0.05);
        draw_text(
            self.icon,
            self.position.x + 20.0,
            self.position.y + self.size.y / 2.0 + 8.0,
            icon_size,
            Color::new(1.0, 1.0, 1.0, icon_alpha),
        );

        let text_alpha = 0.98 + (self.hover_animation * 0.02);
        let text_color = if self.active {
            Color::new(1.0, 1.0, 1.0, text_alpha)
        } else {
            Color::new(0.95, 0.95, 1.0, text_alpha)
        };

        draw_text(
            self.label,
            self.position.x + 55.0,
            self.position.y + self.size.y / 2.0 + 6.0,
            22.0,
            text_color,
        );

        clicked
    }
}

struct ContentPanel {
    position: Vec2,
    size: Vec2,
    alpha: f32,
}

impl ContentPanel {
    fn new(position: Vec2, size: Vec2) -> Self {
        ContentPanel {
            position,
            size,
            alpha: 0.0,
        }
    }

    fn update(&mut self) {
        self.alpha = lerp(self.alpha, 1.0, 0.08);
    }

    fn draw(&self) {
        let panel_alpha = self.alpha * 0.15;
        let border_alpha = self.alpha * 0.25;
        let blur_alpha = self.alpha * 0.3;

        for i in 0..4 {
            let offset = (i as f32 + 1.0) * 1.2;
            let alpha = blur_alpha * (0.5 - i as f32 * 0.1);
            draw_rectangle(
                self.position.x - offset,
                self.position.y - offset,
                self.size.x + offset * 2.0,
                self.size.y + offset * 2.0,
                Color::new(0.0, 0.0, 0.0, alpha),
            );
        }

        draw_rectangle(
            self.position.x - 2.0,
            self.position.y - 2.0,
            self.size.x + 4.0,
            self.size.y + 4.0,
            Color::new(0.05, 0.05, 0.15, 0.45),
        );

        draw_rectangle(
            self.position.x - 1.0,
            self.position.y - 1.0,
            self.size.x + 2.0,
            self.size.y + 2.0,
            Color::new(0.1, 0.1, 0.2, 0.35),
        );

        draw_rectangle(
            self.position.x,
            self.position.y,
            self.size.x,
            self.size.y,
            Color::new(0.9, 0.9, 1.0, panel_alpha),
        );

        draw_rectangle_lines(
            self.position.x,
            self.position.y,
            self.size.x,
            self.size.y,
            2.0,
            Color::new(0.95, 0.95, 1.0, border_alpha),
        );

        draw_rectangle_lines(
            self.position.x + 1.0,
            self.position.y + 1.0,
            self.size.x - 2.0,
            self.size.y - 2.0,
            1.0,
            Color::new(1.0, 1.0, 1.0, border_alpha * 0.5),
        );
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn draw_elegant_text(text: &str, x: f32, y: f32, size: f32, alpha: f32) {
    draw_text(
        text,
        x,
        y,
        size,
        Color::new(1.0, 1.0, 1.0, alpha),
    );
}

fn draw_title_text(text: &str, x: f32, y: f32, size: f32, alpha: f32) {
   
    draw_text(
        text,
        x + 1.0,
        y + 1.0,
        size,
        Color::new(0.6, 0.6, 0.9, alpha * 0.4),
    );
    
    draw_text(
        text,
        x,
        y,
        size,
        Color::new(1.0, 1.0, 1.0, alpha),
    );
}

fn draw_header_with_blur(rect: Rect, alpha: f32) {
    for i in 0..3 {
        let offset = (i as f32 + 1.0) * 1.0;
        let blur_alpha = alpha * (0.4 - i as f32 * 0.1);
        draw_rectangle(
            rect.x,
            rect.y - offset,
            rect.w,
            rect.h + offset * 2.0,
            Color::new(0.0, 0.0, 0.0, blur_alpha),
        );
    }
    
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.05, 0.05, 0.15, alpha),
    );
    
    draw_rectangle_lines(
        rect.x,
        rect.y + rect.h - 1.0,
        rect.w,
        1.0,
        2.0,
        Color::new(0.9, 0.9, 1.0, alpha * 0.6),
    );
}

#[macroquad::main("Retro VM")]
async fn main() {
    let mut state = AppState::Menu;
    let mut content_panel = ContentPanel::new(Vec2::ZERO, Vec2::ZERO);
    let mut frame_time = 0.0;

    let bg_texture = load_texture("assets/mainbg.png").await.unwrap_or(Texture2D::empty());

    loop {
        frame_time += get_frame_time();
        
        clear_background(Color::new(0.05, 0.05, 0.15, 1.0));

        draw_texture_ex(
            &bg_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.05, 0.05, 0.1, 0.15),
        );

        if is_key_pressed(KeyCode::Tab) {
            state = AppState::Menu;
        }

        match &mut state {
            AppState::Menu => {
                let sw = screen_width();
                let sh = screen_height();
                
                let header_height = 120.0;
                let header_rect = Rect::new(0.0, 0.0, sw, header_height);
                
                draw_header_with_blur(header_rect, 0.45);

                draw_title_text(
                    "RETRO VM",
                    60.0,
                    50.0,
                    42.0,
                    0.95,
                );

                draw_elegant_text(
                    "Virtual Machine Management Suite",
                    60.0,
                    80.0,
                    18.0,
                    0.7,
                );

                let tab_width = 280.0;
                let tab_height = 60.0;
                let tab_spacing = 20.0;
                let tabs_start_x = (sw - (tab_width * 2.0 + tab_spacing)) / 2.0;
                let tabs_y = 160.0;

                let mut tabs = vec![
                    MenuTab::new("Normal Mode", "🖥", vec2(tabs_start_x, tabs_y), vec2(tab_width, tab_height)),
                    MenuTab::new("Ghost Mode", "👻", vec2(tabs_start_x + tab_width + tab_spacing, tabs_y), vec2(tab_width, tab_height)),
                ];

                match &state {
                    AppState::Normal(_) => tabs[0].active = true,
                    AppState::Ghost(_) => tabs[1].active = true,
                    _ => {}
                }

                for (i, tab) in tabs.iter_mut().enumerate() {
                    tab.update();
                    if tab.draw() {
                        match i {
                            0 => state = AppState::Normal(NormalMode::new().await),
                            1 => state = AppState::Ghost(GhostState::Animation(GhostAnimation::new().await)),
                            _ => {}
                        }
                    }
                }

                let content_margin = 60.0;
                let content_y = tabs_y + tab_height + 40.0;
                let content_rect = Rect::new(
                    content_margin,
                    content_y,
                    sw - (content_margin * 2.0),
                    sh - content_y - content_margin,
                );

                content_panel.position = content_rect.point().into();
                content_panel.size = content_rect.size().into();
                content_panel.update();
                content_panel.draw();

                let content_x = content_rect.x + 40.0;
                let content_text_y = content_rect.y + 50.0;

                draw_elegant_text(
                    "Welcome to Retro VM",
                    content_x,
                    content_text_y,
                    28.0,
                    0.85,
                );

                let description = "Select a mode from the navigation above to begin your virtual machine management experience.";
                draw_elegant_text(
                    description,
                    content_x,
                    content_text_y + 45.0,
                    18.0,
                    0.65,
                );

                let features = vec![
                    "• Normal Mode - Standard VM interface with full system control",
                    "• Ghost Mode - Advanced features with animation preview",
                    "• Professional UI - Clean, elegant, and responsive design",
                    "• Quick Navigation - Press Tab to return to menu anytime",
                ];

                for (i, feature) in features.iter().enumerate() {
                    draw_elegant_text(
                        feature,
                        content_x + 20.0,
                        content_text_y + 100.0 + (i as f32 * 30.0),
                        16.0,
                        0.55,
                    );
                }

                let status_alpha = (frame_time * 2.0).sin() * 0.3 + 0.7;
                draw_elegant_text(
                    "● System Ready",
                    content_x,
                    content_rect.y + content_rect.h - 40.0,
                    16.0,
                    status_alpha * 0.65,
                );
            }

            AppState::Normal(normal) => {
                if normal.update().await {
                    state = AppState::Menu;
                } else {
                    normal.draw();
                }
            }

            AppState::Ghost(ghost_state) => {
                match ghost_state {
                    GhostState::Animation(ghost_anim) => {
                        if is_key_down(KeyCode::LeftShift) && is_key_pressed(KeyCode::D) {
                            *ghost_state = GhostState::Manager(GhostMode::new());
                        } else {
                            ghost_anim.update();
                            ghost_anim.draw();
                        }
                    }
                    GhostState::Manager(ghost_mode) => {
                        ghost_mode.update().await;
                        ghost_mode.draw();
                    }
                }
            }
        }

        next_frame().await;
    }
}