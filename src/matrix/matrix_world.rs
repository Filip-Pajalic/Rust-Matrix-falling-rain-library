use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, RwLock};

use crate::config::Config;
use crate::matrix::matrix_col::MatrixCol;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub struct MatrixError {
    pub message: String,
}

impl MatrixError {
    pub fn new(message: &str) -> Self {
        MatrixError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MatrixError: {}", self.message)
    }
}

impl Error for MatrixError {}

pub struct MatrixWorld {
    pub width: i32,
    pub height: i32,
    pub font_height: i32,
    pub font_width: i32,
    pub height_internal: i32,
    pub cols: Vec<MatrixCol>,
    pub config: Arc<RwLock<Config>>,
}

impl MatrixWorld {
    pub fn new(config: Arc<RwLock<Config>>) -> Result<Self, MatrixError> {
        let mut matrix = MatrixWorld {
            width: 0,
            height: 0,
            font_height: 0,
            cols: vec![],
            config,
            height_internal: 0,
            font_width: 0,
        };
        matrix.font_height = matrix
            .config
            .read()
            .map_err(|_| MatrixError::new("Failed to acquire read lock on config"))?
            .world
            .font_size_px;
        matrix.calculate_font_grid_size();
        for column_index in 0..matrix.width {
            matrix.cols.push(MatrixCol::new(
                column_index,
                matrix.height as u32,
                matrix.width,
                matrix.config.read().unwrap().glyph.child_spawn_interval_ms as u64,
            ));
        }

        Ok(matrix)
    }

    pub fn update(&mut self, font: &Font, font_size: f32) {
        for i in 0..self.width {
            self.spawn_col(i);
        }
        for col in self.cols.iter_mut() {
            col.update(font, font_size);
        }
        if self.config.read().unwrap().debug {
            self.debug_grid(font);
        }
    }

    fn calculate_grid_size(&self) -> (i32, i32) {
        let grid_size_x = self.width / self.font_width;
        let grid_size_y = self.height / self.font_height;
        (grid_size_x, grid_size_y)
    }
    fn get_grid_offset(&self) -> (i32, i32) {
        let grid_size_x_remainder = (self.width % self.font_width) / 2;
        let grid_size_y_remainder = (self.height % self.font_height) / 2;

        (grid_size_x_remainder, grid_size_y_remainder)
    }

    fn update_font_width(&mut self, font: &Font) {
        let widest_char = self.get_widest_char_width(font);
        let font_width_ratio = widest_char / self.font_height as f32;
        self.font_width = (self.font_height as f32 * font_width_ratio).round() as i32;
    }

    fn get_widest_char_width(&self, font: &Font) -> f32 {
        let characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                1234567890";
        characters
            .chars()
            .map(|c| self.get_character_width(&font, &c.to_string()))
            .fold(0.0, f32::max)
    }

    fn get_character_width(&self, font: &Font, character: &str) -> f32 {
        measure_text(character, Some(font), self.font_height as u16, 1.0).width
    }

    fn get_height_internal_offset_range(&self) -> (i32, i32) {
        let range = (self.height_internal - self.height).abs() / 2;
        (-range, range)
    }

    pub fn debug_grid(&mut self, font: &Font) {
        let ( grid_size_x,  grid_size_y) = self.calculate_grid_size();
        let ( grid_offset_x,  grid_offset_y) = self.get_grid_offset();
        self.update_font_width(font);

        let (min, max) = self.get_height_internal_offset_range();
        //let offset = self.get_character_offset(font, "A"); // so the grid is not over /font offset
        for x in min..grid_size_x + max {
            for y in min..grid_size_y + max {
                draw_rectangle_lines(
                    grid_offset_x as f32 + x as f32 * self.font_width as f32,
                    grid_offset_y as f32 + y as f32 * self.font_height as f32,
                    self.font_width as f32,
                    self.font_height as f32,
                    1.0,
                    RED,
                );
            }
        }
    }

    fn calculate_font_grid_size(&mut self) {
        if let Ok(cfg) = self.config.read() {
            self.width = cfg.world.window_width_px / 10;
            self.height = cfg.world.window_height_px / 10;
        }
    }

    fn spawn_col(&mut self, index: i32) {
        if let Some(col) = self.cols.get_mut(index as usize) {
            if !col.is_spawned {
                *col = MatrixCol::new(
                    index,
                    self.height as u32,
                    self.width,
                    self.config.read().unwrap().glyph.child_spawn_interval_ms as u64,
                );
            }
        }
    }
}
