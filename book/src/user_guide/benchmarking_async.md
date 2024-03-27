## Benchmarking async functions

As of version 0.3.4, Criterion.rs has optional support for benchmarking async functions.
Benchmarking async functions works just like benchmarking regular functions, except that the
caller must provide a futures executor to run the benchmark in.

### Example:

```rust
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use criterion::async_executor::FuturesExecutor;

// Here we have an async function to benchmark
async fn do_something(size: usize) {
    // Do something async with the size
}

fn from_elem(c: &mut Criterion) {
    let size: usize = 1024;

    c.bench_with_input(BenchmarkId::new("input_example", size), &size, |b, &s| {
        // Insert a call to `to_async` to convert the bencher to async mode.
        // The timing loops are the same as with the normal bencher.
        b.to_async(FuturesExecutor).iter(|| do_something(s));
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
```

As can be seen in the code above, to benchmark async functions we must provide an async runtime to
the bencher to run the benchmark in. The runtime structs are listed in the table below.

### Enabling Async Benchmarking

To enable async benchmark support, Criterion.rs must be compiled with one or more of the following
features, depending on which futures executor(s) you want to benchmark on. It is recommended to use
the same executor that you would use in production. If your executor is not listed here, you can
implement the `criterion::async_executor::AsyncExecutor` trait for it to add support, or send a pull
request.

| Crate     | Feature                       | Executor Struct                                                    |
| --------- | ----------------------------- | ------------------------------------------------------------------ |
| Tokio     | "async_tokio"                 | In `tokio::runtime`, `Runtime`, `&Runtime`, `Handle`, or `&Handle` |
| async-std | "async_std" (note underscore) | `AsyncStdExecutor`                                                 |
| Smol      | "async_smol"                  | `SmolExecutor`                                                     |
| futures   | "async_futures"               | `FuturesExecutor`                                                  |
| Other     | "async"                       |                                                                    |

### Considerations when benchmarking async functions

Async functions naturally result in more measurement overhead than synchronous functions. It is
recommended to prefer synchronous functions when benchmarking where possible, especially for small
functions.