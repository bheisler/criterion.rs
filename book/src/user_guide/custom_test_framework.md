# Custom Test Framework

Nightly versions of the rust compiler support custom test frameworks. Criterion.rs provides an
experimental implementation of a custom test framework, meaning that you can use `#[criterion]`
attributes to mark your benchmarks instead of the normal `criterion_group!/criterion_main!` macros.
Right now this requires some unstable features, but at some point in the future
`criterion_group!/criterion_main!` will be deprecated and `#[criterion]` will become the standard
way to define a Criterion.rs benchmark. If you'd like to try this feature out early, see the
documentation below.

## Using `#[criterion]`

Since custom test frameworks are still unstable, you will need to be using a recent nightly compiler.
Once that's installed, add the dependencies to your Cargo.toml:

```toml
[dev-dependencies]
criterion = "0.5"
criterion-macro = "0.4"
```

Note that for `#[criterion]` benchmarks, we don't need to disable the normal testing harness
as we do with regular Criterion.rs benchmarks.

Let's take a look at an example benchmark (note that this example assumes you're using Rust 2018):

```rust
#![feature(custom_test_frameworks)]
#![test_runner(criterion::runner)]

use criterion::Criterion;
use criterion_macro::criterion;
use std::hint::black_box;

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(50)
}

#[criterion]
fn bench_simple(c: &mut Criterion) {
    c.bench_function("Fibonacci-Simple", |b| b.iter(|| fibonacci(black_box(10))));
}

#[criterion(custom_criterion())]
fn bench_custom(c: &mut Criterion) {
    c.bench_function("Fibonacci-Custom", |b| b.iter(|| fibonacci(black_box(20))));
}
```

The first thing to note is that we enable the `custom_test_framework` feature and declare that we
want to use `criterion::runner` as the test runner. We also import `criterion_macro::criterion`,
which is the `#[criterion]` macro itself. In future versions this will likely be re-exported from
the `criterion` crate so that it can be imported from there, but for now we have to import it from
`criterion_macro`.

After that we define our old friend the Fibonacci function and the benchmarks. To create a
benchmark with `#[criterion]` you simply attach the attribute to a function that accepts an `&mut
Criterion`. To provide a custom Criterion object (to override default settings or similar) you can
instead use `#[criterion(<some_expression_that_returns_a_criterion_object>)]` - here we're calling
the `custom_criterion` function. And that's all there is to it!

Keep in mind that in addition to being built on unstable compiler features, the API design for
Criterion.rs and its test framework is still experimental. The macro subcrate will respect SemVer,
but future breaking changes are quite likely.
