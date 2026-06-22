# Matrix Rain

Renderer-agnostic Rust library for Matrix-style falling code.

![Matrix Code Effect](resources/matrix.gif)

## Library Usage

```rust
use matrix_rain::{MatrixRain, MatrixRainConfig};
use std::time::Duration;

let mut rain = MatrixRain::new(MatrixRainConfig::default())?;
rain.update(Duration::from_millis(16));

for glyph in rain.glyphs() {
    // Draw glyph.glyph at glyph.position with glyph.color.
}
# Ok::<(), matrix_rain::MatrixError>(())
```

The default crate is simulation-only. It exposes glyph positions, colors, and characters so any renderer can draw them.

## Features

- `default`: no renderer and no YAML parser.
- `yaml`: enables `MatrixRainConfig::from_yaml_str` and native `from_yaml_file`.
- `macroquad-renderer`: enables the optional macroquad renderer adapter.
- `demo`: enables the macroquad demo binary and YAML config loading.

## Demo

Run the native macroquad demo:

```sh
cargo run --bin matrix-rain-demo --features demo
```

Build native release:

```sh
cargo build --release --bin matrix-rain-demo --features demo
```

Build the browser demo:

```sh
make wasm
make serve
```

## Configuration

The demo reads `config.yaml` on native builds and embeds it for WASM builds. The same fields map to `MatrixRainConfig`, with `debug_overlay` used only by the demo.

The demo is an integration example. The library API does not expose macroquad types unless `macroquad-renderer` is enabled.
