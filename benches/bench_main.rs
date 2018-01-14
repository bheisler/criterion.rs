#[macro_use]
extern crate criterion;
extern crate walkdir;

mod no_plots;
mod compare_functions;
mod external_process;
mod iter_with_large_drop;
mod iter_with_large_setup;
mod iter_with_setup;
mod with_inputs;

criterion_main!{
    no_plots::benches,
    compare_functions::fibonaccis,
    external_process::benches,
    iter_with_large_drop::benches,
    iter_with_large_setup::benches,
    iter_with_setup::benches,
    with_inputs::benches
}