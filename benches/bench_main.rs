#[macro_use]
extern crate criterion;
extern crate walkdir;

mod benchmarks;

criterion_main!{
    benchmarks::compare_functions::fibonaccis,
    benchmarks::external_process::benches,
    benchmarks::iter_with_large_drop::benches,
    benchmarks::iter_with_large_setup::benches,
    benchmarks::iter_with_setup::benches,
    benchmarks::with_inputs::benches,
    benchmarks::special_characters::benches,
}
