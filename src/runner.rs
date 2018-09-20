//! custom test-framework runner
pub use criterion_macro::*;
use ::*;

/// Benchmark definition for the custom benchmark runner
#[doc(hidden)]
pub struct CtfBenchmark {
    /// Benchmark name
    pub name: &'static str,
    /// Benchmark function
    pub fun: fn(&mut Criterion) ->(),
}

impl CtfBenchmark {
    fn run(&self) {
        let mut criterion: Criterion = Criterion::default();
        (self.fun)(&mut criterion);
    }
}

/// Custom benchmark runner
#[doc(hidden)]
pub fn runner(benchmark_groups: &[&CtfBenchmark]) {
    init_logging();
    for g in benchmark_groups {
        g.run();
    }
    Criterion::default()
        .configure_from_args()
        .final_summary();
}
