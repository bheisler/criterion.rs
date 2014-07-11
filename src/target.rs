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

// FIXME Unboxed closures: Get rid of the `Option`
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

    fn run(&mut self, iterations: u64) -> Ns<u64> {
        match *self {
            Function(ref mut fun) => {
                let mut b = bencher::new(iterations);

                // FIXME Unboxed closures: Get rid of this `Option` dance
                let f = fun.take_unwrap();
                f(&mut b);
                *fun = Some(f);

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

    // FIXME Return type should be `Sample`
    pub fn bench(&mut self,
                 sample_size: uint,
                 iterations: u64)
                 -> Ns<Sample<Vec<f64>>> {
        Sample::new(range(0, sample_size).map(|_| {
            self.run(iterations).unwrap() as f64 / iterations as f64
        }).collect()).ns()
    }
}
