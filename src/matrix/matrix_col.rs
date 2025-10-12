use crate::matrix::matrix_character::MatrixCharacter;
use crate::matrix::util::random_duration;
use crate::matrix::batch_renderer::BatchRenderer;
use crate::FONT_HEIGHT;
use macroquad::prelude::*;
use std::time::Duration;
use instant::Instant;

pub struct MatrixCol {
    pub code: MatrixCharacter,
    pub is_spawned: bool,
    start_time: Instant,
    start_delay: Duration,
}

impl MatrixCol {
    pub fn new(x_pos: i32, max_depth: u32, window_width: i32, child_spawn_time: u64) -> Self {
        let start_delay = random_duration(
            Duration::from_millis(100), 
            Duration::from_millis(9000)
        );
        
        MatrixCol {
            code: MatrixCharacter::new(
                Self::calculate_random_y_position(window_width),
                x_pos,
                Self::calculate_random_length(max_depth),
                child_spawn_time,
            ),
            is_spawned: true,
            start_time: Instant::now(),
            start_delay,
        }
    }
    
    /// Update and collect render data for batching
    #[inline]
    pub fn update_and_collect(&mut self, batch: &mut BatchRenderer, font_size: f32) {
        let elapsed = self.start_time.elapsed();
        
        if elapsed >= self.start_delay {
            self.code.traverse_and_collect(batch, font_size);
            
            if !self.code.alive {
                self.is_spawned = false;
            }
        }
    }

    #[inline]
    pub fn has_started(&self) -> bool {
        self.start_time.elapsed() >= self.start_delay
    }

    #[inline]
    fn calculate_random_length(max_depth: u32) -> u32 {
        let min_length = 10;
        let max_length = max_depth / 2;
        
        if max_length <= min_length {
            return min_length;
        }
        
        fastrand::u32(min_length..=max_length)
    }
    
    #[inline]
    fn calculate_random_y_position(window_width: i32) -> i32 {
        let offset = 3;
        let min = -offset;
        let max = (window_width / FONT_HEIGHT) - offset;
        
        fastrand::i32(min..=max)
    }
}