use std::iter;
use test;
use time;

use stream::Stream;

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

pub enum Target<'a> {
    Function(Option<|&mut Bencher|:'a>),
    Program(Stream),
}

impl<'a> Target<'a> {
    pub fn warm_up(&mut self, how_long_ns: u64) -> (u64, u64) {
        let ns_start = time::precise_time_ns();

        match *self {
            Function(ref mut fun_opt) => {
                let fun = fun_opt.take_unwrap();
                let mut b = Bencher { iters: 1, ns_end: 0, ns_start: 0 };

                loop {
                    fun(&mut b);

                    if time::precise_time_ns() - ns_start > how_long_ns {
                        *fun_opt = Some(fun);

                        return (b.ns_end - b.ns_start, b.iters);
                    }

                    b.iters *= 2;
                }
            },
            Program(ref mut prog) => {
                let mut iters = 1;

                loop {
                    let elapsed =
                        from_str(prog.send(iters).recv().as_slice().trim()).
                            expect("Couldn't parse the program output");

                    if time::precise_time_ns() - ns_start > how_long_ns {
                        return (elapsed, iters);
                    }

                    iters *= 2;
                }
            },
        }
    }

    // Collects measurements, each one with different number of iterations: d, 2*d, 3*d, ..., n*d
    pub fn bench(&mut self, n: uint, d: u64) -> Vec<f64> {
        let mut sample = Vec::with_capacity(n);

        match *self {
            Function(ref mut fun_opt) => {
                let mut b = Bencher { iters: n as u64 * d, ns_start: 0, ns_end: 0 };
                let fun = fun_opt.take_unwrap();

                for _ in range(0, n) {
                    fun(&mut b);
                    sample.push(b.ns_end - b.ns_start);
                    b.iters -= d;
                }

                *fun_opt = Some(fun);
            },
            Program(ref mut prog) => {
                let mut iters = n as u64 * d;

                for _ in range(0, n) {
                    prog.send(iters);
                    iters -= d;
                }

                for _ in range(0, n) {
                    let msg = prog.recv();
                    let msg = msg.as_slice().trim();

                    sample.push(from_str(msg).expect("Couldn't parse program output"));
                }
            },
        }

        sample.move_iter().rev().zip(iter::count(d, d)).map(|(elapsed, iters)| {
            elapsed as f64 / iters as f64
        }).collect()
    }
}
