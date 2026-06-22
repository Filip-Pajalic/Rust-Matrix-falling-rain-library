# WASM build

Build the browser demo and copy it beside `index.html`:

```sh
make wasm
```

Build the optimized browser demo:

```sh
make wasm-release
```

Serve the current directory:

```sh
make serve
```

Then open `http://localhost:8000`.

The WASM build targets the `matrix-rain-demo` binary with the explicit `demo` feature. The reusable library itself has no default renderer and does not produce the browser app artifact.
