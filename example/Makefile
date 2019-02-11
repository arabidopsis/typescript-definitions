

bindgen:
	cargo +nightly build --target wasm32-unknown-unknown
	@mkdir pkg 2> /dev/null || true
	wasm-bindgen target/wasm32-unknown-unknown/debug/mywasm.wasm --typescript --out-dir pkg/

wasm:
	wasm-pack build

install:
	rustup target add wasm32-unknown-unknown --toolchain nightly
	cargo +nightly install wasm-bindgen-cli
