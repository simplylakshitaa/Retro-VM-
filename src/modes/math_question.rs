use macroquad::prelude::*;
use crate::modes::hacker::HackerMode;

const BOARD_SIZE: usize = 4;
const TILE_SIZE: f32 = 80.0;
const TILE_MARGIN: f32 = 10.0;
const BOARD_MARGIN: f32 = 20.0;
const ANIMATION_SPEED: f32 = 7.0;
const MOVE_COOLDOWN: f32 = 0.1;

const BG_COLOR: Color = Color::new(0.18, 0.18, 0.18, 1.00);
const BOARD_COLOR: Color = Color::new(0.43, 0.39, 0.35, 1.00);
const EMPTY_TILE_COLOR: Color = Color::new(0.78, 0.73, 0.68, 0.35);
const TILE_COLORS: [Color; 12] = [
    Color::new(0.93, 0.89, 0.85, 1.00),
    Color::new(0.93, 0.88, 0.78, 1.00), 
    Color::new(0.95, 0.69, 0.47, 1.00), 
    Color::new(0.96, 0.58, 0.39, 1.00), 
    Color::new(0.96, 0.49, 0.37, 1.00), 
    Color::new(0.96, 0.37, 0.23, 1.00), 
    Color::new(0.93, 0.81, 0.45, 1.00), 
    Color::new(0.93, 0.80, 0.38, 1.00), 
    Color::new(0.93, 0.78, 0.31, 1.00), 
    Color::new(0.93, 0.77, 0.25, 1.00), 
    Color::new(0.93, 0.76, 0.18, 1.00), 
    Color::new(0.24, 0.24, 0.24, 1.00), 
];

#[derive(Clone, Copy)]
struct Tile {
    value: u32,
    grid_pos: (usize, usize),
    anim_pos: (f32, f32),
    merging: bool,
}

impl Tile {
    fn new(value: u32, grid_pos: (usize, usize)) -> Self {
        let anim_pos = (
            BOARD_MARGIN + grid_pos.1 as f32 * (TILE_SIZE + TILE_MARGIN),
            BOARD_MARGIN + grid_pos.0 as f32 * (TILE_SIZE + TILE_MARGIN),
        );
        Self {
            value,
            grid_pos,
            anim_pos,
            merging: false,
        }
    }
}

pub struct MathQuestion {
    board: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
    score: u32,
    best_score: u32,
    game_over: bool,
    won: bool,
    moving: bool,
    move_cooldown: f32,
    pending_move: Option<(i32, i32)>,
    has_unlocked_hacker: bool,
    pub hacker_mode: Option<HackerMode>,  // Add this line
}

impl MathQuestion {
    pub fn new(best_score: u32) -> Self {
        let mut game = Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            score: 0,
            best_score,
            game_over: false,
            won: false,
            moving: false,
            move_cooldown: 0.0,
            pending_move: None,
            has_unlocked_hacker: best_score >= 2048,
            hacker_mode: None,
        };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    fn add_random_tile(&mut self) {
        let mut empty_cells = Vec::new();
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if self.board[i][j].is_none() {
                    empty_cells.push((i, j));
                }
            }
        }
        if !empty_cells.is_empty() {
            let (i, j) = empty_cells[rand::gen_range(0, empty_cells.len())];
            self.board[i][j] = Some(Tile::new(if rand::gen_range(0, 10) < 9 { 2 } else { 4 }, (i, j)));
        }
    }

    pub fn update(&mut self) -> bool {
        let delta_time = get_frame_time();

        if let Some(hacker_mode) = &mut self.hacker_mode {
            hacker_mode.update();
            
            // Check for escape to exit hacker mode
            return false;
        }

        if self.move_cooldown > 0.0 {
            self.move_cooldown -= delta_time;
        }

        if !self.moving && self.move_cooldown <= 0.0 && !self.game_over {
            if is_key_pressed(KeyCode::Up) {
                self.move_tiles((-1, 0));
            } else if is_key_pressed(KeyCode::Down) {
                self.move_tiles((1, 0));
            } else if is_key_pressed(KeyCode::Left) {
                self.move_tiles((0, -1));
            } else if is_key_pressed(KeyCode::Right) {
                self.move_tiles((0, 1));
            }
        }

        if is_key_pressed(KeyCode::R) {
            self.reset();
        }

        if is_key_pressed(KeyCode::Escape) {
            return true;
        }

        if self.has_unlocked_hacker
            && is_key_down(KeyCode::LeftControl)
            && is_key_pressed(KeyCode::H)
        {
            self.hacker_mode = Some(HackerMode::default());
        }

        self.update_animation(delta_time);
        false
    }
    pub fn draw(&mut self) {
        clear_background(BG_COLOR);
        match &mut self.hacker_mode {
        Some(hm) => hm.draw_hacker_ui(),
        None => {
            self.draw_board();
            self.draw_ui();
        }
    }
    }

    fn move_tiles(&mut self, direction: (i32, i32)) -> bool {
        if self.moving {
            self.pending_move = Some(direction);
            return false;
        }
        let mut moved = false;
        let mut merged = [[false; BOARD_SIZE]; BOARD_SIZE];

        let mut order: Vec<(usize, usize)> = (0..BOARD_SIZE)
            .flat_map(|i| (0..BOARD_SIZE).map(move |j| (i, j)))
            .collect();

        match direction {
            (-1, 0) => order.sort_by_key(|&(i, _)| i),            
            (1, 0) => order.sort_by_key(|&(i, _)| BOARD_SIZE - i),
            (0, -1) => order.sort_by_key(|&(_, j)| j),             
            (0, 1) => order.sort_by_key(|&(_, j)| BOARD_SIZE - j), 
            _ => {}
        }

        let mut new_board: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE] = Default::default();

        for &(i, j) in &order {
            if let Some(mut tile) = self.board[i][j].clone() {
                let (mut x, mut y) = (i as i32, j as i32);
                loop {
                    let nx = x + direction.0;
                    let ny = y + direction.1;
                    if nx < 0 || nx >= BOARD_SIZE as i32 || ny < 0 || ny >= BOARD_SIZE as i32 {
                        break;
                    }
                    match &mut new_board[nx as usize][ny as usize] {
                        None => {
                            x = nx;
                            y = ny;
                        }
                        Some(other) if other.value == tile.value && !merged[nx as usize][ny as usize] => {
                            other.value *= 2;
                            other.merging = true;
                            self.score += other.value;
                            if other.value == 2048 && !self.won {
                                self.won = true;
                                self.has_unlocked_hacker = true;
                            }
                            merged[nx as usize][ny as usize] = true;
                            moved = true;
                            tile = other.clone(); 
                            break;
                        }
                        _ => break,
                    }
                }
                if x != i as i32 || y != j as i32 {
                    moved = true;
                }
                tile.grid_pos = (x as usize, y as usize);
                tile.anim_pos = (
                    BOARD_MARGIN + j as f32 * (TILE_SIZE + TILE_MARGIN),
                    BOARD_MARGIN + i as f32 * (TILE_SIZE + TILE_MARGIN),
                );
                new_board[x as usize][y as usize] = Some(tile);
            }
        }

        if moved {
            self.board = new_board;
            self.add_random_tile();
            self.moving = true;
            self.move_cooldown = MOVE_COOLDOWN;
            if !self.can_move() {
                self.game_over = true;
            }
            if self.score > self.best_score {
                self.best_score = self.score;
            }
        }

        if self.score > self.best_score {
            self.best_score = self.score;
            if self.best_score >= 2048 {
                self.has_unlocked_hacker = true;
            }
        }

        moved
    }

    fn can_move(&self) -> bool {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if self.board[i][j].is_none() {
                    return true;
                }
            }
        }
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if let Some(tile) = &self.board[i][j] {
                    if i + 1 < BOARD_SIZE {
                        if let Some(other) = &self.board[i + 1][j] {
                            if other.value == tile.value {
                                return true;
                            }
                        }
                    }
                    if j + 1 < BOARD_SIZE {
                        if let Some(other) = &self.board[i][j + 1] {
                            if other.value == tile.value {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn reset(&mut self) {
        let best = self.best_score;
        *self = MathQuestion::new(best);
    }

    fn update_animation(&mut self, delta_time: f32) {
        if self.moving {
            let mut all_done = true;
            for i in 0..BOARD_SIZE {
                for j in 0..BOARD_SIZE {
                    if let Some(tile) = &mut self.board[i][j] {
                        let (target_x, target_y) = (
                            BOARD_MARGIN + j as f32 * (TILE_SIZE + TILE_MARGIN),
                            BOARD_MARGIN + i as f32 * (TILE_SIZE + TILE_MARGIN),
                        );
                        let dx = target_x - tile.anim_pos.0;
                        let dy = target_y - tile.anim_pos.1;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 1.0 {
                            tile.anim_pos.0 += dx * ANIMATION_SPEED * delta_time;
                            tile.anim_pos.1 += dy * ANIMATION_SPEED * delta_time;
                            all_done = false;
                        } else {
                            tile.anim_pos.0 = target_x;
                            tile.anim_pos.1 = target_y;
                        }
                        tile.merging = false;
                    }
                }
            }
            if all_done {
                self.moving = false;
                if let Some(dir) = self.pending_move.take() {
                    self.move_tiles(dir);
                }
            }
        }
    }

    fn draw_board(&self) {
        let board_width = BOARD_SIZE as f32 * TILE_SIZE + (BOARD_SIZE as f32 + 1.0) * TILE_MARGIN;
        draw_rectangle(
            BOARD_MARGIN - TILE_MARGIN,
            BOARD_MARGIN - TILE_MARGIN,
            board_width,
            board_width,
            BOARD_COLOR,
        );

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let x = BOARD_MARGIN + j as f32 * (TILE_SIZE + TILE_MARGIN);
                let y = BOARD_MARGIN + i as f32 * (TILE_SIZE + TILE_MARGIN);
                draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, EMPTY_TILE_COLOR);
            }
        }

        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                if let Some(tile) = &self.board[i][j] {
                    self.draw_tile(tile);
                }
            }
        }
    }

    fn draw_tile(&self, tile: &Tile) {
        let value = tile.value;
        let x = tile.anim_pos.0;
        let y = tile.anim_pos.1;
        let color = get_tile_color(value);
        
        draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, color);

        let text = value.to_string();
        let font_size = match text.len() {
            1 => 40.0,
            2 => 36.0,
            3 => 30.0,
            _ => 24.0,
        };

        let text_dim = measure_text(&text, None, font_size as u16, 1.0);
        draw_text(
            &text,
            x + (TILE_SIZE - text_dim.width) / 2.0,
            y + (TILE_SIZE + text_dim.height) / 2.0,
            font_size,
            get_tile_text_color(value),
        );
    }

    fn draw_ui(&self) {
        let board_width = BOARD_SIZE as f32 * TILE_SIZE + (BOARD_SIZE as f32 + 1.0) * TILE_MARGIN;
        let ui_y = BOARD_MARGIN * 2.0 + board_width;

        let score_text = format!("Score: {}", self.score);
        let best_score_text = format!("Best: {}", self.best_score);

        draw_text(
            &score_text,
            BOARD_MARGIN,
            ui_y,
            24.0,
            Color::new(0.93, 0.89, 0.85, 1.00),
        );

        draw_text(
            &best_score_text,
            BOARD_MARGIN,
            ui_y + 30.0,
            20.0,
            Color::new(0.93, 0.89, 0.85, 0.7),
        );

        if self.game_over {
            let message = if self.won {
                "You Win! Press R to restart"
            } else {
                "Game Over! Press R to restart"
            };

            let font_size = 30.0;
            let text_dim = measure_text(message, None, font_size as u16, 1.0);
            let x = (screen_width() - text_dim.width) / 2.0;
            let y = (screen_height() - text_dim.height) / 2.0;

            draw_rectangle(
                x - 20.0,
                y - 20.0,
                text_dim.width + 40.0,
                text_dim.height + 40.0,
                Color::new(0.18, 0.18, 0.18, 0.9),
            );

            draw_text(
                message,
                x,
                y + text_dim.height,
                font_size,
                Color::new(1.0, 1.0, 1.0, 1.0),
            );
        }
    }
}

fn get_tile_color(value: u32) -> Color {
    let index = match value {
        2 => 0,
        4 => 1,
        8 => 2,
        16 => 3,
        32 => 4,
        64 => 5,
        128 => 6,
        256 => 7,
        512 => 8,
        1024 => 9,
        2048 => 10,
        _ => 11,
    };
    TILE_COLORS[index]
}

fn get_tile_text_color(value: u32) -> Color {
    if value <= 4 {
        Color::new(0.47, 0.43, 0.39, 1.00)
    } else {
        Color::new(0.98, 0.96, 0.93, 1.00)
    }
}

#[macroquad::main("Math Game")]
async fn main() {
    let mut math_game = MathQuestion::new(0);
    
    loop {
        if math_game.update() {
            break;
        }
        
        math_game.draw();  // This now requires math_game to be mutable
        
        next_frame().await;
    }
}