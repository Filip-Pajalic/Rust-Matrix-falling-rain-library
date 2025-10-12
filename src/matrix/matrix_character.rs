use crate::animation::animation::{AnimationStep, Easing, Transition};
use crate::animation::Animation;
use crate::matrix::util::{random_duration, AlphanumericMatrix};
use crate::matrix::batch_renderer::BatchRenderer;
use crate::FONT_WIDTH;
use macroquad::prelude::*;
use std::time::Duration;
use instant::Instant;

pub struct MatrixCharacter {
    pub y_pos: i32,
    pub x_pos: i32,
    glyph: char,
    color: Color,
    max_ancestors: u32,
    child: Option<Box<MatrixCharacter>>,
    child_spawn_timer: Instant,
    child_spawn_time: u64,
    glyph_change_timer: Instant,
    glyph_change_interval: Duration,
    pub alive: bool,
    animation: Animation,
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum MatrixColor {
    GREEN = 0x00FF00FF,
}

impl MatrixColor {
    #[inline]
    fn to_color(self, alpha: u8) -> Color {
        let hex = self as u32;
        let r = ((hex >> 24) & 0xFF) as u8;
        let g = ((hex >> 16) & 0xFF) as u8;
        let b = ((hex >> 8) & 0xFF) as u8;
        Color::from_rgba(r, g, b, alpha)
    }
}

impl MatrixCharacter {
    pub fn new(y_pos: i32, x_pos: i32, max_ancestors: u32, child_spawn_time: u64) -> Self {
        let glyph_change_interval = random_duration(
            Duration::from_millis(100),
            Duration::from_millis(700),
        );
        
        MatrixCharacter {
            y_pos,
            x_pos,
            glyph: AlphanumericMatrix::random_char(),
            color: WHITE,
            child: None,
            max_ancestors,
            child_spawn_timer: Instant::now(),
            child_spawn_time,
            glyph_change_timer: Instant::now(),
            glyph_change_interval,
            alive: true,
            animation: Animation::new(Self::create_animation_steps(
                fastrand::u8(50..=255)
            )),
        }
    }

    #[inline]
    fn spawn_child(&mut self) {
        if self.child.is_none()
            && self.y_pos <= self.max_ancestors as i32
            && self.child_spawn_timer.elapsed() > Duration::from_millis(self.child_spawn_time)
        {
            self.child = Some(Box::new(MatrixCharacter::new(
                self.y_pos + 1,
                self.x_pos,
                self.max_ancestors,
                self.child_spawn_time,
            )));
            self.animation.trigger_transition();
        }
    }

    #[inline]
    fn update_appearance(&mut self) {
        if self.glyph_change_timer.elapsed() >= self.glyph_change_interval {
            self.glyph = AlphanumericMatrix::random_char();
            self.glyph_change_timer = Instant::now();
        }

        self.animation.update();
        self.color = self.animation.current_color;
    }

    /// Collect rendering data into a batch (no immediate drawing)
    #[inline]
    fn collect_render_data(&self, batch: &mut BatchRenderer, font_size: f32) {
        let position = Vec2::new(
            self.x_pos as f32 * FONT_WIDTH as f32,
            self.y_pos as f32 * font_size,
        );
        batch.push(position, self.color, self.glyph);
    }

    /// Update logic and collect all render data for batching
    pub fn traverse_and_collect(&mut self, batch: &mut BatchRenderer, font_size: f32) {
        // Update this character
        self.spawn_child();
        self.update_appearance();
        
        // Collect render data (don't draw yet)
        self.collect_render_data(batch, font_size);
        
        // Process child
        if let Some(ref mut child) = self.child {
            if !child.alive && self.animation.concluded {
                self.alive = false;
            } else if child.alive {
                child.traverse_and_collect(batch, font_size);
            }
        } else if self.animation.concluded {
            self.alive = false;
        }
    }

    fn create_animation_steps(alpha: u8) -> Vec<AnimationStep> {
        vec![
            AnimationStep {
                color: WHITE,
                duration: Duration::from_millis(100),
                transition: Transition::new(
                    Duration::from_millis(1000), 
                    Easing::EaseOut, 
                    true
                ),
            },
            AnimationStep {
                color: MatrixColor::GREEN.to_color(alpha),
                duration: Duration::from_millis(3000),
                transition: Transition::new(
                    Duration::from_millis(1000), 
                    Easing::EaseOut, 
                    false
                ),
            },
            AnimationStep {
                color: BLACK,
                duration: Duration::from_millis(500),
                transition: Transition::new(
                    Duration::from_millis(2000), 
                    Easing::EaseOut, 
                    false
                ),
            },
        ]
    }
}