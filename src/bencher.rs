use test;

use time::traits::Nanosecond;
use time::types::Ns;
use time;

/// Helper `struct` to build benchmark functions that follow the setup - bench - teardown pattern.
#[experimental]
pub struct Bencher {
    end: Ns<u64>,
    iterations: u64,
    start: Ns<u64>,
}

#[experimental]
impl Bencher {
    /// Callback for benchmark functions to benchmark a routine
    ///
    /// A benchmark function looks like this:
    ///
    ///     fn bench_me(b: &mut Bencher) {
    ///         // Setup
    ///
    ///         // Bench
    ///         b.iter(|| {
    ///             // Routine to benchmark
    ///         })
    ///
    ///         // Teardown
    ///     }
    ///
    /// See `Criterion::bench()` for details about how to run this benchmark function
    #[experimental]
    pub fn iter<T>(&mut self, routine: || -> T) {
        self.start = time::now();
        for _ in range(0, self.iterations) {
            test::black_box(routine());
        }
        self.end = time::now();
    }
}

// XXX These functions should be inside the `impl Bencher`, but they would leak into the API
pub fn new(iterations: u64) -> Bencher {
    Bencher {
        iterations: iterations,
        end: 0.ns(),
        start: 0.ns(),
    }
}

pub fn elapsed(b: &Bencher) -> Ns<u64> {
    b.end - b.start
}
