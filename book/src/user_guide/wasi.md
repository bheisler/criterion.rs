# WebAssebly/WASI benchmarking

Criterion benchmarks can be compiled to WebAssembly/WASI. This lets you performance test your code with runtimes such as [wasmer](https://wasmer.io/) and [wasmtime](https://wasmtime.dev/), as well as JavaScript environments such as [NodeJS](https://nodejs.org/en/) or web browsers ([FireFox](https://www.mozilla.org/en-US/firefox/new/), [Chrome](https://www.google.com/chrome/), [Safari](https://www.apple.com/safari/)).

## Adding the `wasm32-wasi` target

We're cross-compiling to WebAssembly System Interface (aka WASI) so we have to add the right target. If you forget this step then your benchmarks will not compile and you'll be shown a gentle reminder. We'll use the [rustup](https://rustup.rs/) tool:

```properties
rustup target add wasm32-wasi
```

## Install cargo-wasi

Next we'll install the `cargo-wasi` command. While this is not required by Criterion, it does make it a lot easier to build WASI programs.

```properties
cargo install cargo-wasi
```

## Building

With `wasm32-wasi` and `cargo-wasi` installed, we're almost set to compile our benchmarks. Just one thing left: Telling Criterion not to use default features such as rayon which are not yet available in WASI:

```diff
[dev-dependencies]
-criterion = "0.4"
+criterion = { version = "0.4", default-features = false }
```

Compiling the benchmark with `cargo-wasi` will automatically select the right target and optimize the resulting file. Here I'm using the [hex](https://crates.io/crates/hex) crate as an example:

```properties
cargo wasi build --bench=hex --release
```

But it is also possible to compile it without `cargo-wasi`:

```properties
cargo build --bench=hex --release --target wasm32-wasi
```

There should now be a `.wasm` file in `target/wasm32-wasi/release/deps/`. If you used `cargo-wasi` then there'll be both an optimized and an un-optimized version. Let's copy the newest WASM file out to the top-level directory for convenience:

```console
cp `ls -t target/wasm32-wasi/release/deps/*.wasm | head -n 1` hex.wasm
```

## Running with wasmer/wasmtime

```properties
wasmer run --dir=. hex.wasm -- --bench
```

```properties
wasmtime run --dir=. hex.wasm -- --bench
```

## Running with nodejs

Running in NodeJS can be done via wasmer-cli:

```properties
npm install -g @wasmer/cli
```

Once `wasmer-js` is installed, the interface is identical to plain `wasmer`:

```properties
wasmer-js run --dir=. hex.wasm -- --bench
```

## Running in a browser with webassembly.sh

Browsers do not natively support WASI but there are workarounds. The easiest solution is [webassembly.sh](https://webassembly.sh/). This website shims WASI using an in-memory filesystem.

To use the website, go to https://webassembly.sh/, drag-and-drop the `hex.wasm` file into the browser window, and type:

```properties
hex --bench
```

Once you start the benchmark, the browser window will freeze until the results are ready. This is an unfortunate limitation of running WebAssembly in the browser.

### Exporting results

Writing benchmark results to an in-memory filesystem in the browser is not very useful on its own. Luckily the results are easy to export and download as JSON:

```properties
hex --bench --export=base | download
```

## Comparing results

Let's run the same benchmark with native, wasmer, wasmtime, nodejs, firefox, and chrome, and see how they compare.

```properties
wasmer run --dir=. hex.wasm -- --bench --save-baseline wasmer
wasmer run --dir=. hex.wasm -- --bench --export wasmer > wasmer.json
```

```properties
wasmtime run --dir=. hex.wasm -- --bench --save-baseline wasmtime
wasmtime run --dir=. hex.wasm -- --bench --export wasmtime > wasmtime.json
```

```properties
wasmer-js run --dir=. hex.wasm -- --bench --save-baseline nodejs
wasmer-js run --dir=. hex.wasm -- --bench --export nodejs > nodejs.json
```

# Caveats and pitfalls

