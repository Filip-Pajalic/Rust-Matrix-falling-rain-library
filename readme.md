# Matrix Falling Code in Rust

This is a Rust implementation of the Matrix falling code effect.

It uses the `raylib-rs` crate for graphics, which requires a C compiler (like Clang or GCC) to build.

For a release build, a `config.yaml` file and a `resources` folder containing the font `matrix-code.ttf` must be in the same directory as the executable.

This project was primarily a practice exercise to improve my Rust skills. It might not be perfect, but it works!

![Matrix Code Effect](resources/matrix.gif)

## Prerequisites

* Rust toolchain installed.
* A C compiler (Clang or GCC) available on your system.

## Configuration

* Place `config.yaml` in the same directory as the release executable.
* Create a `resources` folder in the same directory as the release executable and place `matrix-code.ttf` inside it.

## Building and Running

1.  **Build:** `cargo build --release`
2.  **Copy assets:** Copy `config.yaml` and the `resources` folder to the `target/release` directory.
3.  **Run:** Execute the binary located in `target/release`.
