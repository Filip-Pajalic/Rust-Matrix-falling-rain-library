mod animation;
mod config;
mod game;
mod matrix;

use macroquad::prelude::*;
use std::process;

use crate::config::Config;
use crate::game::GameState;
use matrix::*;

const FONT_HEIGHT: i32 = 30;
const FONT_WIDTH: i32 = 30;

#[macroquad::main("Matrix")]
async fn main() -> Result<(), MatrixError> {
    let config = Config::from_file("config.yaml");
    let game_state = GameState::new(config);

    let matrix = MatrixWorld::new(game_state.config.clone());
    match matrix {
        Ok(mut matrix) => {
            let config_read = game_state.config.read().unwrap();
            let window_width = config_read.world.window_width_px;
            let window_height = config_read.world.window_height_px;
            let font_size = config_read.world.font_size_px as f32;

            request_new_screen_size(window_width as f32, window_height as f32);
            let font = load_ttf_font("resources/matrix-code.ttf").await.unwrap();

            drop(config_read);
            loop {
                clear_background(BLACK);
                matrix.update(&font, font_size);
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                next_frame().await;
            }
            Ok(())
        }
        Err(error) => {
            eprintln!("Error creating matrix: {}", error);
            process::exit(1)
        }
    }
}
