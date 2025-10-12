use macroquad::color::{Color, WHITE};
use std::time::Duration;
use instant::Instant;

pub enum Easing {
    EaseOut,
}

pub struct Transition {
    pub duration: Duration,
    pub easing: Easing,
    pub trigger_required: bool,
    pub triggered: bool,
}

impl Transition {
    pub fn new(duration: Duration, easing: Easing, trigger_required: bool) -> Self {
        Transition {
            duration,
            easing,
            trigger_required,
            triggered: false,
        }
    }
}

pub struct AnimationStep {
    pub color: Color,
    pub duration: Duration,
    pub transition: Transition,
}

pub struct Animation {
    pub current_color: Color,
    step_start_time: Instant,  
    steps: Vec<AnimationStep>,
    current_step_index: usize,
    pub concluded: bool,
}

impl Animation {
    pub fn new(steps: Vec<AnimationStep>) -> Self {
        Animation {
            current_color: WHITE,
            step_start_time: Instant::now(),
            steps,
            current_step_index: 0,
            concluded: false,
        }
    }

    pub fn update(&mut self) {
        if self.steps.is_empty() {
            self.concluded = true;
            return;
        }
        self.update_color();
        self.advance_step_if_needed();
    }

    pub fn trigger_transition(&mut self) {
        let current_step = &mut self.steps[self.current_step_index];
        if current_step.transition.trigger_required && !current_step.transition.triggered {
            current_step.transition.triggered = true;
            self.step_start_time = Instant::now();
        }
    }

    fn update_color(&mut self) {
        let current_step = &self.steps[self.current_step_index];
        let elapsed = self.step_start_time.elapsed();
        
        if elapsed < current_step.duration {
            // Still in step duration
            self.current_color = current_step.color;
        } else if !self.is_last_step() {
            let transition = &current_step.transition;
            if transition.trigger_required && !transition.triggered {
                // Waiting for trigger
                self.current_color = current_step.color;
            } else {
                // In transition
                let transition_elapsed = elapsed - current_step.duration;
                let t = (transition_elapsed.as_secs_f32() / transition.duration.as_secs_f32()).min(1.0);
                let eased_t = self.apply_easing(t, &transition.easing);
                let next_color = self.steps[self.current_step_index + 1].color;
                self.current_color = interpolate(current_step.color, next_color, eased_t);
            }
        } else {
            self.current_color = current_step.color;
        }
    }

    fn apply_easing(&self, t: f32, easing: &Easing) -> f32 {
        match easing {
            Easing::EaseOut => ease_out(t),
        }
    }

    fn advance_step_if_needed(&mut self) {
        if self.is_last_step() {
            self.concluded = true;
            return;
        }

        let current_step = &self.steps[self.current_step_index];
        let elapsed = self.step_start_time.elapsed();
        let total_duration = current_step.duration + current_step.transition.duration;

        if elapsed >= total_duration {
            self.current_step_index += 1;
            self.step_start_time = Instant::now();
            
            if self.is_last_step() {
                self.concluded = true;
            }
        }
    }

    fn is_last_step(&self) -> bool {
        self.current_step_index == self.steps.len() - 1
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn interpolate(start: Color, end: Color, factor: f32) -> Color {
    let r = start.r as f32 * (1.0 - factor) + end.r as f32 * factor;
    let g = start.g as f32 * (1.0 - factor) + end.g as f32 * factor;
    let b = start.b as f32 * (1.0 - factor) + end.b as f32 * factor;
    let a = start.a as f32 * (1.0 - factor) + end.a as f32 * factor;
    Color::new(r, g, b, a)
}