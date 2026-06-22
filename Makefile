WASM_TARGET=wasm32-unknown-unknown
WASM_OUT=matrix-rain-demo.wasm

.PHONY: native wasm wasm-release serve clean

native:
	cargo build --bin matrix-rain-demo --features demo

native-release:
	cargo build --bin matrix-rain-demo --release --features demo

wasm:
	cargo build --bin matrix-rain-demo --target $(WASM_TARGET) --features demo
	cp target/$(WASM_TARGET)/debug/matrix-rain-demo.wasm $(WASM_OUT)

wasm-release:
	cargo build --bin matrix-rain-demo --release --target $(WASM_TARGET) --features demo
	cp target/$(WASM_TARGET)/release/matrix-rain-demo.wasm $(WASM_OUT)

serve:
	python3 -m http.server 8000

clean:
	cargo clean
	rm -f $(WASM_OUT)
