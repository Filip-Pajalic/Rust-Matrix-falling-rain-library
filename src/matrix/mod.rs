use std::time::Duration;

use fastrand::Rng;

use crate::{MatrixError, MatrixRainConfig, Rgba};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphRole {
    Head,
    Trail,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphInstance {
    pub glyph: char,
    pub column: u32,
    pub row: i32,
    pub position: Position,
    pub color: Rgba,
    pub role: GlyphRole,
}

pub trait MatrixRenderer {
    fn render_glyphs(&mut self, glyphs: &[GlyphInstance]);
}

pub struct MatrixRain {
    config: MatrixRainConfig,
    rng: Rng,
    columns: Vec<Column>,
    glyphs: Vec<GlyphInstance>,
}

impl MatrixRain {
    pub fn new(config: MatrixRainConfig) -> Result<Self, MatrixError> {
        Self::with_seed(config, fastrand::u64(..))
    }

    pub fn with_seed(config: MatrixRainConfig, seed: u64) -> Result<Self, MatrixError> {
        config.validate()?;

        let mut rain = Self {
            config,
            rng: Rng::with_seed(seed),
            columns: Vec::new(),
            glyphs: Vec::new(),
        };
        rain.rebuild_columns(DelayKind::Initial);
        Ok(rain)
    }

    pub fn update(&mut self, delta: Duration) {
        let row_count = self.config.row_count();
        let charset = self.config.charset.as_bytes();

        for column in &mut self.columns {
            column.update(delta, &self.config, row_count, charset, &mut self.rng);
        }

        self.collect_glyphs();
    }

    pub fn resize(&mut self, viewport_width_px: u32, viewport_height_px: u32) {
        self.config.viewport_width_px = viewport_width_px;
        self.config.viewport_height_px = viewport_height_px;
        self.rebuild_columns(DelayKind::Initial);
        self.collect_glyphs();
    }

    pub fn config(&self) -> &MatrixRainConfig {
        &self.config
    }

    /// Mutable access to the config for **live retuning of timing/appearance**
    /// (stream length, step, delays, colors) without rebuilding the field.
    /// Changing `cell_*`/`viewport_*` here won't relayout the columns — use
    /// [`MatrixRain::resize`] or a fresh [`MatrixRain::new`] for that.
    pub fn config_mut(&mut self) -> &mut MatrixRainConfig {
        &mut self.config
    }

    pub fn glyphs(&self) -> &[GlyphInstance] {
        &self.glyphs
    }

    pub fn column_count(&self) -> u32 {
        self.config.column_count()
    }

    pub fn row_count(&self) -> u32 {
        self.config.row_count()
    }

    fn rebuild_columns(&mut self, delay_kind: DelayKind) {
        let column_count = self.config.column_count();
        self.columns.clear();
        self.columns.reserve(column_count as usize);

        for index in 0..column_count {
            self.columns.push(Column::new(
                index,
                random_delay(&self.config, delay_kind, &mut self.rng),
            ));
        }
    }

    fn collect_glyphs(&mut self) {
        self.glyphs.clear();

        let row_count = self.config.row_count() as i32;
        let cell_width = self.config.cell_width_px as f32;
        let cell_height = self.config.cell_height_px as f32;

        for column in &self.columns {
            let Some(stream) = &column.stream else {
                continue;
            };

            for cell in &stream.cells {
                if cell.row < 0 || cell.row >= row_count {
                    continue;
                }

                let Some(color) = stream.color_for_cell(cell.row, &self.config) else {
                    continue;
                };

                let role = if cell.row == stream.head_row {
                    GlyphRole::Head
                } else {
                    GlyphRole::Trail
                };

                self.glyphs.push(GlyphInstance {
                    glyph: cell.glyph,
                    column: column.index,
                    row: cell.row,
                    position: Position {
                        x: column.index as f32 * cell_width,
                        y: cell.row as f32 * cell_height,
                    },
                    color,
                    role,
                });
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DelayKind {
    Initial,
    Respawn,
}

struct Column {
    index: u32,
    delay_remaining: Duration,
    stream: Option<Stream>,
}

impl Column {
    fn new(index: u32, delay_remaining: Duration) -> Self {
        Self {
            index,
            delay_remaining,
            stream: None,
        }
    }

    fn update(
        &mut self,
        delta: Duration,
        config: &MatrixRainConfig,
        row_count: u32,
        charset: &[u8],
        rng: &mut Rng,
    ) {
        if let Some(stream) = &mut self.stream {
            stream.update(delta, config, charset, rng);

            if stream.is_finished(row_count, config) {
                self.stream = None;
                self.delay_remaining = random_delay(config, DelayKind::Respawn, rng);
            }

            return;
        }

        if delta < self.delay_remaining {
            self.delay_remaining -= delta;
            return;
        }

        let stream_delta = delta - self.delay_remaining;
        self.delay_remaining = Duration::ZERO;
        let mut stream = Stream::new(random_stream_length(config, rng));
        stream.update(stream_delta, config, charset, rng);
        self.stream = Some(stream);
    }
}

struct Stream {
    length: u32,
    head_row: i32,
    step_accumulator: Duration,
    cells: Vec<GlyphCell>,
}

impl Stream {
    fn new(length: u32) -> Self {
        Self {
            length,
            head_row: -1,
            step_accumulator: Duration::ZERO,
            cells: Vec::with_capacity(length as usize),
        }
    }

    fn update(
        &mut self,
        delta: Duration,
        config: &MatrixRainConfig,
        charset: &[u8],
        rng: &mut Rng,
    ) {
        self.step_accumulator += delta;
        let step = Duration::from_millis(config.stream_step_ms);
        // Once the head has fully cleared the bottom (plus trail + fade), stop
        // spawning new cells so the stream can drain and eventually finish.
        // Without this the head cell is re-pushed every step, `cells` is never
        // empty, `is_finished` never fires, and the column never respawns.
        let finish_row = config.row_count() as i32
            + self.length as i32
            + self.fade_step_count(config) as i32;

        while self.step_accumulator >= step {
            self.step_accumulator -= step;
            self.head_row += 1;
            if self.head_row <= finish_row {
                self.cells.push(GlyphCell::new(
                    self.head_row,
                    random_glyph(charset, rng),
                    random_glyph_interval(config, rng),
                ));
            }
        }

        for cell in &mut self.cells {
            cell.update(delta, config, charset, rng);
        }

        let max_distance = self.length as i32 + self.fade_step_count(config) as i32;
        self.cells
            .retain(|cell| self.head_row.saturating_sub(cell.row) <= max_distance);
    }

    fn color_for_cell(&self, row: i32, config: &MatrixRainConfig) -> Option<Rgba> {
        let distance = self.head_row.saturating_sub(row);

        if distance == 0 {
            return Some(config.head_color);
        }

        if distance <= self.length as i32 {
            return Some(config.trail_color);
        }

        let fade_elapsed_ms = (distance - self.length as i32) as u64 * config.stream_step_ms;
        if fade_elapsed_ms >= config.fade_duration_ms {
            return None;
        }

        let remaining = config.fade_duration_ms - fade_elapsed_ms;
        let alpha = ((config.trail_color.a as u64 * remaining) / config.fade_duration_ms) as u8;
        Some(config.trail_color.with_alpha(alpha))
    }

    fn is_finished(&self, row_count: u32, config: &MatrixRainConfig) -> bool {
        let finish_row =
            row_count as i32 + self.length as i32 + self.fade_step_count(config) as i32;
        self.head_row > finish_row && self.cells.is_empty()
    }

    fn fade_step_count(&self, config: &MatrixRainConfig) -> u32 {
        div_ceil_u64(config.fade_duration_ms, config.stream_step_ms) as u32
    }
}

struct GlyphCell {
    row: i32,
    glyph: char,
    glyph_change_elapsed: Duration,
    glyph_change_interval: Duration,
}

impl GlyphCell {
    fn new(row: i32, glyph: char, glyph_change_interval: Duration) -> Self {
        Self {
            row,
            glyph,
            glyph_change_elapsed: Duration::ZERO,
            glyph_change_interval,
        }
    }

    fn update(
        &mut self,
        delta: Duration,
        config: &MatrixRainConfig,
        charset: &[u8],
        rng: &mut Rng,
    ) {
        self.glyph_change_elapsed += delta;

        while self.glyph_change_elapsed >= self.glyph_change_interval {
            self.glyph_change_elapsed -= self.glyph_change_interval;
            self.glyph = random_glyph(charset, rng);
            self.glyph_change_interval = random_glyph_interval(config, rng);
        }
    }
}

fn random_delay(config: &MatrixRainConfig, kind: DelayKind, rng: &mut Rng) -> Duration {
    let (min, max) = match kind {
        DelayKind::Initial => (
            config.initial_spawn_delay_min_ms,
            config.initial_spawn_delay_max_ms,
        ),
        DelayKind::Respawn => (config.respawn_delay_min_ms, config.respawn_delay_max_ms),
    };

    Duration::from_millis(random_range_inclusive(min, max, rng))
}

fn random_stream_length(config: &MatrixRainConfig, rng: &mut Rng) -> u32 {
    random_range_inclusive(
        config.min_stream_length as u64,
        config.max_stream_length as u64,
        rng,
    ) as u32
}

fn random_glyph_interval(config: &MatrixRainConfig, rng: &mut Rng) -> Duration {
    Duration::from_millis(random_range_inclusive(
        config.glyph_change_interval_min_ms,
        config.glyph_change_interval_max_ms,
        rng,
    ))
}

fn random_glyph(charset: &[u8], rng: &mut Rng) -> char {
    let index = rng.usize(0..charset.len());
    charset[index] as char
}

fn random_range_inclusive(min: u64, max: u64, rng: &mut Rng) -> u64 {
    if min == max {
        min
    } else {
        rng.u64(min..=max)
    }
}

fn div_ceil_u64(value: u64, divisor: u64) -> u64 {
    value.div_ceil(divisor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant_config() -> MatrixRainConfig {
        MatrixRainConfig {
            viewport_width_px: 90,
            viewport_height_px: 80,
            cell_width_px: 30,
            cell_height_px: 40,
            min_stream_length: 2,
            max_stream_length: 2,
            stream_step_ms: 10,
            initial_spawn_delay_min_ms: 0,
            initial_spawn_delay_max_ms: 0,
            respawn_delay_min_ms: 0,
            respawn_delay_max_ms: 0,
            glyph_change_interval_min_ms: 1000,
            glyph_change_interval_max_ms: 1000,
            fade_duration_ms: 20,
            charset: "AB".to_string(),
            ..MatrixRainConfig::default()
        }
    }

    #[test]
    fn seeded_runs_are_stable() {
        let config = instant_config();
        let mut first = MatrixRain::with_seed(config.clone(), 42).unwrap();
        let mut second = MatrixRain::with_seed(config, 42).unwrap();

        first.update(Duration::from_millis(30));
        second.update(Duration::from_millis(30));

        assert_eq!(first.glyphs(), second.glyphs());
    }

    #[test]
    fn positions_follow_cell_size() {
        let mut rain = MatrixRain::with_seed(instant_config(), 7).unwrap();
        rain.update(Duration::from_millis(10));

        let glyphs = rain.glyphs();
        assert_eq!(glyphs.len(), 3);
        assert!(glyphs.iter().any(|glyph| glyph.position.x == 60.0));
        assert!(glyphs.iter().all(|glyph| glyph.position.y == 0.0));
    }

    #[test]
    fn resize_rebuilds_column_layout() {
        let mut rain = MatrixRain::with_seed(instant_config(), 7).unwrap();
        assert_eq!(rain.column_count(), 3);

        rain.resize(120, 80);

        assert_eq!(rain.column_count(), 4);
        rain.update(Duration::from_millis(10));
        assert!(rain.glyphs().iter().all(|glyph| glyph.position.x < 120.0));
    }
}
