use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub world: World,
    pub glyph: Glyph,
    pub debug: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct World {
    pub window_width_px: i32,
    pub window_height_px: i32,
    pub font_size_px: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Glyph {
    pub child_spawn_interval_ms: i32,
}

impl Config {
    pub fn from_file(_path: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let config_str = std::fs::read_to_string(_path)
                .unwrap_or_else(|_| panic!("Failed to read config file: {}", _path));
            serde_yaml::from_str(&config_str)
                .unwrap_or_else(|_| panic!("Failed to parse config file: {}", _path))
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // Embed the config file at compile time for WASM
            let config_str = include_str!("../../config.yaml");
            serde_yaml::from_str(config_str)
                .unwrap_or_else(|_| panic!("Failed to parse embedded config"))
        }
    }
}