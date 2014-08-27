#![feature(macro_rules, overloaded_calls, phase, unboxed_closures)]

#[phase(plugin, link)]
extern crate log;
extern crate serialize;
extern crate simplot;
extern crate stats;
extern crate test;
extern crate time;

use std::fmt::Show;
use std::io::Command;
use std::time::Duration;

mod analysis;
mod estimate;
mod format;
mod fs;
mod kde;
mod plot;
mod program;
mod report;
mod routine;

/// Helper `struct` to build benchmark functions that follow the setup - bench - teardown pattern.
#[experimental]
pub struct Bencher {
    iters: u64,
    ns_end: u64,
    ns_start: u64,
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
        self.ns_start = time::precise_time_ns();
        for _ in range(0, self.iters) {
            test::black_box(routine());
        }
        self.ns_end = time::precise_time_ns();
    }
}

/// The "criterion" for the benchmark, which is also the benchmark "manager"
#[experimental]
pub struct Criterion {
    confidence_level: f64,
    measurement_time_ns: u64,
    noise_threshold: f64,
    nresamples: uint,
    sample_size: uint,
    significance_level: f64,
    warm_up_time_ns: u64,
}

#[experimental]
impl Criterion {
    /// This is the default criterion:
    ///
    /// * Confidence level: 0.95
    /// * Measurement time: 1 s
    /// * Noise threshold: 0.01 (1%)
    /// * Bootstrap with 100 000 resamples
    /// * Sample size: 100 measurements
    /// * Significance level: 0.05
    /// * Warm-up time: 1 s
    // TODO (UFCS) remove this method and implement the `Default` trait
    #[experimental]
    pub fn default() -> Criterion {
        Criterion {
            confidence_level: 0.95,
            measurement_time_ns: 1_000_000_000,
            noise_threshold: 0.01,
            nresamples: 100_000,
            sample_size: 100,
            significance_level: 0.05,
            warm_up_time_ns: 1_000_000_000,
        }
    }

    /// Changes the confidence level
    ///
    /// The confidence level is used to calculate the confidence intervals of the estimates
    #[experimental]
    pub fn confidence_level(&mut self, cl: f64) -> &mut Criterion {
        assert!(cl > 0.0 && cl < 1.0);

        self.confidence_level = cl;
        self
    }

    /// Change the measurement time
    ///
    /// The program/function under test is iterated for `measurement_time` ms. And the average run
    /// time is reported as a measurement
    #[experimental]
    pub fn measurement_time(&mut self, dur: Duration) -> &mut Criterion {
        let ns = dur.num_nanoseconds().expect("duration overflow") as u64;

        assert!(ns > 0);

        self.measurement_time_ns = ns;
        self
    }

    /// Changes the noise threshold
    ///
    /// When comparing benchmark results, only relative changes of the execution time above this
    /// threshold are considered significant
    #[experimental]
    pub fn noise_threshold(&mut self, nt: f64) -> &mut Criterion {
        assert!(nt >= 0.0);

        self.noise_threshold = nt;
        self
    }

    /// Changes the number of resamples
    ///
    /// Number of resamples to use for bootstraping via case resampling
    #[experimental]
    pub fn nresamples(&mut self, n: uint) -> &mut Criterion {
        assert!(n > 0);

        self.nresamples = n;
        self
    }

    /// Changes the size of a sample
    ///
    /// A sample consists of severals measurements
    #[experimental]
    pub fn sample_size(&mut self, n: uint) -> &mut Criterion {
        assert!(n > 0);

        self.sample_size = n;
        self
    }

    /// Changes the significance level
    ///
    /// Significance level to use for hypothesis testing
    #[experimental]
    pub fn significance_level(&mut self, sl: f64) -> &mut Criterion {
        assert!(sl > 0.0 && sl < 1.0);

        self.significance_level = sl;
        self
    }

    /// Changes the warm up time
    ///
    /// The program/function under test is executed during `warm_up_time` ms before the real
    /// measurement starts
    #[experimental]
    pub fn warm_up_time(&mut self, dur: Duration) -> &mut Criterion {
        let ns = dur.num_nanoseconds().expect("duration overflow") as u64;

        assert!(ns > 0);

        self.warm_up_time_ns = ns;
        self
    }

    /// Benchmark a function. See `Bench::iter()` for an example of how `fun` should look
    #[experimental]
    pub fn bench(&mut self, id: &str, f: |&mut Bencher|:'static) -> &mut Criterion {
        analysis::function(id, f, self);

        self
    }

    /// Benchmark a family of functions
    ///
    /// `fun` will be benchmarked under each input
    ///
    /// For example, if you want to benchmark `Vec::from_elem` with different size, use these
    /// arguments:
    ///
    ///     let fun = |b, n| Vec::from_elem(n, 0u);
    ///     let inputs = [100, 10_000, 1_000_000];
    ///
    /// This is equivalent to calling `bench` on each of the following functions:
    ///
    ///     let fun1 = |b| Vec::from_elem(100, 0u);
    ///     let fun2 = |b| Vec::from_elem(10_000, 0u);
    ///     let fun3 = |b| Vec::from_elem(1_000_000, 0u);
    #[experimental]
    pub fn bench_with_inputs<I: Show>(
        &mut self,
        id: &str,
        f: |&mut Bencher, &I|:'static,
        inputs: &[I],
    ) -> &mut Criterion {
        analysis::function_with_inputs(id, f, inputs, self);

        self
    }

    /// Benchmark an external program
    ///
    /// The program must conform to the following specification:
    ///
    ///     extern crate time;
    ///
    ///     fn main() {
    ///         // Optional: Get the program arguments
    ///         let args = std::os::args();
    ///
    ///         for line in std::io::stdio::stdin().lines() {
    ///             // Get number of iterations to do
    ///             let iters: u64 = from_str(line.unwrap().as_slice().trim()).unwrap();
    ///
    ///             // Setup
    ///
    ///             // (For best results, use a monotonic timer)
    ///             let start = time::precise_time_ns();
    ///             for _ in range(0, iters) {
    ///                 // Routine to benchmark goes here
    ///             }
    ///             let end = time::precise_time_ns();
    ///
    ///             // Teardown
    ///
    ///             // Report back the time (in nanoseconds) required to execute the routine
    ///             // `iters` times
    ///             println!("{}", end - start);
    ///         }
    ///     }
    ///
    /// For example, to benchmark a python script use the following command
    ///
    ///     let cmd = Command::new("python3").args(["-O", "clock.py"]);
    #[experimental]
    pub fn bench_program(
        &mut self,
        id: &str,
        prog: &Command,
    ) -> &mut Criterion {
        analysis::program(id, prog, self);

        self
    }

    /// Benchmark an external program under various inputs
    ///
    /// For example, to benchmark a python script under various inputs, use this combination:
    ///
    ///     let cmd = Command::new("python3").args(["-O", "fib.py"]);
    ///     let inputs = [5u, 10, 15];
    ///
    /// This is equivalent to calling `bench_prog` on each of the following commands:
    ///
    ///     let cmd1 = Command::new("python3").args(["-O", "fib.py", "5"]);
    ///     let cmd2 = Command::new("python3").args(["-O", "fib.py", "10"]);
    ///     let cmd2 = Command::new("python3").args(["-O", "fib.py", "15"]);
    #[experimental]
    pub fn bench_program_with_inputs<I: Show>(
        &mut self,
        id: &str,
        prog: &Command,
        inputs: &[I],
    ) -> &mut Criterion {
        analysis::program_with_inputs(id, prog, inputs, self);

        self
    }

    /// Summarize the results stored under the `.criterion/${id}` folder
    ///
    /// Note that `bench_family` and `bench_prog_family` internally call the `summarize` method
    #[experimental]
    pub fn summarize(&mut self, id: &str) -> &mut Criterion {
        print!("Summarizing results of {}... ", id);
        plot::summarize(&Path::new(".criterion").join(id), id);
        println!("DONE\n");

        self
    }
}
