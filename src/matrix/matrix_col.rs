use crate::matrix::matrix_character::MatrixCharacter;
use crate::matrix::util::random_duration;
use crate::FONT_HEIGHT;
use ::rand::{Rng, rng};
use macroquad::prelude::*;
use std::time::{Duration, Instant};

pub struct MatrixCol {
    pub code: MatrixCharacter,
    pub is_spawned: bool,
    pub time_elapsed: Instant,
    pub start_delay: Duration,
    pub timer_expired: bool,
}

impl MatrixCol {
    pub fn new(x_pos: i32, max_depth: u32, window_width: i32, child_spawn_time: u64) -> Self {
        MatrixCol {
            code: MatrixCharacter::new(
                Self::calculate_y_values(window_width),
                x_pos,
                Self::random_length(max_depth),
                child_spawn_time,
            ),
            is_spawned: true,
            timer_expired: false,
            time_elapsed: Instant::now(),
            start_delay: random_duration(Duration::from_millis(100), Duration::from_millis(9000)),
        }
    }
    pub fn update(&mut self, font: &Font, font_size: f32) {
        if self.time_elapsed.elapsed() > self.start_delay {
            self.timer_expired = true;
        }

        if self.timer_expired {
            self.code.traverse_and_tick(font, font_size);
            if self.code.alive == false {
                self.is_spawned = false;
            }
        }
    }

    fn random_length(max: u32) -> u32 {
        let mut rng = rng();
        rng.random_range(10..=max as i32 / 2) as u32
    }
    fn calculate_y_values(width: i32) -> i32 {
        let mut rng = rng();
        let min = -3;
        let max = width / FONT_HEIGHT - 3;
        rng.random_range(min..=max)
    }
}
