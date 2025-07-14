use macroquad::prelude::*;

const GHOST_WIDTH: f32 = 11.0;
const GHOST_HEIGHT: f32 = 16.0;

pub struct GhostAnimation {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    animation_frame: usize,
    animation_timer: f32,
    wiggle_offset: f32,
    ghost_texture: Option<Texture2D>,
}

impl GhostAnimation {
    pub async fn new() -> Self {
        // Create a simple pixel-art texture for the ghost
        let ghost_texture = Some(Self::create_ghost_texture().await);
        
        Self {
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            velocity_x: 0.8,
            velocity_y: 0.5,
            animation_frame: 0,
            animation_timer: 0.0,
            wiggle_offset: 0.0,
            ghost_texture,
        }
    }

    async fn create_ghost_texture() -> Texture2D {
        let mut image = Image::gen_image_color(GHOST_WIDTH as u16, GHOST_HEIGHT as u16, BLANK);
        
        // Ghost body pattern (same as in original code)
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

        for (y, row) in ghost_pattern.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == 1 {
                    let color = match y {
                        0..=3 => WHITE,
                        4..=7 => Color::new(0.9, 0.9, 0.9, 1.0),
                        8..=11 => Color::new(0.8, 0.8, 0.8, 1.0),
                        _ => Color::new(0.7, 0.7, 0.7, 1.0),
                    };
                    image.set_pixel(x as u32, y as u32, color);
                }
            }
        }

        // Add eyes
        image.set_pixel(3, 3, BLUE);
        image.set_pixel(4, 3, BLUE);
        image.set_pixel(3, 4, BLUE);
        image.set_pixel(4, 4, BLUE);
        image.set_pixel(6, 3, BLUE);
        image.set_pixel(7, 3, BLUE);
        image.set_pixel(6, 4, BLUE);
        image.set_pixel(7, 4, BLUE);

        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        texture
    }

    pub fn update(&mut self) {
        // Movement
        self.x += self.velocity_x;
        self.y += self.velocity_y + self.wiggle_offset.sin() * 0.3;
        self.wiggle_offset += 0.1;

        // Bounce off walls
        if self.x < GHOST_WIDTH || self.x > screen_width() - GHOST_WIDTH {
            self.velocity_x *= -1.0;
        }
        if self.y < GHOST_HEIGHT || self.y > screen_height() - GHOST_HEIGHT {
            self.velocity_y *= -1.0;
        }

        // Animation
        self.animation_timer += get_frame_time();
        if self.animation_timer >= 0.1 {
            self.animation_timer = 0.0;
            self.animation_frame = (self.animation_frame + 1) % 4;
        }
    }

    pub fn draw(&self) {
        // Draw shadow
        draw_rectangle(
            self.x - GHOST_WIDTH / 2.0,
            self.y + GHOST_HEIGHT / 2.0 - 2.0,
            GHOST_WIDTH,
            4.0,
            Color::new(0.1, 0.1, 0.2, 0.3),
        );

        // Draw ghost
        if let Some(texture) = &self.ghost_texture {
            let y_offset = match self.animation_frame {
                1 | 3 => 1.0,
                _ => 0.0,
            };

            draw_texture_ex(
                texture,
                self.x - GHOST_WIDTH / 2.0,
                self.y - GHOST_HEIGHT / 2.0 + y_offset,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(GHOST_WIDTH * 4.0, GHOST_HEIGHT * 4.0)),
                    flip_x: self.velocity_x < 0.0,
                    ..Default::default()
                },
            );
        } else {
            // Fallback if texture fails to load
            draw_rectangle(
                self.x - GHOST_WIDTH / 2.0,
                self.y - GHOST_HEIGHT / 2.0,
                GHOST_WIDTH,
                GHOST_HEIGHT,
                WHITE,
            );
        }
    }
}