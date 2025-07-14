use macroquad::prelude::*;
use macroquad::texture::FilterMode;
use macroquad::rand::gen_range;
use image::{Rgba, DynamicImage, GenericImageView};
use std::fs::File;
use std::io::Read;
use base64::{Engine as _, engine::general_purpose};
use std::path::PathBuf;
use rfd::FileDialog;

// Constants
const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WALL_THICKNESS: f32 = 6.0;
const MARGIN: f32 = 20.0;
const HUD_HEIGHT: f32 = 40.0;
const COLS: usize = 16;
const ROWS: usize = 12;
const ENEMY_COUNT: usize = 5;
const PLAYER_HEALTH: i32 = 3;
const GAME_DURATION: f32 = 180.0; // 3 minutes

// Game states
#[derive(PartialEq)]
enum GameState {
    Input,
    Playing,
    Win,
    Loss,
}

// Input modes
enum InputMode {
    ImagePath,
    SecretMessage,
}

// Maze cell structure
#[derive(Clone, Copy)]
struct Cell {
    visited: bool,
    walls: [bool; 4], // top, right, bottom, left
}

impl Cell {
    fn new() -> Self {
        Self {
            visited: false,
            walls: [true; 4],
        }
    }
}

// Wall structure
struct Wall {
    rect: Rect,
}

impl Wall {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { rect: Rect::new(x, y, w, h) }
    }

    fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, YELLOW);
    }
}

// Enemy structure
struct Enemy {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    size: Vec2,
    direction: Vec2,
    direction_change_timer: f32,
}

impl Enemy {
    async fn new(x: f32, y: f32) -> Self {
        let texture = load_texture("assets/virus.png")
            .await
            .unwrap_or_else(|_| {
                // Fallback texture if image not found
                let white_texture = Texture2D::from_rgba8(1, 1, &[255, 0, 0, 255]);
                white_texture.set_filter(FilterMode::Nearest);
                white_texture
            });
        texture.set_filter(FilterMode::Nearest);

        Self {
            pos: vec2(x, y),
            speed: 80.0,
            texture,
            size: vec2(32.0, 32.0),
            direction: vec2(1.0, 0.0),
            direction_change_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32, walls: &[Wall]) {
        self.direction_change_timer -= dt;
        
        if self.direction_change_timer <= 0.0 {
            let directions = [
                vec2(1.0, 0.0),
                vec2(-1.0, 0.0),
                vec2(0.0, 1.0),
                vec2(0.0, -1.0),
            ];
            self.direction = directions[gen_range(0, directions.len())];
            self.direction_change_timer = gen_range(1.0, 3.0);
        }

        let new_pos = self.pos + self.direction.normalize() * self.speed * dt;
        let padding = 6.0;
        let future_rect = Rect::new(
            new_pos.x + padding / 2.0,
            new_pos.y + padding / 2.0,
            self.size.x - padding,
            self.size.y - padding,
        );

        if !walls.iter().any(|w| w.rect.overlaps(&future_rect)) {
            self.pos = new_pos;
        } else {
            self.direction_change_timer = 0.0;
        }
    }

    fn draw(&self) {
        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                ..Default::default()
            }
        );
    }

    fn collision_rect(&self) -> Rect {
        let padding = 6.0;
        Rect::new(
            self.pos.x + padding / 2.0,
            self.pos.y + padding / 2.0,
            self.size.x - padding,
            self.size.y - padding,
        )
    }
}

// Player structure
struct Player {
    pos: Vec2,
    speed: f32,
    texture: Texture2D,
    size: Vec2,
    hit_cooldown: f32,
}

impl Player {
    async fn new() -> Self {
        let texture = load_texture("assets/floppy.png")
            .await
            .unwrap_or_else(|_| {
                let white_texture = Texture2D::from_rgba8(1, 1, &[255, 255, 255, 255]);
                white_texture.set_filter(FilterMode::Nearest);
                white_texture
            });
        texture.set_filter(FilterMode::Nearest);

        let cell_width = (WINDOW_WIDTH - 2.0 * MARGIN) / COLS as f32;
        let cell_height = (WINDOW_HEIGHT - 2.0 * MARGIN - HUD_HEIGHT) / ROWS as f32;
        let start_x = MARGIN + cell_width / 2.0 - 16.0;
        let start_y = MARGIN + HUD_HEIGHT + cell_height / 2.0 - 16.0;

        Self {
            pos: vec2(start_x, start_y),
            speed: 200.0,
            texture,
            size: vec2(32.0, 32.0),
            hit_cooldown: 0.0,
        }
    }

    fn update(&mut self, dt: f32, walls: &[Wall]) {
        self.hit_cooldown -= dt;
        
        let mut dir = vec2(0.0, 0.0);
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { dir.x += 1.0; }
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { dir.x -= 1.0; }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { dir.y -= 1.0; }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { dir.y += 1.0; }

        if dir != Vec2::ZERO {
            let new_pos = self.pos + dir.normalize() * self.speed * dt;
            let padding = 6.0;
            let future_rect = Rect::new(
                new_pos.x + padding / 2.0,
                new_pos.y + padding / 2.0,
                self.size.x - padding,
                self.size.y - padding,
            );

            if !walls.iter().any(|w| w.rect.overlaps(&future_rect)) {
                self.pos = new_pos;
            }
        }
    }

    fn draw(&self) {
        let color = if self.hit_cooldown > 0.0 && (self.hit_cooldown * 10.0).sin() > 0.0 {
            Color::new(1.0, 0.5, 0.5, 1.0) // Red tint when hit
        } else {
            WHITE
        };

        draw_texture_ex(
            &self.texture,
            self.pos.x,
            self.pos.y,
            color,
            DrawTextureParams {
                dest_size: Some(self.size),
                ..Default::default()
            }
        );
    }

    fn collision_rect(&self) -> Rect {
        let padding = 6.0;
        Rect::new(
            self.pos.x + padding / 2.0,
            self.pos.y + padding / 2.0,
            self.size.x - padding,
            self.size.y - padding,
        )
    }
}

// Main game structure
pub struct FloppyDiskGame {
    state: GameState,
    player: Player,
    walls: Vec<Wall>,
    enemies: Vec<Enemy>,
    cpu_texture: Texture2D,
    cpu_size: Vec2,
    cpu_pos: Vec2,
    time_left: f32,
    grace_timer: f32,
    health: i32,
    input_mode: InputMode,
    image_path: String,
    secret_message: String,
    current_input: String,
    result_message: Option<String>,
}

impl FloppyDiskGame {
    pub async fn new() -> Self {
        // Preload textures
        let cpu_texture = load_texture("assets/cpu.png")
            .await
            .unwrap_or_else(|_| {
                let tex = Texture2D::from_rgba8(1, 1, &[0, 255, 0, 255]);
                tex.set_filter(FilterMode::Nearest);
                tex
            });
        cpu_texture.set_filter(FilterMode::Nearest);

        let player = Player::new().await;
        let walls = generate_maze_walls();
        let enemies = spawn_enemies().await;
        let cpu_size = vec2(48.0, 48.0);
        let cpu_pos = vec2(
            WINDOW_WIDTH - cpu_size.x - MARGIN - 10.0,
            WINDOW_HEIGHT - cpu_size.y - MARGIN - 10.0
        );

        Self {
            state: GameState::Input,
            player,
            walls,
            enemies,
            cpu_texture,
            cpu_size,
            cpu_pos,
            time_left: GAME_DURATION,
            grace_timer: 1.0,
            health: PLAYER_HEALTH,
            input_mode: InputMode::ImagePath,
            image_path: String::new(),
            secret_message: String::new(),
            current_input: String::new(),
            result_message: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        match self.state {
            GameState::Input => self.handle_input(),
            GameState::Playing => self.update_game(dt),
            GameState::Win | GameState::Loss => self.handle_game_end(),
        }

        // Restart game if R is pressed
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Enter) {
            match self.input_mode {
                InputMode::ImagePath => {
                // Open file dialog to select image using rfd
                    if let Some(path) = FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        self.image_path = path.to_string_lossy().to_string();
                        self.current_input.clear();
                        self.input_mode = InputMode::SecretMessage;
                    }
                },
                InputMode::SecretMessage => {
                    self.secret_message = self.current_input.clone();
                    self.current_input.clear();
                    self.state = GameState::Playing;
                },
            }
        } else if is_key_pressed(KeyCode::Backspace) {
            self.current_input.pop();
        } else {
            // Handle text input
            let chars = get_char_pressed();
            for c in chars {
                if c.is_ascii() {
                    self.current_input.push(c);
                }
            }
        }
    }

    fn update_game(&mut self, dt: f32) {
        self.time_left -= dt;
        
        // Grace period at game start
        if self.grace_timer > 0.0 {
            self.grace_timer -= dt;
            self.player.update(dt, &[]);
        } else {
            self.player.update(dt, &self.walls);
        }

        // Update enemies
        for enemy in &mut self.enemies {
            enemy.update(dt, &self.walls);
        }

        // Check for enemy collisions
        if self.player.hit_cooldown <= 0.0 {
            for enemy in &self.enemies {
                if self.player.collision_rect().overlaps(&enemy.collision_rect()) {
                    self.health -= 1;
                    self.player.hit_cooldown = 1.5;
                    break;
                }
            }
        }

        // Check win condition
        if self.player.collision_rect().overlaps(&Rect::new(
            self.cpu_pos.x, self.cpu_pos.y, 
            self.cpu_size.x, self.cpu_size.y
        )) {
            self.state = GameState::Win;
            self.process_win();
        }

        // Check loss conditions
        if self.health <= 0 || self.time_left <= 0.0 {
            self.state = GameState::Loss;
            self.process_loss();
        }
    }

    fn handle_game_end(&self) {
        // Game end logic (handled in draw)
    }

    fn process_win(&mut self) {
        // Create output directories if they don't exist
        let encoded_dir = "encoded";
        let binary_dir = "binary";
        std::fs::create_dir_all(encoded_dir).unwrap_or_default();
        std::fs::create_dir_all(binary_dir).unwrap_or_default();

        // Generate output filenames
        let input_path = PathBuf::from(&self.image_path);
        let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        
        let encoded_path = format!("{}/{}_encoded.png", encoded_dir, file_stem);
        let binary_path = format!("{}/{}_binary.txt", binary_dir, file_stem);

        match self.encode_message(&self.image_path, &self.secret_message, &encoded_path) {
            Ok(_) => {
                // Also save the binary representation
                if let Ok(binary_str) = self.image_to_binary(&self.image_path) {
                    std::fs::write(&binary_path, binary_str).unwrap_or_default();
                    self.result_message = Some(format!(
                        "Success!\nEncoded image saved to: {}\nBinary data saved to: {}",
                        encoded_path, binary_path
                    ));
                } else {
                    self.result_message = Some(format!(
                        "Encoded image saved to: {}\nFailed to save binary data",
                        encoded_path
                    ));
                }
            },
            Err(e) => {
                self.result_message = Some(format!("Error encoding message: {}", e));
            }
        }
    }

    fn process_loss(&mut self) {
        let binary_dir = "binary";
        std::fs::create_dir_all(binary_dir).unwrap_or_default();

        let input_path = PathBuf::from(&self.image_path);
        let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        let binary_path = format!("{}/{}_binary.txt", binary_dir, file_stem);

        match self.image_to_binary(&self.image_path) {
            Ok(binary_str) => {
                std::fs::write(&binary_path, &binary_str).unwrap_or_default();
                let truncated = if binary_str.len() > 50 {
                    format!("{}...", &binary_str[..50])
                } else {
                    binary_str.clone()
                };
                self.result_message = Some(format!(
                    "Image converted to binary (first 50 chars): {}\nSaved to: {}",
                    truncated, binary_path
                ));
            },
            Err(e) => {
                self.result_message = Some(format!("Error converting image: {}", e));
            }
        }
    }

    fn encode_message(&self, image_path: &str, message: &str, output_path: &str) -> Result<(), String> {
        let mut img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        
        let msg_bytes = message.as_bytes();
        let msg_len = msg_bytes.len() as u32;
        let mut data = Vec::with_capacity(4 + msg_bytes.len());
        data.extend_from_slice(&msg_len.to_be_bytes());
        data.extend_from_slice(msg_bytes);

        self.lsb_encode(&mut img, &data)?;

        img.save(output_path)
            .map_err(|e| format!("Failed to save image: {}", e))?;

        Ok(())
    }

    fn lsb_encode(&self, img: &mut DynamicImage, data: &[u8]) -> Result<(), String> {
        let (width, height) = img.dimensions();
        let mut img_buf = img.to_rgba8();
        
        let required_pixels = data.len() * 8 / 3;
        if ((width * height) as usize) < required_pixels {
            return Err("Image too small for message".to_string());
        }
        
        let mut data_iter = data.iter().flat_map(|&b| (0..8).map(move |i| (b >> i) & 1));
        
        for (_x, _y, pixel) in img_buf.enumerate_pixels_mut() {
            if let Some(bit) = data_iter.next() {
                let Rgba([r, g, b, a]) = *pixel;
                *pixel = Rgba([
                    (r & 0xFE) | bit,
                    (g & 0xFE) | (data_iter.next().unwrap_or(0)),
                    (b & 0xFE) | (data_iter.next().unwrap_or(0)),
                    a
                ]);
            } else {
                break;
            }
        }

        *img = DynamicImage::ImageRgba8(img_buf);
        Ok(())
    }

    fn image_to_binary(&self, image_path: &str) -> Result<String, String> {
        let mut file = File::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read image: {}", e))?;
        
        Ok(buffer.iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<String>>()
            .join(" "))
    }

    async fn reset(&mut self) {
        *self = FloppyDiskGame::new().await;
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        match self.state {
            GameState::Input => self.draw_input_screen(),
            GameState::Playing => self.draw_game(),
            GameState::Win => self.draw_game_over(true),
            GameState::Loss => self.draw_game_over(false),
        }
    }

    fn draw_input_screen(&self) {
        let prompt = match self.input_mode {
            InputMode::ImagePath => "Press Enter to select an image file",
            InputMode::SecretMessage => "Enter secret message to encode:",
        };
        
        draw_text(
            prompt,
            screen_width() / 2.0 - 150.0,
            screen_height() / 2.0 - 50.0,
            24.0,
            WHITE,
        );
        
        if let InputMode::SecretMessage = self.input_mode {
            draw_text(
                &self.current_input,
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0,
                24.0,
                WHITE,
            );
        }
        
        if let InputMode::ImagePath = self.input_mode {
            draw_text(
                &self.image_path,
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0,
                24.0,
                WHITE,
            );
        }
        
        draw_text(
            "Press Enter to continue",
            screen_width() / 2.0 - 150.0,
            screen_height() / 2.0 + 50.0,
            20.0,
            GRAY,
        );
    }

    fn draw_game(&self) {
        // Draw walls
        for wall in &self.walls {
            wall.draw();
        }
        
        // Draw enemies
        for enemy in &self.enemies {
            enemy.draw();
        }
        
        // Draw player
        self.player.draw();

        // Draw CPU
        draw_texture_ex(
            &self.cpu_texture,
            self.cpu_pos.x,
            self.cpu_pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.cpu_size),
                ..Default::default()
            }
        );
        // Draw HUD
        let health_color = if self.health <= 1 { RED } else { GREEN };
        draw_text(
            &format!("Health: {}", self.health), 
            10.0, 30.0, 24.0, health_color
        );
        
        let timer_color = if self.time_left < 10.0 { RED } else { GREEN };
        draw_text(
            &format!("Time Left: {:.1}s", self.time_left), 
            150.0, 30.0, 24.0, timer_color
        );

        // Time warning
        if self.time_left < 10.0 {
            draw_rectangle(
                0.0, 0.0, 
                screen_width(), screen_height(), 
                Color::from_rgba(255, 0, 0, 80)
            );
            draw_text(
                "⚠️ DISK IS CORRUPTING!",
                screen_width() / 2.0 - 170.0,
                screen_height() / 2.0 - 40.0,
                36.0,
                ORANGE,
            );
        }

        draw_text(
            "Press R to Restart", 
            screen_width() - 200.0, 30.0, 20.0, GRAY
        );
    }

    fn draw_game_over(&self, won: bool) {
        self.draw_game(); // Draw the game behind the overlay

        // Dark overlay
        draw_rectangle(
            0.0, 0.0, 
            screen_width(), screen_height(), 
            Color::from_rgba(0, 0, 0, 180)
        );

        // Game over message
        if won {
            draw_text(
                "✅ Delivered to CPU!", 
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0 - 60.0,
                36.0,
                GREEN,
            );
        } else {
            draw_text(
                "💀 Disk Corrupted! Game Over.", 
                screen_width() / 2.0 - 200.0,
                screen_height() / 2.0 - 60.0,
                36.0,
                RED,
            );
        }

        // Result message
        if let Some(msg) = &self.result_message {
            let lines: Vec<&str> = msg.split('\n').collect();
            for (i, line) in lines.iter().enumerate() {
                draw_text(
                    line,
                    screen_width() / 2.0 - 250.0,
                    screen_height() / 2.0 + 20.0 + (i as f32 * 25.0),
                    20.0,
                    YELLOW,
                );
            }
        }

        draw_text(
            "Press R to Restart", 
            screen_width() / 2.0 - 100.0,
            screen_height() / 2.0 + 100.0,
            24.0,
            WHITE,
        );
    }
}

// Helper functions
fn generate_maze_walls() -> Vec<Wall> {
    let mut grid = vec![vec![Cell::new(); COLS]; ROWS];
    let mut walls = vec![];
    let mut stack = vec![];
    let mut current = (0, 0);
    grid[0][0].visited = true;

    while let Some(_) = Some(current) {
        let (x, y) = current;

        let mut neighbors = vec![];
        if y > 0 && !grid[y - 1][x].visited { neighbors.push((x, y - 1, 0)); }
        if x < COLS - 1 && !grid[y][x + 1].visited { neighbors.push((x + 1, y, 1)); }
        if y < ROWS - 1 && !grid[y + 1][x].visited { neighbors.push((x, y + 1, 2)); }
        if x > 0 && !grid[y][x - 1].visited { neighbors.push((x - 1, y, 3)); }

        if !neighbors.is_empty() {
            let (nx, ny, dir) = neighbors[gen_range(0, neighbors.len())];
            grid[y][x].walls[dir] = false;
            grid[ny][nx].walls[(dir + 2) % 4] = false;
            stack.push(current);
            grid[ny][nx].visited = true;
            current = (nx, ny);
        } else if let Some(prev) = stack.pop() {
            current = prev;
        } else {
            break;
        }
    }

    let cell_width = (WINDOW_WIDTH - 2.0 * MARGIN) / COLS as f32;
    let cell_height = (WINDOW_HEIGHT - 2.0 * MARGIN - HUD_HEIGHT) / ROWS as f32;

    for y in 0..ROWS {
        for x in 0..COLS {
            let cx = MARGIN + x as f32 * cell_width;
            let cy = MARGIN + HUD_HEIGHT + y as f32 * cell_height;
            let cell = grid[y][x];

            if cell.walls[0] {
                walls.push(Wall::new(cx, cy, cell_width, WALL_THICKNESS));
            }
            if cell.walls[1] {
                walls.push(Wall::new(cx + cell_width - WALL_THICKNESS, cy, WALL_THICKNESS, cell_height));
            }
            if cell.walls[2] {
                walls.push(Wall::new(cx, cy + cell_height - WALL_THICKNESS, cell_width, WALL_THICKNESS));
            }
            if cell.walls[3] {
                walls.push(Wall::new(cx, cy, WALL_THICKNESS, cell_height));
            }
        }
    }

    walls
}

async fn spawn_enemies() -> Vec<Enemy> {
    let cell_width = (WINDOW_WIDTH - 2.0 * MARGIN) / COLS as f32;
    let cell_height = (WINDOW_HEIGHT - 2.0 * MARGIN - HUD_HEIGHT) / ROWS as f32;
    let mut enemies = Vec::new();

    for _ in 0..ENEMY_COUNT {
        let x = gen_range(5, COLS);
        let y = gen_range(5, ROWS);
        let cx = MARGIN + x as f32 * cell_width + cell_width / 2.0 - 16.0;
        let cy = MARGIN + HUD_HEIGHT + y as f32 * cell_height + cell_height / 2.0 - 16.0;
        
        enemies.push(Enemy::new(cx, cy).await);
    }

    enemies
}

#[macroquad::main("Floppy Disk Courier")]
async fn main() {
    let mut game = FloppyDiskGame::new().await;

    loop {
        let dt = get_frame_time();
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}