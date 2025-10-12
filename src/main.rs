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
            
            let font_bytes = include_bytes!("../resources/matrix-code.ttf");
            let font = load_ttf_font_from_bytes(font_bytes).unwrap();

            drop(config_read);
            
            // FPS tracking
            let mut fps_samples = Vec::new();
            let mut frame_time_samples = Vec::new();
            
            loop {
                clear_background(BLACK);
                
                matrix.update(&font, font_size);
                
                // Collect FPS samples
                fps_samples.push(get_fps() as f32);
                frame_time_samples.push(get_frame_time() * 1000.0);
                
                // Keep only last 60 samples
                if fps_samples.len() > 60 {
                    fps_samples.remove(0);
                    frame_time_samples.remove(0);
                }
                
                // Calculate averages
                let avg_fps = fps_samples.iter().sum::<f32>() / fps_samples.len() as f32;
                let avg_frame_time = frame_time_samples.iter().sum::<f32>() / frame_time_samples.len() as f32;
                
                // Draw FPS in front of everything with big white text
                draw_text(
                    &format!("FPS: {:.1}", avg_fps), 
                    10.0, 30.0, 40.0, WHITE
                );
                draw_text(
                    &format!("Frame: {:.2}ms", avg_frame_time), 
                    10.0, 70.0, 40.0, WHITE
                );
                
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
