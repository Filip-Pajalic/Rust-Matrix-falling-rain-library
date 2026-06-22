use macroquad::prelude::*;
use matrix_rain::{MacroquadRenderer, MatrixError, MatrixRain, MatrixRainConfig};
use serde::Deserialize;
use std::collections::VecDeque;
use std::time::Duration;

const FPS_SAMPLE_COUNT: usize = 60;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct DemoConfig {
    #[serde(flatten)]
    rain: MatrixRainConfig,
    debug_overlay: bool,
}

#[macroquad::main("Matrix Rain")]
async fn main() -> Result<(), MatrixError> {
    let demo_config = load_demo_config()?;
    let rain_config = demo_config.rain.clone();

    request_new_screen_size(
        rain_config.viewport_width_px as f32,
        rain_config.viewport_height_px as f32,
    );

    let font = load_ttf_font_from_bytes(include_bytes!("../resources/matrix-code.ttf")).map_err(
        |error| MatrixError::resource(format!("failed to load embedded font: {error:?}")),
    )?;

    let mut rain = MatrixRain::new(rain_config.clone())?;
    let mut renderer = MacroquadRenderer::new(&font, rain_config.cell_height_px as u16);
    let mut fps_samples = VecDeque::with_capacity(FPS_SAMPLE_COUNT);
    let mut frame_time_samples = VecDeque::with_capacity(FPS_SAMPLE_COUNT);

    loop {
        let frame_time = get_frame_time();
        rain.update(Duration::from_secs_f32(frame_time.max(0.0)));

        clear_background(BLACK);
        renderer.render(&rain);

        if demo_config.debug_overlay {
            track_frame_stats(&mut fps_samples, &mut frame_time_samples, frame_time);
            draw_debug_overlay(&fps_samples, &frame_time_samples, rain.glyphs().len());
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }

    Ok(())
}

fn track_frame_stats(
    fps_samples: &mut VecDeque<f32>,
    frame_time_samples: &mut VecDeque<f32>,
    frame_time: f32,
) {
    push_sample(fps_samples, get_fps() as f32);
    push_sample(frame_time_samples, frame_time * 1000.0);
}

fn push_sample(samples: &mut VecDeque<f32>, value: f32) {
    if samples.len() == FPS_SAMPLE_COUNT {
        samples.pop_front();
    }
    samples.push_back(value);
}

fn draw_debug_overlay(
    fps_samples: &VecDeque<f32>,
    frame_time_samples: &VecDeque<f32>,
    visible_glyphs: usize,
) {
    let avg_fps = average(fps_samples);
    let avg_frame_time = average(frame_time_samples);

    draw_text(&format!("FPS: {avg_fps:.1}"), 10.0, 30.0, 30.0, WHITE);
    draw_text(
        &format!("Frame: {avg_frame_time:.2}ms"),
        10.0,
        62.0,
        30.0,
        WHITE,
    );
    draw_text(
        &format!("Glyphs: {visible_glyphs}"),
        10.0,
        94.0,
        30.0,
        WHITE,
    );
}

fn average(samples: &VecDeque<f32>) -> f32 {
    if samples.is_empty() {
        0.0
    } else {
        samples.iter().sum::<f32>() / samples.len() as f32
    }
}

fn load_demo_config() -> Result<DemoConfig, MatrixError> {
    #[cfg(target_arch = "wasm32")]
    let config = include_str!("../config.yaml").to_string();

    #[cfg(not(target_arch = "wasm32"))]
    let config = std::fs::read_to_string("config.yaml")
        .map_err(|error| MatrixError::resource(format!("failed to read config.yaml: {error}")))?;

    let demo_config = serde_yaml::from_str::<DemoConfig>(&config)
        .map_err(|error| MatrixError::invalid_config(error.to_string()))?;
    demo_config.rain.validate()?;
    Ok(demo_config)
}
