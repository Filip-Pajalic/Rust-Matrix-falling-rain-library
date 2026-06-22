use serde::{Deserialize, Serialize};

use crate::MatrixError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const MATRIX_GREEN: Self = Self::new(0, 255, 0, 220);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }
}

impl Default for Rgba {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MatrixRainConfig {
    pub viewport_width_px: u32,
    pub viewport_height_px: u32,
    pub cell_width_px: u32,
    pub cell_height_px: u32,
    pub min_stream_length: u32,
    pub max_stream_length: u32,
    pub stream_step_ms: u64,
    pub initial_spawn_delay_min_ms: u64,
    pub initial_spawn_delay_max_ms: u64,
    pub respawn_delay_min_ms: u64,
    pub respawn_delay_max_ms: u64,
    pub glyph_change_interval_min_ms: u64,
    pub glyph_change_interval_max_ms: u64,
    pub fade_duration_ms: u64,
    pub charset: String,
    pub head_color: Rgba,
    pub trail_color: Rgba,
}

impl Default for MatrixRainConfig {
    fn default() -> Self {
        Self {
            viewport_width_px: 1920,
            viewport_height_px: 1080,
            cell_width_px: 30,
            cell_height_px: 40,
            min_stream_length: 10,
            max_stream_length: 28,
            stream_step_ms: 80,
            initial_spawn_delay_min_ms: 100,
            initial_spawn_delay_max_ms: 9000,
            respawn_delay_min_ms: 100,
            respawn_delay_max_ms: 1400,
            glyph_change_interval_min_ms: 100,
            glyph_change_interval_max_ms: 700,
            fade_duration_ms: 600,
            charset: "EIOPQRTUWY012345789".to_string(),
            head_color: Rgba::WHITE,
            trail_color: Rgba::MATRIX_GREEN,
        }
    }
}

impl MatrixRainConfig {
    pub fn validate(&self) -> Result<(), MatrixError> {
        if self.viewport_width_px == 0 || self.viewport_height_px == 0 {
            return Err(MatrixError::invalid_config(
                "viewport dimensions must be greater than zero",
            ));
        }

        if self.cell_width_px == 0 || self.cell_height_px == 0 {
            return Err(MatrixError::invalid_config(
                "cell dimensions must be greater than zero",
            ));
        }

        if self.min_stream_length == 0 {
            return Err(MatrixError::invalid_config(
                "min_stream_length must be greater than zero",
            ));
        }

        if self.max_stream_length < self.min_stream_length {
            return Err(MatrixError::invalid_config(
                "max_stream_length must be greater than or equal to min_stream_length",
            ));
        }

        if self.stream_step_ms == 0 {
            return Err(MatrixError::invalid_config(
                "stream_step_ms must be greater than zero",
            ));
        }

        validate_range(
            self.initial_spawn_delay_min_ms,
            self.initial_spawn_delay_max_ms,
            "initial spawn delay",
        )?;
        validate_range(
            self.respawn_delay_min_ms,
            self.respawn_delay_max_ms,
            "respawn delay",
        )?;
        validate_range(
            self.glyph_change_interval_min_ms,
            self.glyph_change_interval_max_ms,
            "glyph change interval",
        )?;

        if self.fade_duration_ms == 0 {
            return Err(MatrixError::invalid_config(
                "fade_duration_ms must be greater than zero",
            ));
        }

        if self.charset.is_empty() {
            return Err(MatrixError::invalid_config("charset must not be empty"));
        }

        if !self.charset.is_ascii() {
            return Err(MatrixError::invalid_config(
                "charset must be ASCII so glyph selection is byte-stable",
            ));
        }

        Ok(())
    }

    pub fn column_count(&self) -> u32 {
        div_ceil(self.viewport_width_px, self.cell_width_px)
    }

    pub fn row_count(&self) -> u32 {
        div_ceil(self.viewport_height_px, self.cell_height_px)
    }

    #[cfg(feature = "yaml")]
    pub fn from_yaml_str(config: &str) -> Result<Self, MatrixError> {
        let config = serde_yaml::from_str::<Self>(config)
            .map_err(|error| MatrixError::invalid_config(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    #[cfg(all(feature = "yaml", not(target_arch = "wasm32")))]
    pub fn from_yaml_file(path: impl AsRef<std::path::Path>) -> Result<Self, MatrixError> {
        let path = path.as_ref();
        let config = std::fs::read_to_string(path).map_err(|error| {
            MatrixError::resource(format!("failed to read {}: {error}", path.display()))
        })?;
        Self::from_yaml_str(&config)
    }
}

fn validate_range(min: u64, max: u64, name: &str) -> Result<(), MatrixError> {
    if max < min {
        return Err(MatrixError::invalid_config(format!(
            "{name} max must be >= min"
        )));
    }

    Ok(())
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}
