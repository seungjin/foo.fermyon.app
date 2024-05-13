#cargo build --target wasm32-wasi --release
cd rock;
cargo component build --release;

cd ..;
cargo component build --release;

wasm-tools compose -d ./rock/target/wasm32-wasi/release/rock.wasm ./target/wasm32-wasi/release/box.wasm -o service.wasm