use clock::Clock;
use test::black_box;
use time::precise_time_ns;

pub struct Bencher {
    clock: Option<Clock>,
    iterations: u64,
    ns_end: u64,
    ns_start: u64,
}

impl Bencher {
    pub fn bench_n(&mut self, n: u64, f: |&mut Bencher|) {
        self.iterations = n;
        f(self);
    }

    pub fn iter<T>(&mut self, inner: || -> T) {
        self.ns_start = precise_time_ns();
        for _ in range(0, self.iterations) {
            black_box(inner());
        }
        self.ns_end = precise_time_ns();
    }

    pub fn new(clock: Option<Clock>) -> Bencher {
        Bencher {
            clock: clock,
            iterations: 0,
            ns_end: 0,
            ns_start: 0,
        }
    }

    pub fn ns_elapsed(&self) -> u64 {
        self.ns_end - self.ns_start
    }

    pub fn ns_per_iter(&self) -> f64 {
        let iters = self.iterations;
        let elapsed = self.ns_elapsed() as f64;

        match self.clock {
            None => {
                elapsed / (iters + 1) as f64
            },
            // XXX this operation introduces variance in the measurement
            // I'll assume the variance introduced is negligible
            Some(clock) => {
                (elapsed - clock.cost()) / iters as f64
            },
        }
    }
}
