use bencher::Bencher;
use bencher;
use statistics::Sample;
use stream::Stream;
use time::Time;
use time::prefix::Nano;
use time::traits::{Nanosecond,Prefix};
use time::types::Ns;
use time::unit;
use time;

pub enum Target<'a, I> {
    Function(fn (&mut Bencher)),
    FunctionFamily(fn (&mut Bencher, &I), &'a I),
    Program(Stream),
}

impl<'a, I> Target<'a, I> {
    pub fn warm_up<P: Prefix>(
                   &mut self,
                   how_long: Time<P, unit::Second, u64>)
                   -> (Ns<u64>, u64) {
        let how_long = how_long.to::<Nano>();
        let mut iterations = 1;

        let init = time::now();
        loop {
            let elapsed = self.run(iterations);

            if time::now() - init > how_long {
                return (elapsed, iterations);
            }

            iterations *= 2;
        }
    }

    pub fn run(&mut self, iterations: u64) -> Ns<u64> {
        match *self {
            Function(fun) => {
                let mut b = bencher::new(iterations);

                fun(&mut b);

                bencher::elapsed(&b)
            },
            FunctionFamily(fun, input) => {
                let mut b = bencher::new(iterations);

                fun(&mut b, input);

                bencher::elapsed(&b)
            },
            Program(ref mut prog) => {
                prog.send(iterations);

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
        iterations: u64
    ) -> Ns<Sample<Vec<f64>>> {
        let mut sample = Vec::from_elem(sample_size, 0);

        match *self {
            Function(fun) => {
                let mut b = bencher::new(iterations);

                for measurement in sample.mut_iter() {
                    fun(&mut b);
                    *measurement = bencher::elapsed(&b).unwrap();
                }
            },
            FunctionFamily(fun, input) => {
                let mut b = bencher::new(iterations);

                for measurement in sample.mut_iter() {
                    fun(&mut b, input);
                    *measurement = bencher::elapsed(&b).unwrap();
                }
            },
            Program(ref mut prog) => {
                for _ in range(0, sample_size) {
                    prog.send(iterations);
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
            elapsed as f64 / iterations as f64
        }).collect()).ns()
    }
}
