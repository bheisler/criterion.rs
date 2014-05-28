use bootstrap::Bootstrap;
use clock::Clock;
use outlier::Outliers;
use serialize::json;
use std::io::{File,Truncate,Write};
use test::black_box;
use test::stats::Stats;
use time::precise_time_ns;

#[deriving(Encodable)]
pub struct Sample {
    data: Vec<f64>,
    iters: uint,
}

impl Sample {
    pub fn new<'a, T>(size: uint,
                      action: ||:'a -> T,
                      iters: uint,
                      clock: Option<Clock>)
        -> (Sample, ||:'a -> T)
    {
        let mut total_time = Vec::from_elem(size, 0u64);

        for t in total_time.mut_iter() {
            let start = precise_time_ns();
            for _ in range(0, iters) {
                black_box(action());
            }
            *t = precise_time_ns() - start;
        }

        let time_per_iter: Vec<f64> = total_time.move_iter().map(|t| {
            match clock {
                None => {
                    t as f64 / (iters + 1) as f64
                },
                // XXX this operation adds variance to our measurement, but
                // we'll consider the increment to be negligible
                Some(clock) => {
                    (t as f64 - clock.cost()) / iters as f64
                },
            }
        }).collect();

        let sample = Sample {
            data: time_per_iter,
            iters: iters,
        };

        (sample, action)
    }

    pub fn bootstrap(&self, nresamples: uint, cl: f64) -> Bootstrap {
        Bootstrap::new(self, nresamples, cl)
    }

    pub fn data<'a>(&'a self) -> &'a [f64] {
        self.data.as_slice()
    }

    pub fn dump(&self, path: &Path) {
        let json = json::Encoder::str_encode(self);

        match File::open_mode(&Path::new(path), Truncate, Write) {
            Err(_) => fail!("couldn't open {}", path.display()),
            Ok(mut file) => match file.write_str(json.as_slice()) {
                Err(_) => fail!("couldn't write {}", path.display()),
                Ok(_) => {},
            }
        }
    }

    pub fn iters(&self) -> uint {
        self.iters
    }

    pub fn mean(&self) -> f64 {
        self.data.as_slice().mean()
    }

    pub fn median(&self) -> f64 {
        self.data.as_slice().median()
    }

    pub fn median_abs_dev(&self) -> f64 {
        self.data.as_slice().median_abs_dev()
    }

    pub fn outliers(&self) -> Outliers {
        Outliers::new(self)
    }

    pub fn std_dev(&self) -> f64 {
        self.data.as_slice().std_dev()
    }

    pub fn quartiles(&self) -> (f64, f64, f64) {
        self.data.as_slice().quartiles()
    }

    // remove severe outliers using the IQR criteria
    pub fn without_outliers(&self) -> Sample {
        let (q1, _, q3) = self.quartiles();
        let iqr = q3 - q1;
        let (lb, ub) = (q1 - 3.0 * iqr, q3 + 3.0 * iqr);

        let data: Vec<f64> = self.data.iter().filter_map(|&x| {
            if x > lb && x < ub {
                Some(x)
            } else {
                None
            }
        }).collect();

        Sample {
            data: data,
            iters: self.iters,
        }
    }
}

impl Container for Sample {
    fn len(&self) -> uint {
        self.data.len()
    }
}
