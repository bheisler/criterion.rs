use std::iter::{AdditiveIterator, mod};
use time;

use format;
use {Bencher, Criterion};

pub trait Routine {
    fn bench<I: Iterator<u64>>(&mut self, iters: I) -> Vec<u64>;
    fn warm_up(&mut self, how_long_ns: u64) -> (u64, u64);

    fn sample(&mut self, criterion: &Criterion) -> Vec<(u64, u64)> {
        let wu_ns = criterion.warm_up_time_ns;
        let m_ns = criterion.measurement_time_ns;

        println!("> Warming up for {}", format::time(wu_ns as f64));

        let (wu_elapsed, wu_iters) = self.warm_up(wu_ns);

        // Initial guess for the mean execution time
        let met = wu_elapsed as f64 / wu_iters as f64;

        let n = criterion.sample_size;
        // Solve: [d + 2*d + 3*d + ... + n*d] * met = m_ns
        let d = (2. * m_ns as f64 / met / (n * (n + 1)) as f64).ceil() as u64;

        let m_iters = iter::iterate(d, |a| a + d).take(n).collect::<Vec<u64>>();

        let m_ns = m_iters.iter().map(|&x| x).sum() as f64 * met;
        println!("> Collecting {} samples in estimated {}", n, format::time(m_ns));
        let m_elapsed = self.bench(m_iters.iter().map(|&x| x));

        m_iters.into_iter().zip(m_elapsed.into_iter()).collect()
    }
}

pub struct Function<'a>(pub |&mut Bencher|:'a);

impl<'a> Routine for Function<'a> {
    fn bench<I: Iterator<u64>>(&mut self, iters: I) -> Vec<u64> {
        let Function(ref mut f) = *self;

        let mut b = Bencher { iters: 0, ns_start: 0, ns_end: 0 };

        iters.map(|iters| {
            b.iters = iters;
            (*f)(&mut b);
            b.ns_end - b.ns_start
        }).collect()
    }

    fn warm_up(&mut self, how_long_ns: u64) -> (u64, u64) {
        let Function(ref mut f) = *self;
        let mut b = Bencher { iters: 1, ns_end: 0, ns_start: 0 };
        let ns_start = time::precise_time_ns();

        loop {
            (*f)(&mut b);

            if time::precise_time_ns() - ns_start > how_long_ns {
                return (b.ns_end - b.ns_start, b.iters);
            }

            b.iters *= 2;
        }
    }
}
