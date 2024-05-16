#cargo build --target wasm32-wasi --release
cd ../foo;
cargo component build --release;

cd ../spin-app;
cargo component build --release;

wasm-tools compose -d ../foo/target/wasm32-wasi/release/foo.wasm ./target/wasm32-wasi/release/box.wasm -o service.wasm