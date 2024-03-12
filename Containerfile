FROM scratch
COPY spin.toml /spin.toml
COPY target/wasm32-wasi/release/*.wasm /target/wasm32-wasi/release/
ENTRYPOINT ["/spin.toml"]