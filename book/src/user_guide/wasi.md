## WASI cook book

### Install cargo-wasi

```bash
cargo install cargo-wasi
```

### Building

```bash
cargo build --bench=hex --target wasm32-wasi
```

```bash
cargo wasi build --bench=hex
```

### Running wasmer

```bash
wasmer run --dir . target/wasm32-wasi/debug/deps/*.wasm -- --bench
```

Or if you're using `cargo-wasi`:
```bash
wasmer run --dir . target/wasm32-wasi/debug/deps/*.wasi.wasm -- --bench
```

### Running with nodejs

TBD

### Running in a browser with webassembly.sh

TBD

### Exporting results

TBD

### Comparing results

TBD
