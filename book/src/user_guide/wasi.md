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
-criterion = "0.5.1"
+criterion = { version = "0.5.1", default-features = false }
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

Let's run the same benchmark with native, wasmer, wasmtime, nodejs, firefox, and chromium, and see how they compare. Step 1 is the generate the json files:

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

```properties
hex --bench --save-baseline firefox
hex --bench --export firefox | download
```

```properties
hex --bench --save-baseline chromium
hex --bench --export chromium | download
```

Step 2 is to tabulate the json files:

```properties
cargo bench --bench=hex -- --compare --baselines=native.json,wasmer.json,wasmtime.json,nodejs.json,firefox.json,chromium.json --compare-list
```

Console output:

```bash
faster_hex_decode
-----------------
native       1.00       3.6±0.02µs       ? ?/sec
wasmer      14.72      52.6±0.49µs       ? ?/sec
wasmtime    16.83      60.1±0.53µs       ? ?/sec
chromium    17.66      63.1±0.70µs       ? ?/sec
firefox     19.82      70.8±6.53µs       ? ?/sec
nodejs      20.76      74.2±0.34µs       ? ?/sec

faster_hex_decode_fallback
--------------------------
native       1.00      10.9±0.12µs       ? ?/sec
wasmer       1.49      16.2±0.04µs       ? ?/sec
firefox      1.61      17.6±0.51µs       ? ?/sec
wasmtime     1.65      18.1±0.73µs       ? ?/sec
chromium     1.87      20.4±0.16µs       ? ?/sec
nodejs       2.30      25.1±0.56µs       ? ?/sec

faster_hex_decode_unchecked
---------------------------
native       1.00   1239.7±16.97ns       ? ?/sec
wasmer      14.27      17.7±0.35µs       ? ?/sec
wasmtime    14.36      17.8±0.23µs       ? ?/sec
firefox     14.38      17.8±1.83µs       ? ?/sec
chromium    16.53      20.5±0.28µs       ? ?/sec
nodejs      20.36      25.2±0.15µs       ? ?/sec

faster_hex_encode
-----------------
native       1.00     948.3±5.47ns       ? ?/sec
wasmer      19.17      18.2±0.36µs       ? ?/sec
chromium    21.25      20.2±0.17µs       ? ?/sec
nodejs      22.85      21.7±0.09µs       ? ?/sec
wasmtime    24.01      22.8±0.53µs       ? ?/sec
firefox     30.68      29.1±0.89µs       ? ?/sec

faster_hex_encode_fallback
--------------------------
native       1.00      11.1±0.20µs       ? ?/sec
firefox      1.98      21.9±0.57µs       ? ?/sec
chromium     2.04      22.7±0.20µs       ? ?/sec
wasmtime     2.05      22.8±0.13µs       ? ?/sec
wasmer       2.06      22.8±0.15µs       ? ?/sec
nodejs       2.38      26.4±0.09µs       ? ?/sec

hex_decode
----------
native       1.00     244.6±2.36µs       ? ?/sec
firefox      1.66    405.7±14.22µs       ? ?/sec
wasmer       1.72     421.4±9.65µs       ? ?/sec
wasmtime     1.73     423.0±3.00µs       ? ?/sec
nodejs       2.00     490.3±3.49µs       ? ?/sec
chromium     2.81    688.5±12.23µs       ? ?/sec

hex_encode
----------
native       1.00      69.2±0.40µs       ? ?/sec
wasmtime     1.18      81.7±0.38µs       ? ?/sec
wasmer       1.46     100.9±1.22µs       ? ?/sec
nodejs       2.20     152.5±1.93µs       ? ?/sec
firefox      3.25     224.8±7.53µs       ? ?/sec
chromium     4.08     282.7±4.19µs       ? ?/sec

rustc_hex_decode
----------------
native       1.00     103.1±2.78µs       ? ?/sec
wasmer       1.33     136.8±4.06µs       ? ?/sec
wasmtime     1.38     142.3±3.31µs       ? ?/sec
firefox      1.50     154.7±4.80µs       ? ?/sec
nodejs       1.78     183.3±2.02µs       ? ?/sec
chromium     2.04     210.0±3.37µs       ? ?/sec

rustc_hex_encode
----------------
native       1.00      30.9±0.42µs       ? ?/sec
wasmtime     2.24      69.1±0.36µs       ? ?/sec
wasmer       2.25      69.6±0.74µs       ? ?/sec
nodejs       2.40      74.2±1.94µs       ? ?/sec
chromium     2.67      82.6±2.61µs       ? ?/sec
firefox      3.31     102.2±2.66µs       ? ?/sec
```

# Caveats and pitfalls

## Warm-up and JIT

Most WebAssembly environments don't reach peak performance until the code has been running for a little while. This means the warm-up step is essential and skipping it (by setting it to 0 seconds or using the `--quick` flag) will lead to inaccurate results.

## Re-running benchmarks in [webassembly.sh](https://webassembly.sh/)

The WebAssembly.sh website shims the WebAssembly System Interface (WASI) required by Criterion. But this shim is not perfect and causes criterion to fail spectacularly when run more then once. Should this happen to you, reloading your browser window should work around the problem.

## Wasm and default-features.

Criterion's default features have to be disabled when compiling to wasm. Failing to do so will trigger a compilation error. If see an error saying a feature is incompatible with wasm, open your `Cargo.toml` file and make this change:

```diff
[dev-dependencies]
-criterion = "0.5.1"
+criterion = { version = "0.5.1", default-features = false }
```
