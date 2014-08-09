use test;

use statistics::Sample;
use stream::Stream;
use time::Time;
use time::prefix::Nano;
use time::traits::{Nanosecond, Prefix};
use time::types::Ns;
use time::unit;
use time;

/// Helper `struct` to build benchmark functions that follow the setup - bench - teardown pattern.
#[experimental]
pub struct Bencher {
    end: Ns<u64>,
    iters: u64,
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
        for _ in range(0, self.iters) {
            test::black_box(routine());
        }
        self.end = time::now();
    }
}

pub enum Target<'a> {
    Function(Option<|&mut Bencher|:'a>),
    Program(Stream),
}

impl<'a> Target<'a> {
    pub fn warm_up<P: Prefix>(
                   &mut self,
                   how_long: Time<P, unit::Second, u64>)
                   -> (Ns<u64>, u64) {
        let how_long = how_long.to::<Nano>();
        let mut iters = 1;

        let init = time::now();
        loop {
            let elapsed = self.run(iters);

            if time::now() - init > how_long {
                return (elapsed, iters);
            }

            iters *= 2;
        }
    }

    pub fn run(&mut self, iters: u64) -> Ns<u64> {
        match *self {
            Function(ref mut fun_opt) => {
                let mut b = Bencher { iters: iters, end: 0.ns(), start: 0.ns() };

                let fun = fun_opt.take_unwrap();
                fun(&mut b);
                *fun_opt = Some(fun);

                b.end - b.start
            },
            Program(ref mut prog) => {
                prog.send(iters);

                let msg = prog.recv();
                let msg = msg.as_slice().trim();

                let elapsed: u64 = match from_str(msg) {
                    None => fail!("Couldn't parse program output: {}", msg),
                    Some(elapsed) => elapsed,
                };

                elapsed.ns()
            },
        }
    }

    pub fn bench(
        &mut self,
        sample_size: uint,
        iters: u64
    ) -> Ns<Sample<Vec<f64>>> {
        let mut sample = Vec::from_elem(sample_size, 0);

        match *self {
            Function(ref mut fun_opt) => {
                let mut b = Bencher { iters: iters, start: 0.ns(), end: 0.ns() };
                let fun = fun_opt.take_unwrap();

                for measurement in sample.mut_iter() {
                    fun(&mut b);
                    *measurement = (b.end - b.start).unwrap();
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
        }).collect()).ns()
    }
}
