use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, RwLock};

use crate::config::Config;
use crate::matrix::matrix_col::MatrixCol;
use crate::matrix::batch_renderer::BatchRenderer;
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
    width: i32,
    height: i32,
    font_height: i32,
    font_width: i32,
    cols: Vec<MatrixCol>,
    child_spawn_interval_ms: u64,
    debug_mode: bool,
    cached_font_width: Option<i32>,
    // Batch renderer for efficient drawing
    batch_renderer: BatchRenderer,
}

impl MatrixWorld {
    pub fn new(config: Arc<RwLock<Config>>) -> Result<Self, MatrixError> {
        let config_read = config
            .read()
            .map_err(|_| MatrixError::new("Failed to acquire read lock on config"))?;
        
        let child_spawn_interval = config_read.glyph.child_spawn_interval_ms as u64;
        let debug = config_read.debug;
        let font_height = config_read.world.font_size_px;
        let window_width = config_read.world.window_width_px;
        let window_height = config_read.world.window_height_px;
        
        drop(config_read);
        
        let width = window_width / 10;
        let height = window_height / 10;
        
        // Estimate max characters on screen for batch size
        // Each column can have up to (height / 2) characters
        let estimated_chars = (width * height / 2) as usize;
        
        let mut cols = Vec::with_capacity(width as usize);
        
        for column_index in 0..width {
            cols.push(MatrixCol::new(
                column_index,
                height as u32,
                width,
                child_spawn_interval,
            ));
        }
        
        Ok(MatrixWorld {
            width,
            height,
            font_height,
            font_width: 0,
            cols,
            child_spawn_interval_ms: child_spawn_interval,
            debug_mode: debug,
            cached_font_width: None,
            batch_renderer: BatchRenderer::new(estimated_chars),
        })
    }

    /// Main update loop with batch rendering
    pub fn update(&mut self, font: &Font, font_size: f32) {
        // Clear batch from previous frame
        self.batch_renderer.clear();
        
        // Phase 1: Update logic and collect render data
        for col in self.cols.iter_mut() {
            col.update_and_collect(&mut self.batch_renderer, font_size);
            
            // Respawn dead columns
            if !col.is_spawned {
                *col = MatrixCol::new(
                    col.code.x_pos,
                    self.height as u32,
                    self.width,
                    self.child_spawn_interval_ms,
                );
            }
        }
        
        // Phase 2: Render everything in one batch
        self.batch_renderer.render(font, font_size);
        
        // Debug rendering
        if self.debug_mode {
            self.debug_grid(font);
        }
    }

    #[inline]
    fn calculate_grid_size(&self) -> (i32, i32) {
        let grid_size_x = self.width / self.font_width;
        let grid_size_y = self.height / self.font_height;
        (grid_size_x, grid_size_y)
    }

    #[inline]
    fn get_grid_offset(&self) -> (i32, i32) {
        let grid_size_x_remainder = (self.width % self.font_width) / 2;
        let grid_size_y_remainder = (self.height % self.font_height) / 2;
        (grid_size_x_remainder, grid_size_y_remainder)
    }

    fn update_font_width(&mut self, font: &Font) {
        if self.cached_font_width.is_some() {
            return;
        }
        
        let widest_char = self.get_widest_char_width(font);
        let font_width_ratio = widest_char / self.font_height as f32;
        self.font_width = (self.font_height as f32 * font_width_ratio).round() as i32;
        self.cached_font_width = Some(self.font_width);
    }

    fn get_widest_char_width(&self, font: &Font) -> f32 {
        const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz1234567890";
        
        CHARS
            .chars()
            .map(|c| {
                let mut buf = [0u8; 4];
                let char_str = c.encode_utf8(&mut buf);
                self.get_character_width(font, char_str)
            })
            .fold(0.0, f32::max)
    }

    #[inline]
    fn get_character_width(&self, font: &Font, character: &str) -> f32 {
        measure_text(character, Some(font), self.font_height as u16, 1.0).width
    }

    pub fn debug_grid(&mut self, font: &Font) {
        self.update_font_width(font);
        
        let (grid_size_x, grid_size_y) = self.calculate_grid_size();
        let (grid_offset_x, grid_offset_y) = self.get_grid_offset();
        
        for x in 0..grid_size_x {
            for y in 0..grid_size_y {
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
    
    #[inline]
    pub fn width(&self) -> i32 {
        self.width
    }
    
    #[inline]
    pub fn height(&self) -> i32 {
        self.height
    }
    
    #[inline]
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }
    
    pub fn toggle_debug(&mut self) {
        self.debug_mode = !self.debug_mode;
    }
}