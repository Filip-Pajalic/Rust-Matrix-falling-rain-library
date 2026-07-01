//! C-ABI WASM binding for the `matrix-rain` simulation core.
//!
//! The Rust library owns the *simulation* (stream spawning, glyph changes,
//! fade); JS owns the *rendering*. Each frame JS calls [`mr_update`] with the
//! elapsed milliseconds, then reads a flat `f32` buffer of glyph instances via
//! [`mr_buffer_ptr`] and paints them on a canvas.
//!
//! Buffer layout — 7 f32 per glyph, tightly packed:
//!   [ x, y, codepoint, r, g, b, a ]
//! x/y are pixel positions (top-left of the cell); codepoint is the ASCII glyph;
//! r/g/b/a are 0..255. The count is the return value of `mr_update`.
//!
//! Single-threaded wasm, so a couple of `static mut` globals are fine.
#![allow(static_mut_refs)]

use std::time::Duration;

use matrix_rain::{MatrixRain, MatrixRainConfig, Rgba};

const FIELDS_PER_GLYPH: usize = 7;

static mut RAIN: Option<MatrixRain> = None;
static mut BUFFER: Vec<f32> = Vec::new();

/// Create (or recreate) the simulation sized to a `width` x `height` viewport,
/// with `cell_w` x `cell_h` glyph cells. `seed_lo`/`seed_hi` form a u64 seed so
/// runs are deterministic and we never touch wasm-unavailable entropy sources.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn mr_init(
    width: u32,
    height: u32,
    cell_w: u32,
    cell_h: u32,
    seed_lo: u32,
    seed_hi: u32,
    min_len: u32,
    max_len: u32,
    step_ms: u32,
    init_min_ms: u32,
    init_max_ms: u32,
    respawn_min_ms: u32,
    respawn_max_ms: u32,
) {
    let config = MatrixRainConfig {
        viewport_width_px: width.max(1),
        viewport_height_px: height.max(1),
        cell_width_px: cell_w.max(1),
        cell_height_px: cell_h.max(1),
        // Stream shape + timing come from the JS caller so the look can be tuned
        // without recompiling.
        min_stream_length: min_len.max(1),
        max_stream_length: max_len.max(min_len.max(1)),
        stream_step_ms: step_ms.max(1) as u64,
        initial_spawn_delay_min_ms: init_min_ms as u64,
        initial_spawn_delay_max_ms: init_max_ms.max(init_min_ms) as u64,
        respawn_delay_min_ms: respawn_min_ms as u64,
        respawn_delay_max_ms: respawn_max_ms.max(respawn_min_ms) as u64,
        // Bright near-white head, classic green trail. JS may re-tint per theme
        // but the alpha (fade) is authored here.
        head_color: Rgba::new(210, 255, 220, 255),
        trail_color: Rgba::new(0, 255, 90, 210),
        ..MatrixRainConfig::default()
    };
    let seed = ((seed_hi as u64) << 32) | seed_lo as u64;
    unsafe {
        RAIN = MatrixRain::with_seed(config, seed).ok();
    }
}

/// Resize the field (e.g. on window resize). No-op if not initialized.
#[no_mangle]
pub extern "C" fn mr_resize(width: u32, height: u32) {
    unsafe {
        if let Some(rain) = RAIN.as_mut() {
            rain.resize(width.max(1), height.max(1));
        }
    }
}

/// Live-retune stream shape + speed without rebuilding the field — so a UI
/// slider changes the running animation instead of restarting it. Cell size /
/// viewport changes still need `mr_init` (they relayout the columns).
#[no_mangle]
pub extern "C" fn mr_set_timing(
    min_len: u32,
    max_len: u32,
    step_ms: u32,
    respawn_min_ms: u32,
    respawn_max_ms: u32,
) {
    unsafe {
        if let Some(rain) = RAIN.as_mut() {
            let cfg = rain.config_mut();
            cfg.min_stream_length = min_len.max(1);
            cfg.max_stream_length = max_len.max(cfg.min_stream_length);
            cfg.stream_step_ms = (step_ms.max(1)) as u64;
            cfg.respawn_delay_min_ms = respawn_min_ms as u64;
            cfg.respawn_delay_max_ms = respawn_max_ms.max(respawn_min_ms) as u64;
        }
    }
}

/// Advance the simulation by `dt_ms` and pack the visible glyphs into the shared
/// buffer. Returns the glyph count (buffer holds `count * 7` f32).
#[no_mangle]
pub extern "C" fn mr_update(dt_ms: f32) -> u32 {
    unsafe {
        let Some(rain) = RAIN.as_mut() else {
            return 0;
        };
        // Clamp dt so a backgrounded tab that resumes doesn't fast-forward wildly.
        let dt = Duration::from_secs_f32(dt_ms.clamp(0.0, 100.0) / 1000.0);
        rain.update(dt);

        let glyphs = rain.glyphs();
        BUFFER.clear();
        BUFFER.reserve(glyphs.len() * FIELDS_PER_GLYPH);
        for g in glyphs {
            BUFFER.push(g.position.x);
            BUFFER.push(g.position.y);
            BUFFER.push(g.glyph as u32 as f32);
            BUFFER.push(g.color.r as f32);
            BUFFER.push(g.color.g as f32);
            BUFFER.push(g.color.b as f32);
            BUFFER.push(g.color.a as f32);
        }
        glyphs.len() as u32
    }
}

/// Pointer into wasm linear memory for the packed glyph buffer. Valid until the
/// next `mr_update`. JS wraps it as `Float32Array(memory.buffer, ptr, count*7)`.
#[no_mangle]
pub extern "C" fn mr_buffer_ptr() -> *const f32 {
    unsafe { BUFFER.as_ptr() }
}
