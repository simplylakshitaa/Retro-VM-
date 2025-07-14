use macroquad::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct GhostMode {
    kali_iso_path: String,
    ram_gb: u8,
    cpu_cores: u8,
    qemu_path: String,
    is_vm_running: bool,
    vm_process: Option<std::process::Child>,
    last_error: Option<String>,
    show_animation: bool,
    animation_timer: u32,
    file_dialog_open: bool,
    file_dialog_target: FileDialogTarget,
    show_help: bool,
}

enum FileDialogTarget {
    QemuPath,
    IsoPath,
}

impl GhostMode {
    pub fn new() -> Self {
        Self {
            kali_iso_path: "assets/kali-linux-2025.2-installer-amd64.iso".to_string(),
            ram_gb: 4,
            cpu_cores: 2,
            qemu_path: Self::default_qemu_path(),
            is_vm_running: false,
            vm_process: None,
            last_error: None,
            show_animation: true,
            animation_timer: 0,
            file_dialog_open: false,
            file_dialog_target: FileDialogTarget::QemuPath,
            show_help: false,
        }
    }

    pub async fn update(&mut self) {
        self.animation_timer = self.animation_timer.wrapping_add(1);
        
        if self.file_dialog_open {
            if is_key_pressed(KeyCode::Escape) {
                self.file_dialog_open = false;
            }
            return;
        }

        if is_key_pressed(KeyCode::Escape) {
            if self.show_animation {
                std::process::exit(0);
            } else {
                self.show_animation = true;
                self.animation_timer = 0;
            }
        }

        if is_key_pressed(KeyCode::F1) {
            self.show_help = !self.show_help;
        }
    }

    pub fn draw(&mut self) {
        // Gradient background
        clear_background(Color::from_rgba(15, 15, 25, 255));

        if self.show_animation {
            self.draw_animation();
        } else {
            // Main content container
            draw_rectangle(20., 20., screen_width() - 40., screen_height() - 40., 
                Color::from_rgba(30, 30, 50, 220));
            draw_rectangle_lines(20., 20., screen_width() - 40., screen_height() - 40., 2., 
                Color::from_rgba(100, 100, 150, 255));

            self.draw_header();
            self.draw_configuration();
            self.draw_actions();
            self.draw_status();
            
            if self.show_help {
                self.draw_help();
            }
        }

        if self.file_dialog_open {
            self.draw_file_dialog();
        }
    }

    fn draw_animation(&mut self) {
        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;
        
        // Glowing background effect
        let pulse = (self.animation_timer as f32 * 0.05).sin().abs();
        draw_circle(center_x, center_y, 150., 
            Color::from_rgba(100, 100, 200, (30. + pulse * 30.) as u8));

        draw_text_ex(
            "👻 GHOST MODE",
            center_x - 150.0,
            center_y - 150.0,
            TextParams {
                font_size: 48,
                color: Color::from_rgba(200, 200, 255, 255),
                ..Default::default()
            },
        );

        let frame = (self.animation_timer / 15) % 6;
        let ghost_size = 80.0;
        let ghost_y_offset = match frame {
            1 | 3 => 5.0,
            2 | 4 => -5.0,
            _ => 0.0,
        };

        // Draw ghost with shadow
        draw_circle(center_x, center_y + ghost_y_offset + 10., ghost_size, 
            Color::from_rgba(0, 0, 20, 60));
        draw_circle(center_x, center_y + ghost_y_offset, ghost_size, WHITE);
        
        // Eyes that follow mouse
        let mouse_pos = mouse_position();
        let look_dir = (mouse_pos.0 - center_x).signum() * 0.3;
        draw_circle(center_x - 20. + look_dir * 10., center_y + ghost_y_offset - 10., 10., BLUE);
        draw_circle(center_x + 20. + look_dir * 10., center_y + ghost_y_offset - 10., 10., BLUE);

        // Animated mouth
        let mouth = match frame {
            0..=1 => "⌣",
            2..=3 => "—",
            _ => "⌢",
        };
        draw_text(mouth, center_x - 10., center_y + ghost_y_offset + 15., 30., BLUE);

        // Enhanced button
        if Self::draw_button_ex(
            "🚀 ENTER VM CONTROLS",
            center_x - 150.0,
            center_y + 150.0,
            Some(300.0),
            Some(50.0),
            TextParams {
                font_size: 24,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 300.0,
                height: 50.0,
                normal_color: Some(Color::from_rgba(200, 200, 255, 255)),
                hover_color: Some(Color::from_rgba(180, 180, 255, 255)),
                pressed_color: Some(Color::from_rgba(160, 160, 255, 255)),
                border_color: Some(Color::from_rgba(100, 100, 200, 255)),
            },
        ) {
            self.show_animation = false;
        }
    }

    fn draw_header(&self) {
        draw_text_ex(
            "👻 GHOST MODE - KALI LINUX VM",
            40.0,
            60.0,
            TextParams {
                font_size: 32,
                color: Color::from_rgba(200, 200, 255, 255),
                ..Default::default()
            },
        );
        draw_line(40.0, 90.0, screen_width() - 40.0, 90.0, 2.0, 
            Color::from_rgba(100, 100, 150, 255));
    }

    fn draw_configuration(&mut self) {
        let mut y = 120.0;
        let section_width = screen_width() - 80.0;

        // Section title
        draw_text_ex(
            "VM CONFIGURATION",
            40.0,
            y - 10.0,
            TextParams {
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        y += 30.0;

        // Configuration items with better layout
        let item_height = 40.0;
        let label_width = 150.0;
        let value_width = 300.0;
        let button_width = 40.0;

        let qemu_path = self.qemu_path.clone();
        let is_default_qemu = qemu_path == Self::default_qemu_path();
        self.draw_config_item(
            "QEMU Path:", 
            &qemu_path, 
            40.0, y, 
            label_width, value_width, button_width, item_height,
            is_default_qemu
        );
        y += item_height + 10.0;

        let kali_iso_path = self.kali_iso_path.clone();
        self.draw_config_item(
            "Kali ISO:", 
            &kali_iso_path, 
            40.0, y, 
            label_width, value_width, button_width, item_height,
            false
        );
        y += item_height + 20.0;

        // Sliders section
        draw_text_ex(
            "RESOURCE ALLOCATION",
            40.0,
            y - 10.0,
            TextParams {
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        y += 30.0;

        // RAM Slider with visual indicator
        self.draw_slider(
            &format!("RAM: {} GB", self.ram_gb),
            40.0, y, 
            section_width, item_height,
            self.ram_gb, 2, 32,
            Color::from_rgba(100, 200, 100, 255)
        );
        y += item_height + 10.0;

        // CPU Cores Slider
        self.draw_slider(
            &format!("CPU Cores: {}", self.cpu_cores),
            40.0, y, 
            section_width, item_height,
            self.cpu_cores, 1, 8,
            Color::from_rgba(100, 100, 200, 255)
        );
    }

    fn draw_config_item(
        &mut self,
        label: &str,
        value: &str,
        x: f32, y: f32,
        label_w: f32, value_w: f32, button_w: f32, h: f32,
        is_default: bool
    ) {
        // Label
        draw_text_ex(
            label,
            x,
            y + h / 2.0 + 8.0,
            TextParams {
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );

        // Value background
        draw_rectangle(
            x + label_w,
            y,
            value_w,
            h,
            Color::from_rgba(40, 40, 60, 255),
        );
        draw_rectangle_lines(
            x + label_w,
            y,
            value_w,
            h,
            1.0,
            Color::from_rgba(80, 80, 100, 255),
        );

        // Value text
        draw_text_ex(
            value,
            x + label_w + 10.0,
            y + h / 2.0 + 8.0,
            TextParams {
                font_size: 20,
                color: if is_default { GRAY } else { WHITE },
                ..Default::default()
            },
        );

        // Browse button
        if Self::draw_button_ex(
            "📂",
            x + label_w + value_w + 10.0,
            y,
            Some(button_w),
            Some(h),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: button_w,
                height: h,
                normal_color: Some(Color::from_rgba(80, 80, 120, 255)),
                hover_color: Some(Color::from_rgba(100, 100, 150, 255)),
                pressed_color: Some(Color::from_rgba(120, 120, 180, 255)),
                border_color: Some(Color::from_rgba(60, 60, 100, 255)),
            },
        ) {
            self.file_dialog_open = true;
            match label {
                "QEMU Path:" => self.file_dialog_target = FileDialogTarget::QemuPath,
                "Kali ISO:" => self.file_dialog_target = FileDialogTarget::IsoPath,
                _ => {}
            }
        }
    }

    fn draw_slider(
        &mut self,
        label: &str,
        x: f32, y: f32,
        w: f32, h: f32,
        value: u8, min: u8, max: u8,
        color: Color
    ) {
        // Label
        draw_text_ex(
            label,
            x,
            y + h / 2.0 + 8.0,
            TextParams {
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );

        // Slider track
        let slider_x = x + 150.0;
        let slider_width = w - 150.0 - 100.0;
        draw_rectangle(
            slider_x,
            y + h / 2.0 - 5.0,
            slider_width,
            10.0,
            Color::from_rgba(50, 50, 70, 255),
        );

        // Slider fill
        let fill_width = slider_width * (value - min) as f32 / (max - min) as f32;
        draw_rectangle(
            slider_x,
            y + h / 2.0 - 5.0,
            fill_width,
            10.0,
            color,
        );

        // Slider handle
        let handle_x = slider_x + fill_width - 10.0;
        draw_rectangle(
            handle_x,
            y + h / 2.0 - 10.0,
            20.0,
            20.0,
            WHITE,
        );

        // Decrease button
        if Self::draw_button_ex(
            "-",
            slider_x + slider_width + 10.0,
            y,
            Some(40.0),
            Some(h),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 40.0,
                height: h,
                ..Default::default()
            },
        ) && value > min {
            match label {
                s if s.contains("RAM") => self.ram_gb -= 1,
                s if s.contains("CPU") => self.cpu_cores -= 1,
                _ => {}
            }
        }

        // Increase button
        if Self::draw_button_ex(
            "+",
            slider_x + slider_width + 60.0,
            y,
            Some(40.0),
            Some(h),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 40.0,
                height: h,
                ..Default::default()
            },
        ) && value < max {
            match label {
                s if s.contains("RAM") => self.ram_gb += 1,
                s if s.contains("CPU") => self.cpu_cores += 1,
                _ => {}
            }
        }
    }

    fn draw_actions(&mut self) {
        let y = screen_height() - 150.0;
        let button_spacing = 20.0;

        // Launch/Terminate buttons
        let button_width = 180.0;
        let button_height = 50.0;

        if Self::draw_button_ex(
            if self.is_vm_running { "☠️ TERMINATE VM" } else { "🚀 LAUNCH VM" },
            40.0,
            y,
            Some(button_width),
            Some(button_height),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: button_width,
                height: button_height,
                normal_color: Some(if self.is_vm_running { 
                    Color::from_rgba(200, 100, 100, 255) 
                } else { 
                    Color::from_rgba(100, 200, 100, 255) 
                }),
                hover_color: Some(if self.is_vm_running { 
                    Color::from_rgba(220, 120, 120, 255) 
                } else { 
                    Color::from_rgba(120, 220, 120, 255) 
                }),
                pressed_color: Some(if self.is_vm_running { 
                    Color::from_rgba(180, 80, 80, 255) 
                } else { 
                    Color::from_rgba(80, 180, 80, 255) 
                }),
                border_color: Some(Color::from_rgba(60, 60, 100, 255)),
            },
        ) {
            if self.is_vm_running {
                self.kill_vm();
            } else {
                self.launch_vm();
            }
        }

        // Help button
        if Self::draw_button_ex(
            "❓ HELP",
            40.0 + button_width + button_spacing,
            y,
            Some(100.0),
            Some(button_height),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 100.0,
                height: button_height,
                normal_color: Some(Color::from_rgba(100, 100, 200, 255)),
                hover_color: Some(Color::from_rgba(120, 120, 220, 255)),
                pressed_color: Some(Color::from_rgba(80, 80, 180, 255)),
                border_color: Some(Color::from_rgba(60, 60, 100, 255)),
            },
        ) {
            self.show_help = !self.show_help;
        }

        // Back to animation button
        if Self::draw_button_ex(
            "👻 BACK",
            screen_width() - 40.0 - 120.0,
            y,
            Some(120.0),
            Some(button_height),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 120.0,
                height: button_height,
                normal_color: Some(Color::from_rgba(200, 200, 255, 255)),
                hover_color: Some(Color::from_rgba(180, 180, 255, 255)),
                pressed_color: Some(Color::from_rgba(160, 160, 255, 255)),
                border_color: Some(Color::from_rgba(100, 100, 200, 255)),
            },
        ) {
            self.show_animation = true;
            self.animation_timer = 0;
        }
    }

    fn draw_status(&self) {
        let y = screen_height() - 80.0;
        
        // Status bar
        draw_rectangle(
            40.0,
            y,
            screen_width() - 80.0,
            40.0,
            Color::from_rgba(40, 40, 60, 255),
        );
        draw_rectangle_lines(
            40.0,
            y,
            screen_width() - 80.0,
            40.0,
            1.0,
            Color::from_rgba(80, 80, 100, 255),
        );

        let status_text = if self.is_vm_running {
            format!("● VM ACTIVE (PID: {})", 
                self.vm_process.as_ref().map(|p| p.id()).unwrap_or(0))
        } else {
            "○ VM INACTIVE".to_string()
        };

        let status_color = if self.is_vm_running { 
            Color::from_rgba(100, 255, 100, 255) 
        } else { 
            Color::from_rgba(255, 100, 100, 255) 
        };

        draw_text_ex(
            &status_text,
            50.0,
            y + 25.0,
            TextParams {
                font_size: 20,
                color: status_color,
                ..Default::default()
            },
        );

        if let Some(err) = &self.last_error {
            draw_text_ex(
                &format!("ERROR: {}", err),
                screen_width() / 2.0,
                y + 25.0,
                TextParams {
                    font_size: 20,
                    color: Color::from_rgba(255, 100, 100, 255),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_help(&self) {
        let width = screen_width() * 0.7;
        let height = screen_height() * 0.6;
        let x = (screen_width() - width) / 2.0;
        let y = (screen_height() - height) / 2.0;

        // Help panel background
        draw_rectangle(x, y, width, height, Color::from_rgba(20, 20, 40, 240));
        draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(100, 100, 200, 255));

        // Title
        draw_text_ex(
            "GHOST MODE HELP",
            x + 20.0,
            y + 40.0,
            TextParams {
                font_size: 30,
                color: Color::from_rgba(200, 200, 255, 255),
                ..Default::default()
            },
        );

        let mut help_y = y + 80.0;
        let line_height = 25.0;

        // Help content
        let help_text = [
            "1. Download Kali ISO from https://www.kali.org/get-kali/",
            "",
            #[cfg(target_os = "windows")]
            "2. Install QEMU from https://qemu.weilnetz.de/w64/",
            #[cfg(target_os = "windows")]
            "3. Default path: C:\\Program Files\\qemu\\qemu-system-x86_64.exe",
            #[cfg(target_os = "windows")]
            "4. WHPX often fails - this uses software fallback (TCG)",
            #[cfg(target_os = "linux")]
            "2. Install QEMU/KVM: sudo apt install qemu-kvm",
            "",
            "Controls:",
            "- ESC: Return to animation/exit",
            "- F1: Toggle this help",
            "- Click buttons to interact",
        ];

        for line in help_text.iter() {
            if !line.is_empty() {
                draw_text_ex(
                    line,
                    x + 30.0,
                    help_y,
                    TextParams {
                        font_size: 20,
                        color: LIGHTGRAY,
                        ..Default::default()
                    },
                );
            }
            help_y += line_height;
        }

        // Close button
        if Self::draw_button_ex(
            "CLOSE",
            x + width - 120.0,
            y + height - 50.0,
            Some(100.0),
            Some(40.0),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: 100.0,
                height: 40.0,
                normal_color: Some(Color::from_rgba(200, 200, 255, 255)),
                hover_color: Some(Color::from_rgba(180, 180, 255, 255)),
                pressed_color: Some(Color::from_rgba(160, 160, 255, 255)),
                border_color: Some(Color::from_rgba(100, 100, 200, 255)),
            },
        ) {
            // Button click handled in update()
        }
    }

    fn draw_file_dialog(&mut self) {
        let width = 500.0;
        let height = 300.0;
        let x = (screen_width() - width) / 2.0;
        let y = (screen_height() - height) / 2.0;

        // Dialog background
        draw_rectangle(x, y, width, height, Color::from_rgba(40, 40, 60, 255));
        draw_rectangle_lines(x, y, width, height, 2.0, Color::from_rgba(100, 100, 200, 255));

        // Title
        let title = match self.file_dialog_target {
            FileDialogTarget::QemuPath => "SELECT QEMU EXECUTABLE",
            FileDialogTarget::IsoPath => "SELECT KALI ISO FILE",
        };
        draw_text_ex(
            title,
            x + 20.0,
            y + 40.0,
            TextParams {
                font_size: 24,
                color: WHITE,
                ..Default::default()
            },
        );

        // In a real implementation, you'd show file browser here
        draw_text_ex(
            "File browser would appear here",
            x + 20.0,
            y + 100.0,
            TextParams {
                font_size: 20,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );

        // Buttons
        let button_y = y + height - 70.0;
        let button_width = 120.0;
        let button_height = 40.0;

        // Select button
        if Self::draw_button_ex(
            "SELECT",
            x + width - 2.0 * button_width - 20.0,
            button_y,
            Some(button_width),
            Some(button_height),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: button_width,
                height: button_height,
                normal_color: Some(Color::from_rgba(100, 200, 100, 255)),
                hover_color: Some(Color::from_rgba(120, 220, 120, 255)),
                pressed_color: Some(Color::from_rgba(80, 180, 80, 255)),
                border_color: Some(Color::from_rgba(60, 100, 60, 255)),
            },
        ) {
            // In a real implementation, you'd get the selected file path here
            match self.file_dialog_target {
                FileDialogTarget::QemuPath => self.qemu_path = "new_path".to_string(),
                FileDialogTarget::IsoPath => self.kali_iso_path = "new_path".to_string(),
            }
            self.file_dialog_open = false;
        }

        // Cancel button
        if Self::draw_button_ex(
            "CANCEL",
            x + width - button_width - 10.0,
            button_y,
            Some(button_width),
            Some(button_height),
            TextParams {
                font_size: 20,
                color: BLACK,
                ..Default::default()
            },
            ButtonParams {
                width: button_width,
                height: button_height,
                normal_color: Some(Color::from_rgba(200, 100, 100, 255)),
                hover_color: Some(Color::from_rgba(220, 120, 120, 255)),
                pressed_color: Some(Color::from_rgba(180, 80, 80, 255)),
                border_color: Some(Color::from_rgba(100, 60, 60, 255)),
            },
        ) {
            self.file_dialog_open = false;
        }
    }

    fn launch_vm(&mut self) {
        if self.is_vm_running {
            self.last_error = Some("VM is already running".into());
            return;
        }

        let qemu_exe = if PathBuf::from(&self.qemu_path).exists() {
            self.qemu_path.clone()
        } else {
            Self::default_qemu_path()
        };

        let mut cmd = Command::new(qemu_exe);
        cmd.args(&[
            "-m", &format!("{}", (self.ram_gb as u32) * 1024),
            "-smp", &self.cpu_cores.to_string(),
            "-cdrom", &self.kali_iso_path,
            "-boot", "d",
            "-vga", "virtio",
            "-net", "nic",
            "-net", "user",
            "-cpu", "qemu64",
            "-accel", "tcg",
            "-display", "sdl",
        ]);

        match cmd.spawn() {
            Ok(child) => {
                self.vm_process = Some(child);
                self.is_vm_running = true;
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to launch QEMU: {}", e));
                self.is_vm_running = false;
            }
        }
    }

    fn kill_vm(&mut self) {
        if let Some(mut child) = self.vm_process.take() {
            if let Err(e) = child.kill() {
                self.last_error = Some(format!("Failed to kill QEMU process: {}", e));
            } else {
                self.is_vm_running = false;
                self.last_error = None;
            }
        }
    }

    fn default_qemu_path() -> String {
        #[cfg(target_os = "windows")]
        { r"assets/qemu/qemu-system-x86_64.exe".to_string() }
        #[cfg(target_os = "linux")]
        { "qemu-system-x86_64".to_string() }
    }

    fn draw_button(label: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
        let mouse_pos = mouse_position();
        let hovered = Rect::new(x, y, w, h).contains(vec2(mouse_pos.0, mouse_pos.1));
        let color = if hovered {
            if is_mouse_button_down(MouseButton::Left) { 
                Color::from_rgba(80, 80, 120, 255) 
            } else { 
                Color::from_rgba(100, 100, 150, 255) 
            }
        } else { 
            Color::from_rgba(60, 60, 100, 255) 
        };
        
        draw_rectangle(x, y, w, h, color);
        draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(100, 100, 200, 255));
        
        draw_text_ex(
            label,
            x + w / 2.0 - measure_text(label, None, 20, 1.0).width / 2.0,
            y + h / 2.0 + 8.0,
            TextParams {
                font_size: 20,
                color: WHITE,
                ..Default::default()
            },
        );
        
        hovered && is_mouse_button_released(MouseButton::Left)
    }

    fn draw_button_ex(
        label: &str,
        x: f32,
        y: f32,
        w: Option<f32>,
        h: Option<f32>,
        text_params: TextParams,
        button_params: ButtonParams,
    ) -> bool {
        let w = w.unwrap_or(100.0);
        let h = h.unwrap_or(40.0);
        let mouse_pos = mouse_position();
        let hovered = Rect::new(x, y, w, h).contains(vec2(mouse_pos.0, mouse_pos.1));
        let color = if hovered {
            if is_mouse_button_down(MouseButton::Left) {
                button_params.pressed_color.unwrap_or(Color::from_rgba(80, 80, 120, 255))
            } else {
                button_params.hover_color.unwrap_or(Color::from_rgba(100, 100, 150, 255))
            }
        } else {
            button_params.normal_color.unwrap_or(Color::from_rgba(60, 60, 100, 255))
        };
        
        draw_rectangle(x, y, w, h, color);
        if let Some(border_color) = button_params.border_color {
            draw_rectangle_lines(x, y, w, h, 2.0, border_color);
        }
        
        draw_text_ex(
            label,
            x + w / 2.0 - measure_text(label, text_params.font, text_params.font_size, 1.0).width / 2.0,
            y + h / 2.0 + text_params.font_size as f32 / 3.0,
            text_params,
        );
        
        hovered && is_mouse_button_released(MouseButton::Left)
    }
}

struct ButtonParams {
    width: f32,
    height: f32,
    normal_color: Option<Color>,
    hover_color: Option<Color>,
    pressed_color: Option<Color>,
    border_color: Option<Color>,
}

impl Default for ButtonParams {
    fn default() -> Self {
        Self {
            width: 100.0,
            height: 40.0,
            normal_color: Some(Color::from_rgba(60, 60, 100, 255)),
            hover_color: Some(Color::from_rgba(80, 80, 120, 255)),
            pressed_color: Some(Color::from_rgba(40, 40, 80, 255)),
            border_color: Some(Color::from_rgba(100, 100, 200, 255)),
        }
    }
}

#[macroquad::main("Ghost Mode VM Manager")]
async fn main() {
    let mut ghost_mode = GhostMode::new();

    loop {
        ghost_mode.update().await;
        ghost_mode.draw();
        
        next_frame().await;
    }
}