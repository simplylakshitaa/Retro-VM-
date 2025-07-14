use macroquad::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::collections::HashMap;
use std::process::Command;

const BOARD_WIDTH: f32 = 500.0;
const BOARD_HEIGHT: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameScreen {
    GameMenu,
    NewGame,
    PreviousGame,
    GameSettings,
    ThemeSettings,
    Exit,
}

#[derive(Debug, Clone)]
struct PieceTextures {
    white_pawn: Texture2D,
    white_knight: Texture2D,
    white_bishop: Texture2D,
    white_rook: Texture2D,
    white_queen: Texture2D,
    white_king: Texture2D,
    black_pawn: Texture2D,
    black_knight: Texture2D,
    black_bishop: Texture2D,
    black_rook: Texture2D,
    black_queen: Texture2D,
    black_king: Texture2D,
}
#[derive(Debug, Clone)]
pub struct ChessGameState {
    board: [[i32; 8]; 8],
    selected_square_row: i32,
    selected_square_col: i32,
    is_white_turn: bool,
    en_passant_target_row: i32,
    en_passant_target_col: i32,
    white_king_moved: bool,
    white_rook_kingside_moved: bool,
    white_rook_queenside_moved: bool,
    black_king_moved: bool,
    black_rook_kingside_moved: bool,
    black_rook_queenside_moved: bool,
    promotion_active: bool,
    promotion_row: i32,
    promotion_col: i32,
    is_white_promoting: bool,
    game_over: bool,
    promotion_pending: bool,
}

impl Default for ChessGameState {
    fn default() -> Self {
        ChessGameState {
            board: [
                [-4, -2, -3, -5, -6, -3, -2, -4],
                [-1, -1, -1, -1, -1, -1, -1, -1],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [1, 1, 1, 1, 1, 1, 1, 1],
                [4, 2, 3, 5, 6, 3, 2, 4],
            ],
            selected_square_row: -1,
            selected_square_col: -1,
            is_white_turn: true,
            en_passant_target_row: -1,
            en_passant_target_col: -1,
            white_king_moved: false,
            white_rook_kingside_moved: false,
            white_rook_queenside_moved: false,
            black_king_moved: false,
            black_rook_kingside_moved: false,
            black_rook_queenside_moved: false,
            promotion_active: false,
            promotion_row: 0,
            promotion_col: 0,
            is_white_promoting: false,
            game_over: false,
            promotion_pending: false,
        }
    }
}

impl ChessGameState {
    fn reset(&mut self) {
        *self = ChessGameState::default();
    }

    fn save(&self) -> std::io::Result<()> {
        let mut file = File::create("saved_game.dat")?;
        
        for row in &self.board {
            for &piece in row {
                file.write_all(&piece.to_le_bytes())?;
            }
        }
        
        file.write_all(&[self.is_white_turn as u8])?;
        file.write_all(&(self.en_passant_target_row as i32).to_le_bytes())?;
        file.write_all(&(self.en_passant_target_col as i32).to_le_bytes())?;
        file.write_all(&[self.white_king_moved as u8])?;
        file.write_all(&[self.white_rook_kingside_moved as u8])?;
        file.write_all(&[self.white_rook_queenside_moved as u8])?;
        file.write_all(&[self.black_king_moved as u8])?;
        file.write_all(&[self.black_rook_kingside_moved as u8])?;
        file.write_all(&[self.black_rook_queenside_moved as u8])?;
        
        Ok(())
    }

    fn load(&mut self) -> std::io::Result<()> {
        if !Path::new("saved_game.dat").exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
        }
        
        let mut file = File::open("saved_game.dat")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        let mut board = [[0; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let index = (i * 8 + j) * 4;
                board[i][j] = i32::from_le_bytes([
                    buffer[index],
                    buffer[index + 1],
                    buffer[index + 2],
                    buffer[index + 3],
                ]);
            }
        }
        
        let mut offset = 64 * 4;
        self.is_white_turn = buffer[offset] != 0;
        offset += 1;
        
        self.en_passant_target_row = i32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;
        
        self.en_passant_target_col = i32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;
        
        self.white_king_moved = buffer[offset] != 0;
        offset += 1;
        
        self.white_rook_kingside_moved = buffer[offset] != 0;
        offset += 1;
        
        self.white_rook_queenside_moved = buffer[offset] != 0;
        offset += 1;
        
        self.black_king_moved = buffer[offset] != 0;
        offset += 1;
        
        self.black_rook_kingside_moved = buffer[offset] != 0;
        offset += 1;
        
        self.black_rook_queenside_moved = buffer[offset] != 0;
        
        self.board = board;
        self.selected_square_row = -1;
        self.selected_square_col = -1;
        self.promotion_active = false;
        self.promotion_pending = false;
        self.game_over = false;
        
        Ok(())
    }
}

pub struct ChessGameResources {
    textures: PieceTextures,
    background_image: Texture2D,
    menu_background: Texture2D,
    board_themes: Vec<(String, Texture2D)>,
    current_board_theme: usize,
    highlight_legal_moves: bool,
    music_volume: f32,
    sound_volume: f32,
}

impl ChessGameResources {
    pub async fn new() -> Self {
        let textures = PieceTextures {
            white_pawn: load_texture("assets/white_pawn.png").await.unwrap(),
            white_knight: load_texture("assets/white_knight.png").await.unwrap(),
            white_bishop: load_texture("assets/white_bishop.png").await.unwrap(),
            white_rook: load_texture("assets/white_rook.png").await.unwrap(),
            white_queen: load_texture("assets/white_queen.png").await.unwrap(),
            white_king: load_texture("assets/white_king.png").await.unwrap(),
            black_pawn: load_texture("assets/black_pawn.png").await.unwrap(),
            black_knight: load_texture("assets/black_knight.png").await.unwrap(),
            black_bishop: load_texture("assets/black_bishop.png").await.unwrap(),
            black_rook: load_texture("assets/black_rook.png").await.unwrap(),
            black_queen: load_texture("assets/black_queen.png").await.unwrap(),
            black_king: load_texture("assets/black_king.png").await.unwrap(),
        };

        let background_image = load_texture("assets/chess_background.png").await.unwrap();
        let menu_background = load_texture("assets/menuBackground.png").await.unwrap();

        let board_themes = vec![
            ("Classic".to_string(), load_texture("assets/chessboard.png").await.unwrap()),
        ];

        ChessGameResources {
            textures,
            background_image,
            menu_background,
            board_themes,
            current_board_theme: 0,
            highlight_legal_moves: true,
            music_volume: 1.0,
            sound_volume: 1.0,
        }
    }
}

pub struct ChessGame {
    resources: ChessGameResources,
    game_state: ChessGameState,
    current_screen: GameScreen,
    using_keyboard: bool,
    selected_button: usize,
    key_released: bool,
}


impl ChessGame {
    pub async fn new() -> Self {
        Self {
            resources: ChessGameResources::new().await,
            game_state: ChessGameState::default(),
            current_screen: GameScreen::GameMenu,
            using_keyboard: false,
            selected_button: 0,
            key_released: true,
        }
    }

    pub fn update(&mut self) -> GameStatus {
        // Check game state first
        if is_checkmate(&self.game_state.board, !self.game_state.is_white_turn, &self.game_state) {
            self.game_state.game_over = true;
            return GameStatus::Checkmate;
        }

        // Handle global key shortcuts
        if is_key_pressed(KeyCode::Y) && self.current_screen != GameScreen::GameMenu {
            self.current_screen = GameScreen::GameMenu;
            return GameStatus::Playing;
        }

        // Handle current screen logic
            match self.current_screen {
                GameScreen::GameMenu => {
                let buttons = [
                    Rect::new(screen_width() * 0.1, screen_height() * 0.4, screen_width() * 0.15, screen_height() * 0.075),
                    Rect::new(screen_width() * 0.1, screen_height() * 0.4 + screen_height() * 0.075 + screen_height() * 0.028, screen_width() * 0.15, screen_height() * 0.075),
                    Rect::new(screen_width() * 0.1, screen_height() * 0.4 + 2.0 * (screen_height() * 0.075 + screen_height() * 0.028), screen_width() * 0.15, screen_height() * 0.075),
                    Rect::new(screen_width() * 0.1, screen_height() * 0.4 + 3.0 * (screen_height() * 0.075 + screen_height() * 0.028), screen_width() * 0.15, screen_height() * 0.075),
                    Rect::new(screen_width() * 0.1, screen_height() * 0.4 + 4.0 * (screen_height() * 0.075 + screen_height() * 0.028), screen_width() * 0.15, screen_height() * 0.075),
                ];

                // Handle mouse/keyboard navigation
                let mouse_pos = mouse_position();
                unsafe {
                    static mut PREV_MOUSE_POS: (f32, f32) = (0.0, 0.0);
                    if mouse_pos.0 != PREV_MOUSE_POS.0 || mouse_pos.1 != PREV_MOUSE_POS.1 {
                        self.using_keyboard = false;
                        PREV_MOUSE_POS = mouse_pos;
                    }
                }

                // Keyboard selection
                if is_key_down(KeyCode::Down) && self.key_released {
                    self.selected_button = (self.selected_button + 1) % 5;
                    self.key_released = false;
                    self.using_keyboard = true;
                } else if is_key_down(KeyCode::Up) && self.key_released {
                    self.selected_button = (self.selected_button - 1 + 5) % 5;
                    self.key_released = false;
                    self.using_keyboard = true;
                } else if !is_key_down(KeyCode::Up) && !is_key_down(KeyCode::Down) {
                    self.key_released = true;
                }

                // Handle selections
                if is_key_pressed(KeyCode::Enter) || 
                (is_mouse_button_pressed(MouseButton::Left)) && 
                buttons.iter().enumerate().any(|(i, btn)| {
                    if is_point_in_rect(mouse_pos.0, mouse_pos.1, btn.x, btn.y, btn.w, btn.h) {
                        self.selected_button = i;
                        true
                    } else {
                        false
                    }
                }) 
                {
                    match self.selected_button {
                        0 => {
                            self.current_screen = GameScreen::NewGame;
                            self.game_state.reset();
                        }
                        1 => if self.game_state.load().is_ok() {
                            self.current_screen = GameScreen::NewGame;
                        },
                        2 => self.current_screen = GameScreen::ThemeSettings,
                        3 => self.current_screen = GameScreen::GameSettings,
                        4 => return GameStatus::Exit,
                        _ => {}
                    }
                }
            },
            
            GameScreen::NewGame => {
                self.handle_piece_movement();
                
                // Handle save game button
                let save_btn = Rect::new(screen_width() - 150.0, 20.0, 120.0, 40.0);
                if is_point_in_rect(mouse_position().0, mouse_position().1, save_btn.x, save_btn.y, save_btn.w, save_btn.h) 
                    && is_mouse_button_pressed(MouseButton::Left) 
                {
                    // Save the game
                    if let Err(e) = self.game_state.save() {
                        eprintln!("Failed to save game: {}", e);
                    }
                    
                    // Open the module.txt file
                    let txt_path = Path::new("assets/module.txt");
                    if txt_path.exists() {
                        let result = if cfg!(target_os = "windows") {
                            Command::new("cmd")
                                .args(&["/C", "start", "", txt_path.to_str().unwrap()])
                                .spawn()
                        } else if cfg!(target_os = "macos") {
                            Command::new("open")
                                .arg(txt_path)
                                .spawn()
                        } else {
                            Command::new("xdg-open")
                                .arg(txt_path)
                                .spawn()
                        };
                        
                        if let Err(e) = result {
                            eprintln!("Failed to open module.txt: {}", e);
                        }
                    } else {
                        eprintln!("module.txt not found at: {}", txt_path.display());
                    }
                }
            },
            GameScreen::PreviousGame => {
                // Add handling for PreviousGame screen
                self.handle_piece_movement();
                
                // You might want to add a "Return to Menu" button or similar
                let back_btn = Rect::new(20.0, 20.0, 120.0, 40.0);
                if is_point_in_rect(mouse_position().0, mouse_position().1, back_btn.x, back_btn.y, back_btn.w, back_btn.h) 
                    && is_mouse_button_pressed(MouseButton::Left) 
                {
                    self.current_screen = GameScreen::GameMenu;
                }
            },
            GameScreen::GameSettings => {
                self.current_screen = self.draw_game_settings();
            },
            
            GameScreen::ThemeSettings => {
                self.current_screen = self.draw_theme_settings();
            },
            
            GameScreen::Exit => {
                return GameStatus::Exit;
            }
        }

        GameStatus::Playing
    }

    pub fn draw(&self) {
        match self.current_screen {
            GameScreen::GameMenu => {
                self.draw_menu();
            }
            GameScreen::NewGame => {
                self.draw_game();
                
                let save_button = Rect::new(
                    screen_width() - 150.0,
                    20.0,
                    120.0,
                    40.0,
                );
                let save_hovered = is_point_in_rect(mouse_position().0, mouse_position().1, save_button.x, save_button.y, save_button.w, save_button.h);

                draw_rectangle(save_button.x, save_button.y, save_button.w, save_button.h, if save_hovered { LIGHTGRAY } else { GRAY });
                if save_hovered {
                    draw_rectangle_lines(save_button.x, save_button.y, save_button.w, save_button.h, 2.0, GOLD);
                }
                draw_text(
                    "Save Game", 
                    save_button.x + 10.0, 
                    save_button.y + 10.0, 
                    20.0,              
                    BLACK
                );
            }
            GameScreen::GameSettings => {
                // Drawing handled in draw_game_settings
            }
            GameScreen::ThemeSettings => {
                // Drawing handled in draw_theme_settings
            }
            _ => {}
        }
    }

    fn handle_piece_movement(&mut self) {
        let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
        let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_pos = mouse_position();

            if mouse_pos.0 >= board_offset_x
                && mouse_pos.0 < (board_offset_x + BOARD_WIDTH)
                && mouse_pos.1 >= board_offset_y
                && mouse_pos.1 < (board_offset_y + BOARD_HEIGHT)
            {
                let row = ((mouse_pos.1 - board_offset_y) / 62.5) as i32;
                let col = ((mouse_pos.0 - board_offset_x) / 62.5) as i32;

                if self.game_state.selected_square_row == -1 && self.game_state.selected_square_col == -1 {
                    if (self.game_state.is_white_turn && self.game_state.board[row as usize][col as usize] > 0)
                        || (!self.game_state.is_white_turn && self.game_state.board[row as usize][col as usize] < 0)
                    {
                        self.game_state.selected_square_row = row;
                        self.game_state.selected_square_col = col;
                    }
                }
                else {
                    if self.game_state.board[row as usize][col as usize] != 0
                        && self.game_state.board[row as usize][col as usize]
                            * self.game_state.board[self.game_state.selected_square_row as usize]
                                [self.game_state.selected_square_col as usize]
                            > 0
                    {
                        self.game_state.selected_square_row = row;
                        self.game_state.selected_square_col = col;
                    } else {
                        if is_valid_move(
                            &self.game_state.board,
                            self.game_state.board[self.game_state.selected_square_row as usize]
                                [self.game_state.selected_square_col as usize],
                            self.game_state.selected_square_row,
                            self.game_state.selected_square_col,
                            row,
                            col,
                            &self.game_state,
                            true,
                        ) {
                            let piece = self.game_state.board[self.game_state.selected_square_row as usize]
                                [self.game_state.selected_square_col as usize];
                            let is_white = piece > 0;

                            if piece.abs() == 6 && (self.game_state.selected_square_col - col).abs() == 2 {
                                let kingside = col > self.game_state.selected_square_col;
                                let rook_col = if kingside { 7 } else { 0 };
                                let new_rook_col = if kingside { 5 } else { 3 };

                                self.game_state.board[row as usize][new_rook_col as usize] =
                                    self.game_state.board[row as usize][rook_col as usize];
                                self.game_state.board[row as usize][rook_col as usize] = 0;
                            }

                            if piece.abs() == 1
                                && col != self.game_state.selected_square_col
                                && self.game_state.board[row as usize][col as usize] == 0
                            {
                                self.game_state.board[self.game_state.selected_square_row as usize]
                                    [col as usize] = 0;
                            }

                            let original_piece = self.game_state.board[row as usize][col as usize];
                            self.game_state.board[row as usize][col as usize] = piece;
                            self.game_state.board[self.game_state.selected_square_row as usize]
                                [self.game_state.selected_square_col as usize] = 0;

                            if piece.abs() == 1 && (row - self.game_state.selected_square_row).abs() == 2 {
                                self.game_state.en_passant_target_row =
                                    (row + self.game_state.selected_square_row) / 2;
                                self.game_state.en_passant_target_col = col;
                            } else {
                                self.game_state.en_passant_target_row = -1;
                                self.game_state.en_passant_target_col = -1;
                            }

                            if piece.abs() == 6 {
                                if is_white {
                                    self.game_state.white_king_moved = true;
                                } else {
                                    self.game_state.black_king_moved = true;
                                }
                            }
                            if piece.abs() == 4 {
                                if is_white {
                                    if self.game_state.selected_square_col == 0 {
                                        self.game_state.white_rook_queenside_moved = true;
                                    }
                                    if self.game_state.selected_square_col == 7 {
                                        self.game_state.white_rook_kingside_moved = true;
                                    }
                                } else {
                                    if self.game_state.selected_square_col == 0 {
                                        self.game_state.black_rook_queenside_moved = true;
                                    }
                                    if self.game_state.selected_square_col == 7 {
                                        self.game_state.black_rook_kingside_moved = true;
                                    }
                                }
                            }

                            if piece.abs() == 1 && (row == 0 || row == 7) {
                                self.game_state.promotion_pending = true;
                                self.game_state.promotion_row = row;
                                self.game_state.promotion_col = col;
                                self.game_state.is_white_promoting = is_white;
                                return;
                            }

                            if is_king_in_check(&self.game_state.board, self.game_state.is_white_turn, &self.game_state) {
                                self.game_state.board[self.game_state.selected_square_row as usize]
                                    [self.game_state.selected_square_col as usize] = piece;
                                self.game_state.board[row as usize][col as usize] = original_piece;
                            } else {
                                self.game_state.is_white_turn = !self.game_state.is_white_turn;
                            }

                            if is_king_in_check(&self.game_state.board, !self.game_state.is_white_turn, &self.game_state) {
                                if is_checkmate(&self.game_state.board, !self.game_state.is_white_turn, &self.game_state) {
                                    self.game_state.game_over = true;
                                }
                            }
                        }
                        self.game_state.selected_square_row = -1;
                        self.game_state.selected_square_col = -1;
                    }
                }
            }
        }

        if self.game_state.promotion_pending && is_mouse_button_pressed(MouseButton::Left) {
            let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
            let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

            let menu_width = 250.0;
            let menu_height = 80.0;
            let menu_x = board_offset_x + (BOARD_WIDTH - menu_width) / 2.0;
            let menu_y = board_offset_y + (BOARD_HEIGHT - menu_height - 10.0) / 2.0;

            let mouse_pos = mouse_position();

            for i in 0..4 {
                let btn_x = menu_x + 20.0 + i as f32 * 55.0;
                let btn_y = menu_y + 20.0;

                if is_point_in_rect(mouse_pos.0, mouse_pos.1, btn_x, btn_y, 50.0, 50.0) {
                    self.game_state.board[self.game_state.promotion_row as usize][self.game_state.promotion_col as usize] =
                        if self.game_state.is_white_promoting {
                            5 - i as i32
                        } else {
                            -(5 - i as i32)
                        };
                    self.game_state.promotion_pending = false;

                    if is_king_in_check(&self.game_state.board, self.game_state.is_white_turn, &self.game_state) {
                        self.game_state.board[self.game_state.promotion_row as usize][self.game_state.promotion_col as usize] = 
                            if self.game_state.is_white_promoting { 1 } else { -1 };
                        self.game_state.promotion_pending = true;
                    } else {
                        self.game_state.is_white_turn = !self.game_state.is_white_turn;

                        if is_king_in_check(&self.game_state.board, !self.game_state.is_white_turn, &self.game_state) {
                            if is_checkmate(&self.game_state.board, !self.game_state.is_white_turn, &self.game_state) {
                                self.game_state.game_over = true;
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    fn draw_menu(&self) {
        draw_texture_ex(
            &self.resources.menu_background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        let button_width = screen_width() * 0.15;
        let button_height = screen_height() * 0.075;
        let gap = screen_height() * 0.028;
        let button_x = screen_width() * 0.1;
        let total_button_stack_height = (5.0 * button_height) + (4.0 * gap);
        let title_space = screen_height() * 0.12;
        let start_y = ((screen_height() - total_button_stack_height) / 2.0) + title_space;

        let buttons = [
            Rect::new(button_x, start_y, button_width, button_height),
            Rect::new(button_x, start_y + button_height + gap, button_width, button_height),
            Rect::new(button_x, start_y + 2.0 * (button_height + gap), button_width, button_height),
            Rect::new(button_x, start_y + 3.0 * (button_height + gap), button_width, button_height),
            Rect::new(button_x, start_y + 4.0 * (button_height + gap), button_width, button_height),
        ];

        let button_texts = ["New Game", "Previous Game", "Theme Settings", "Game Settings", "Exit"];

        draw_text_ex(
            "CHESS",
            button_x,
            start_y - title_space - 100.0,
            TextParams {
                font_size: 100,
                color: WHITE,
                ..Default::default()
            },
        );

        for (i, button) in buttons.iter().enumerate() {
            let is_selected = (i == self.selected_button) && self.using_keyboard;
            let is_hovered = is_point_in_rect(
                mouse_position().0, 
                mouse_position().1, 
                button.x, 
                button.y, 
                button.w, 
                button.h
            ) && !self.using_keyboard;

            if is_selected || is_hovered {
                draw_rectangle_lines(button.x, button.y, button.w, button.h, 2.0, WHITE);

                let text_x = button.x + 20.0;
                let text_y = button.y + button.h / 4.0;
                let text = button_texts[i];
                let font_size = 20.0;

                draw_text(text, text_x - 1.0, text_y, font_size, WHITE);
                draw_text(text, text_x + 1.0, text_y, font_size, WHITE);
                draw_text(text, text_x, text_y - 1.0, font_size, WHITE);
                draw_text(text, text_x, text_y + 1.0, font_size, WHITE);
                draw_text(text, text_x, text_y, font_size, BLACK);
            } else {
                draw_text(
                    button_texts[i],
                    button.x + 20.0,
                    button.y + button.h / 4.0,
                    20.0,
                    WHITE,
                );
            }
        }
    }

    fn draw_game(&self) {
        draw_texture_ex(
            &self.resources.background_image,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        self.draw_chess_board();
        self.draw_pieces();

        if self.game_state.selected_square_row != -1 && self.game_state.selected_square_col != -1 {
            self.draw_valid_moves(
                self.game_state.board[self.game_state.selected_square_row as usize][self.game_state.selected_square_col as usize],
                self.game_state.selected_square_row,
                self.game_state.selected_square_col,
            );
        }

        if self.game_state.promotion_pending {
            self.draw_promotion_menu();
        }

        if self.game_state.game_over {
            draw_text(
                "Checkmate! Game Over.",
                screen_width() / 2.0 - 150.0,
                screen_height() / 2.0 - 20.0,
                30.0,
                RED,
            );
        }
    }

    fn draw_chess_board(&self) {
        let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
        let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

        if !self.resources.board_themes.is_empty() {
            let current_board = &self.resources.board_themes[self.resources.current_board_theme].1;
            draw_texture_ex(
                current_board,
                board_offset_x,
                board_offset_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(BOARD_WIDTH, BOARD_HEIGHT)),
                    ..Default::default()
                },
            );
        } else {
            for row in 0..8 {
                for col in 0..8 {
                    let color = if (row + col) % 2 == 0 {
                        BEIGE
                    } else {
                        DARKBROWN
                    };
                    draw_rectangle(
                        (col as f32 * 62.5 + board_offset_x),
                        (row as f32 * 62.5 + board_offset_y),
                        62.5,
                        62.5,
                        color,
                    );
                }
            }
        }

        if is_king_in_check(&self.game_state.board, self.game_state.is_white_turn, &self.game_state) {
            let mut king_row = -1;
            let mut king_col = -1;
            let king_piece = if self.game_state.is_white_turn { 6 } else { -6 };

            for row in 0..8 {
                for col in 0..8 {
                    if self.game_state.board[row as usize][col as usize] == king_piece {
                        king_row = row as i32;
                        king_col = col as i32;
                        break;
                    }
                }
                if king_row != -1 {
                    break;
                }
            }

            if king_row != -1 && king_col != -1 {
                draw_rectangle(
                    (king_col as f32 * 62.5 + board_offset_x),
                    (king_row as f32 * 62.5 + board_offset_y),
                    62.5,
                    62.5,
                    Color::new(255.0, 0.0, 0.0, 0.5),
                );
            }
        }
    }

    fn draw_pieces(&self) {
        let square_size = BOARD_WIDTH / 8.0;
        let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
        let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

        for row in 0..8 {
            for col in 0..8 {
                let piece = self.game_state.board[row as usize][col as usize];
                if piece == 0 {
                    continue;
                }

                let piece_texture = match piece {
                    1 => &self.resources.textures.white_pawn,
                    2 => &self.resources.textures.white_knight,
                    3 => &self.resources.textures.white_bishop,
                    4 => &self.resources.textures.white_rook,
                    5 => &self.resources.textures.white_queen,
                    6 => &self.resources.textures.white_king,
                    -1 => &self.resources.textures.black_pawn,
                    -2 => &self.resources.textures.black_knight,
                    -3 => &self.resources.textures.black_bishop,
                    -4 => &self.resources.textures.black_rook,
                    -5 => &self.resources.textures.black_queen,
                    -6 => &self.resources.textures.black_king,
                    _ => continue,
                };

                let x = board_offset_x + col as f32 * square_size + (square_size - piece_texture.width() * 0.45) / 2.0;
                let y = board_offset_y + row as f32 * square_size + (square_size - piece_texture.height() * 0.45) / 2.0;

                draw_texture_ex(
                    piece_texture,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(piece_texture.width() * 0.45, piece_texture.height() * 0.45)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_promotion_menu(&self) {
        let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
        let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));

        let menu_width = 250.0;
        let menu_height = 80.0;
        let menu_x = board_offset_x + (BOARD_WIDTH - menu_width) / 2.0;
        let menu_y = board_offset_y + (BOARD_HEIGHT - menu_height - 10.0) / 2.0;
        draw_rectangle(menu_x, menu_y, menu_width, menu_height, LIGHTGRAY);

        let pieces = if self.game_state.is_white_promoting {
            [
                &self.resources.textures.white_queen,
                &self.resources.textures.white_rook,
                &self.resources.textures.white_bishop,
                &self.resources.textures.white_knight,
            ]
        } else {
            [
                &self.resources.textures.black_queen,
                &self.resources.textures.black_rook,
                &self.resources.textures.black_bishop,
                &self.resources.textures.black_knight,
            ]
        };

        let labels = ["Q", "R", "B", "K"];

        for i in 0..4 {
            let btn_x = menu_x + 20.0 + i as f32 * 55.0;
            let btn_y = menu_y + 20.0;

            let mouse_pos = mouse_position();
            let btn_color = if is_point_in_rect(mouse_pos.0, mouse_pos.1, btn_x, btn_y, 50.0, 50.0) {
                LIGHTGRAY
            } else {
                WHITE
            };

            draw_rectangle(btn_x, btn_y, 50.0, 50.0, btn_color);
            draw_texture_ex(
                pieces[i],
                btn_x,
                btn_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(50.0, 50.0)),
                    ..Default::default()
                },
            );
            draw_text(
                labels[i], 
                btn_x + 20.0,
                btn_y + 35.0,
                20.0,
                BLACK
            );

            if btn_color == LIGHTGRAY {
                draw_rectangle_lines(btn_x, btn_y, 50.0, 50.0, 1.0, GOLD);
            }
        }
    }

    fn draw_valid_moves(&self, piece: i32, row: i32, col: i32) {
        if !self.resources.highlight_legal_moves {
            return;
        }

        let board_offset_x = (screen_width() - BOARD_WIDTH) / 2.0;
        let board_offset_y = (screen_height() - BOARD_HEIGHT) / 2.0;

        for r in 0..8 {
            for c in 0..8 {
                if is_valid_move(&self.game_state.board, piece, row, col, r, c, &self.game_state, true) {
                    draw_rectangle(
                        (c as f32 * 62.5 + board_offset_x),
                        (r as f32 * 62.5 + board_offset_y),
                        62.5,
                        62.5,
                        Color::new(0.0, 1.0, 0.0, 0.3),
                    );
                }
            }
        }
    }

    fn draw_game_settings(&mut self) -> GameScreen {
        clear_background(BLACK);
        
        draw_texture_ex(
            &self.resources.menu_background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));

        let title_size = measure_text("Game Settings", None, 40, 1.0);
        draw_text_ex(
            "Game Settings",
            screen_width() / 2.0 - title_size.width / 2.0,
            50.0,
            TextParams {
                font_size: 40,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_text("Highlight Legal Moves:", 100.0, 120.0, 20.0, WHITE);
        let highlight_toggle = Rect::new(350.0, 120.0, 50.0, 25.0);
        
        draw_rectangle(
            highlight_toggle.x,
            highlight_toggle.y,
            highlight_toggle.w,
            highlight_toggle.h,
            if self.resources.highlight_legal_moves { GREEN } else { RED },
        );
        
        let toggle_text = if self.resources.highlight_legal_moves { "ON" } else { "OFF" };
        let text_width = measure_text(toggle_text, None, 20, 1.0).width;
        draw_text(
            toggle_text,
            highlight_toggle.x + (highlight_toggle.w - text_width) / 2.0,
            highlight_toggle.y + 17.0,
            20.0,
            WHITE,
        );

        if is_point_in_rect(mouse_position().0, mouse_position().1, highlight_toggle.x, highlight_toggle.y, highlight_toggle.w, highlight_toggle.h) {
            draw_rectangle_lines(highlight_toggle.x, highlight_toggle.y, highlight_toggle.w, highlight_toggle.h, 2.0, GOLD);
            if is_mouse_button_pressed(MouseButton::Left) {
                self.resources.highlight_legal_moves = !self.resources.highlight_legal_moves;
            }
        }

        let back_button = self.draw_button(
            "Back",
            screen_width() / 2.0 - 100.0,
            500.0,
            200.0,
            50.0,
            is_point_in_rect(
                mouse_position().0,
                mouse_position().1,
                screen_width() / 2.0 - 100.0,
                500.0,
                200.0,
                50.0,
            ),
        );

        if back_button.clicked {
            return GameScreen::GameMenu;
        }

        GameScreen::GameSettings
    }

    fn draw_theme_settings(&mut self) -> GameScreen {
        draw_texture_ex(
            &self.resources.menu_background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));

        let title_width = measure_text("Theme Settings", None, 40, 1.0).width;
        draw_text_ex(
            "Theme Settings",
            screen_width() / 2.0 - title_width / 2.0,
            50.0,
            TextParams {
                font_size: 40,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_text("Select Board Theme:", 100.0, 120.0, 20.0, WHITE);

        let theme_button_width = 200.0;
        let theme_button_height = 50.0;
        let theme_button_x = 350.0;
        let theme_button_y = 120.0;
        let theme_button_gap = 10.0;

        for (i, theme) in self.resources.board_themes.iter().enumerate() {
            let theme_button = Rect::new(
                theme_button_x,
                theme_button_y + i as f32 * (theme_button_height + theme_button_gap),
                theme_button_width,
                theme_button_height,
            );

            let is_selected = i == self.resources.current_board_theme;
            let is_hovered = is_point_in_rect(
                mouse_position().0,
                mouse_position().1,
                theme_button.x,
                theme_button.y,
                theme_button.w,
                theme_button.h,
            );

            draw_rectangle(
                theme_button.x,
                theme_button.y,
                theme_button.w,
                theme_button.h,
                if is_selected {
                    GREEN
                } else if is_hovered {
                    LIGHTGRAY
                } else {
                    GRAY
                },
            );

            draw_text(
                theme.0.as_str(),
                theme_button.x + 10.0,
                theme_button.y + 15.0,
                20.0,
                BLACK,
            );

            if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                self.resources.current_board_theme = i;
            }
        }

        if !self.resources.board_themes.is_empty() {
            draw_texture_ex(
                &self.resources.board_themes[self.resources.current_board_theme].1,
                700.0,
                120.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(300.0, 300.0)),
                    ..Default::default()
                },
            );
        }

        let back_button = Rect::new(
            screen_width() / 2.0 - 100.0,
            500.0,
            200.0,
            50.0,
        );
        let is_back_button_hovered = is_point_in_rect(
            mouse_position().0,
            mouse_position().1,
            back_button.x,
            back_button.y,
            back_button.w,
            back_button.h,
        );

        let back_text = "Back";
        let text_x = back_button.x + 80.0;
        let text_y = back_button.y + 15.0;
        let font_size = 20.0;

        if is_back_button_hovered {
            draw_text(back_text, text_x - 1.0, text_y, font_size, WHITE);
            draw_text(back_text, text_x + 1.0, text_y, font_size, WHITE);
            draw_text(back_text, text_x, text_y - 1.0, font_size, WHITE);
            draw_text(back_text, text_x, text_y + 1.0, font_size, WHITE);
            draw_text(back_text, text_x, text_y, font_size, BLACK);
        } else {
            draw_text(back_text, text_x, text_y, font_size, WHITE);
        }

        if is_back_button_hovered && is_mouse_button_pressed(MouseButton::Left) {
            return GameScreen::GameMenu;
        }

        GameScreen::ThemeSettings
    }

    fn draw_button(&self, text: &str, x: f32, y: f32, w: f32, h: f32, hovered: bool) -> ButtonResult {
        draw_rectangle(x, y, w, h, if hovered { LIGHTGRAY } else { GRAY });
        
        if hovered {
            draw_rectangle_lines(x, y, w, h, 2.0, GOLD);
        }

        let text_size = measure_text(text, None, 20, 1.0);
        draw_text(
            text,
            x + (w - text_size.width) / 2.0,
            y + (h + text_size.height) / 2.0,
            20.0,
            if hovered { BLACK } else { WHITE },
        );

        ButtonResult {
            clicked: hovered && is_mouse_button_pressed(MouseButton::Left),
        }
    }
}

pub enum GameStatus {
    Playing,
    Checkmate,
    Exit,
}


// All the helper functions (is_valid_move, would_be_in_check, can_castle, is_king_in_check, is_checkmate, is_point_in_rect)
// remain exactly the same as in your original code, just moved inside the impl ChessGame block

struct ButtonResult {
    clicked: bool,
}

fn is_valid_move(board: &[[i32; 8]; 8], piece: i32, start_row: i32, start_col: i32, end_row: i32, end_col: i32, game_state: &ChessGameState, validate_check: bool) -> bool {
    if end_row < 0 || end_row >= 8 || end_col < 0 || end_col >= 8 {
        return false;
    }

    if board[end_row as usize][end_col as usize] != 0 && (board[end_row as usize][end_col as usize] * piece > 0) {
        return false;
    }

    let row_diff = end_row - start_row;
    let col_diff = end_col - start_col;

    match piece.abs() {
        1 => { 
            let direction = if piece > 0 { -1 } else { 1 };

            if col_diff == 0 && row_diff == direction && board[end_row as usize][end_col as usize] == 0 {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }

            if col_diff == 0 && row_diff == 2 * direction &&
                ((piece == 1 && start_row == 6) || (piece == -1 && start_row == 1)) &&
                board[(start_row + direction) as usize][start_col as usize] == 0 &&
                board[end_row as usize][end_col as usize] == 0
            {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }

            if col_diff.abs() == 1 && row_diff == direction && board[end_row as usize][end_col as usize] * piece < 0 {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }

            if col_diff.abs() == 1 && row_diff == direction &&
                end_row == game_state.en_passant_target_row && end_col == game_state.en_passant_target_col
            {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
        }
        2 => { 
            if (row_diff.abs() == 2 && col_diff.abs() == 1) || (row_diff.abs() == 1 && col_diff.abs() == 2) {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
        }
        3 => { 
            if row_diff.abs() == col_diff.abs() {
                let step_row = if row_diff > 0 { 1 } else { -1 };
                let step_col = if col_diff > 0 { 1 } else { -1 };

                for i in 1..row_diff.abs() {
                    if board[(start_row + i * step_row) as usize][(start_col + i * step_col) as usize] != 0 {
                        return false;
                    }
                }
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
        }
        4 => {
            if row_diff == 0 || col_diff == 0 {
                let step_row = if row_diff == 0 { 0 } else if row_diff > 0 { 1 } else { -1 };
                let step_col = if col_diff == 0 { 0 } else if col_diff > 0 { 1 } else { -1 };

                for i in 1..row_diff.abs().max(col_diff.abs()) {
                    if board[(start_row + i * step_row) as usize][(start_col + i * step_col) as usize] != 0 {
                        return false;
                    }
                }
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
        }
        5 => { 
            if row_diff.abs() == col_diff.abs() {
                let step_row = if row_diff > 0 { 1 } else { -1 };
                let step_col = if col_diff > 0 { 1 } else { -1 };

                for i in 1..row_diff.abs() {
                    if board[(start_row + i * step_row) as usize][(start_col + i * step_col) as usize] != 0 {
                        return false;
                    }
                }
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
            if row_diff == 0 || col_diff == 0 {
                let step_row = if row_diff == 0 { 0 } else if row_diff > 0 { 1 } else { -1 };
                let step_col = if col_diff == 0 { 0 } else if col_diff > 0 { 1 } else { -1 };

                for i in 1..row_diff.abs().max(col_diff.abs()) {
                    if board[(start_row + i * step_row) as usize][(start_col + i * step_col) as usize] != 0 {
                        return false;
                    }
                }
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }
        }
        6 => { 
            if row_diff.abs() <= 1 && col_diff.abs() <= 1 {
                if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                    return false;
                }
                return true;
            }

            if col_diff.abs() == 2 && row_diff == 0 {
                let kingside = col_diff > 0;
                let is_white = piece > 0;

                if can_castle(board, is_white, kingside, game_state) {
                    if validate_check && would_be_in_check(board, piece, start_row, start_col, end_row, end_col, piece > 0, game_state) {
                        return false;
                    }
                    return true;
                }
            }
        }
        _ => {}
    }

    false
}

fn would_be_in_check(board: &[[i32; 8]; 8], piece: i32, start_row: i32, start_col: i32, end_row: i32, end_col: i32, is_white: bool, game_state: &ChessGameState) -> bool {
    let mut temp_board = *board;
    temp_board[end_row as usize][end_col as usize] = piece;
    temp_board[start_row as usize][start_col as usize] = 0;

    is_king_in_check(&temp_board, is_white, game_state)
}

fn can_castle(board: &[[i32; 8]; 8], is_white: bool, kingside: bool, game_state: &ChessGameState) -> bool {
    let row = if is_white { 7 } else { 0 };
    let king_col = 4;
    let rook_col = if kingside { 7 } else { 0 };
    let step = if kingside { 1 } else { -1 };

    if is_white {
        if game_state.white_king_moved {
            return false;
        }
        if kingside && game_state.white_rook_kingside_moved {
            return false;
        }
        if !kingside && game_state.white_rook_queenside_moved {
            return false;
        }
    } else {
        if game_state.black_king_moved {
            return false;
        }
        if kingside && game_state.black_rook_kingside_moved {
            return false;
        }
        if !kingside && game_state.black_rook_queenside_moved {
            return false;
        }
    }

    for col in (king_col + step)..rook_col {
        if board[row as usize][col as usize] != 0 {
            return false;
        }
    }

    for col in king_col..(king_col + 2 * step) {
        let mut temp_board = *board;
        temp_board[row as usize][col as usize] = if is_white { 6 } else { -6 };
        temp_board[row as usize][king_col as usize] = 0;

        if is_king_in_check(&temp_board, is_white, game_state) {
            return false;
        }
    }

    true
}

fn is_king_in_check(board: &[[i32; 8]; 8], is_white: bool, game_state: &ChessGameState) -> bool {
    let mut king_row = -1;
    let mut king_col = -1;
    let king_piece = if is_white { 6 } else { -6 };

    for row in 0..8 {
        for col in 0..8 {
            if board[row as usize][col as usize] == king_piece {
                king_row = row;
                king_col = col;
                break;
            }
        }
        if king_row != -1 {
            break;
        }
    }

    if king_row == -1 || king_col == -1 {
        return false;
    }

    for row in 0..8 {
        for col in 0..8 {
            let piece = board[row as usize][col as usize];
            if piece != 0 && (piece * king_piece < 0) {
                if is_valid_move(board, piece, row, col, king_row, king_col, game_state, false) {
                    return true;
                }
            }
        }
    }

    false
}

fn is_checkmate(board: &[[i32; 8]; 8], is_white: bool, game_state: &ChessGameState) -> bool {
    // Must be in check first
    if !is_king_in_check(board, is_white, game_state) {
        return false;
    }

    // Check all possible moves for all pieces of current color
    for start_row in 0..8 {
        for start_col in 0..8 {
            let piece = board[start_row][start_col];
            
            // Skip empty squares and opponent pieces
            if piece == 0 || (is_white && piece < 0) || (!is_white && piece > 0) {
                continue;
            }

            // Check all possible destination squares
            for end_row in 0..8 {
                for end_col in 0..8 {
                    // Only check valid moves
                    if is_valid_move(
                        board, 
                        piece, 
                        start_row as i32, 
                        start_col as i32, 
                        end_row as i32, 
                        end_col as i32, 
                        game_state, 
                        true
                    ) {
                        // Simulate the move
                        let mut temp_board = *board;
                        temp_board[end_row][end_col] = piece;
                        temp_board[start_row][start_col] = 0;

                        // If this move gets out of check, it's not checkmate
                        if !is_king_in_check(&temp_board, is_white, game_state) {
                            return false;
                        }
                    }
                }
            }
        }
    }
    
    // No legal moves found that get out of check
    true
}

fn is_point_in_rect(x: f32, y: f32, rect_x: f32, rect_y: f32, rect_w: f32, rect_h: f32) -> bool {
    x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
}