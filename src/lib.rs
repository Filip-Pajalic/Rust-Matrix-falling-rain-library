pub mod config;
mod error;
pub mod matrix;

#[cfg(feature = "macroquad-renderer")]
pub mod renderers;

pub use config::{MatrixRainConfig, Rgba};
pub use error::MatrixError;
pub use matrix::{GlyphInstance, GlyphRole, MatrixRain, MatrixRenderer, Position};

#[cfg(feature = "macroquad-renderer")]
pub use renderers::macroquad::MacroquadRenderer;
