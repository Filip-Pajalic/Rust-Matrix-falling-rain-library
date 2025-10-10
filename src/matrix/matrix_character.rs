use crate::animation::animation::{AnimationStep, Easing, Transition};
use crate::animation::Animation;
use crate::matrix::util::{random_duration, AlphanumericMatrix};
use crate::FONT_WIDTH;
use ::rand::{Rng, rng};
use macroquad::prelude::*;
use std::time::{Duration, Instant};

pub struct MatrixCharacter {
    pub y_pos: i32,
    pub x_pos: i32,
    pub glyph: char,
    pub color: Color,
    pub max_ancestors: u32,
    pub child: Option<Box<MatrixCharacter>>,
    pub child_spawn_timer: Instant,
    pub child_spawn_time: u64,
    pub glyph_change_timer: Instant,
    pub glyph_change_interval: Duration,
    pub alive: bool,
    pub animation: Animation,
}

#[repr(u32)]
#[derive(Clone)]
enum MatrixColor {
    GREEN = 0x00FF00FF,
}

impl MatrixColor {
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
        let mut rng = rng();
        MatrixCharacter {
            y_pos,
            x_pos,
            glyph: Self::generate_random_character(),
            color: WHITE,
            child: None,
            max_ancestors,
            child_spawn_timer: Instant::now(),
            child_spawn_time,
            glyph_change_timer: Instant::now(),
            glyph_change_interval: random_duration(
                Duration::from_millis(100),
                Duration::from_millis(700),
            ),
            alive: true,
            animation: Animation::new(Self::create_animation_steps(
                rng.random_range(50..=255) as u8
            )),
        }
    }

    fn spawn_child(&mut self) {
        if self.child.is_none()
            && self.y_pos <= self.max_ancestors as i32
            && self.child_spawn_timer.elapsed() > Duration::from_millis(self.child_spawn_time)
        {
            self.child = Some(Box::new(MatrixCharacter::new(
                self.y_pos + 1,
                self.x_pos,
                self.max_ancestors.clone(),
                self.child_spawn_time,
            )));
            self.animation.trigger_transition();
        }
    }
    pub fn generate_random_character() -> char {
        let mut rng = rng();
        rng.sample(AlphanumericMatrix) as char
    }
    fn symbol_to_string(&self) -> String {
        self.glyph.to_string()
    }

    fn update(&mut self, font: &Font, font_size: f32) {
        self.spawn_child();
        self.update_appearance();

        draw_text_ex(
            &self.symbol_to_string(),
            self.x_pos as f32 * FONT_WIDTH as f32,
            self.y_pos as f32 * font_size,
            TextParams {
                font_size: font_size as u16,
                font: Some(font),
                color: self.color,
                ..Default::default()
            },
        );
    }

    fn update_appearance(&mut self) {
        if self.glyph_change_timer.elapsed() >= self.glyph_change_interval {
            self.glyph = Self::generate_random_character();
            self.glyph_change_timer = Instant::now()
        }

        self.animation.update();
        self.color = self.animation.current_color;
    }

    pub fn traverse_and_tick(&mut self, font: &Font, font_size: f32) {
        if let Some(ref mut child) = self.child {
            child.traverse_and_tick(font, font_size);
            if !child.alive {
                if self.animation.concluded {
                    self.alive = false;
                }
            }
        } else {
            if self.animation.concluded {
                self.alive = false;
            }
        }
        self.update(font, font_size);
    }
    fn create_animation_steps(alpha: u8) -> Vec<AnimationStep> {
        let steps = vec![
            AnimationStep {
                color: WHITE,
                duration: Duration::from_millis(100),
                transition: Transition::new(Duration::from_millis(1000), Easing::EaseOut, true),
            },
            AnimationStep {
                color: MatrixColor::GREEN.to_color(alpha),
                duration: Duration::from_millis(3000),
                transition: Transition::new(Duration::from_millis(1000), Easing::EaseOut, false),
            },
            AnimationStep {
                color: BLACK,
                duration: Duration::from_millis(500),
                transition: Transition::new(Duration::from_millis(2000), Easing::EaseOut, false),
            },
        ];
        steps
    }
}
