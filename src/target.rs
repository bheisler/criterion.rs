use test;
use time;

use statistics::Sample;
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
    pub fn warm_up(&mut self, how_long: u64) -> (u64, u64) {
        let mut iters = 1;

        let init = time::precise_time_ns();
        loop {
            let elapsed = self.run(iters);

            if time::precise_time_ns() - init > how_long {
                return (elapsed, iters);
            }

            iters *= 2;
        }
    }

    pub fn run(&mut self, iters: u64) -> u64 {
        match *self {
            Function(ref mut fun_opt) => {
                let mut b = Bencher { iters: iters, ns_end: 0, ns_start: 0 };

                let fun = fun_opt.take_unwrap();
                fun(&mut b);
                *fun_opt = Some(fun);

                b.ns_end - b.ns_start
            },
            Program(ref mut prog) => {
                prog.send(iters);

                let msg = prog.recv();
                let msg = msg.as_slice().trim();

                let elapsed: u64 = match from_str(msg) {
                    None => fail!("Couldn't parse program output: {}", msg),
                    Some(elapsed) => elapsed,
                };

                elapsed
            },
        }
    }

    pub fn bench(
        &mut self,
        sample_size: uint,
        iters: u64
    ) -> Sample<Vec<f64>> {
        let mut sample = Vec::from_elem(sample_size, 0);

        match *self {
            Function(ref mut fun_opt) => {
                let mut b = Bencher { iters: iters, ns_start: 0, ns_end: 0 };
                let fun = fun_opt.take_unwrap();

                for measurement in sample.mut_iter() {
                    fun(&mut b);
                    *measurement = b.ns_end - b.ns_start;
                }

                *fun_opt = Some(fun);
            },
            Program(ref mut prog) => {
                for _ in range(0, sample_size) {
                    prog.send(iters);
                }

                for measurement in sample.mut_iter() {
                    let msg = prog.recv();
                    let msg = msg.as_slice().trim();

                    *measurement = match from_str(msg) {
                        None => fail!("Couldn't parse program output"),
                        Some(elapsed) => elapsed,
                    };
                }
            },
        }

        Sample::new(sample.move_iter().map(|elapsed| {
            elapsed as f64 / iters as f64
        }).collect())
    }
}
