use serialize::json;
use stats::{ConfidenceInterval, Distribution};
use std::collections::TreeMap;
use std::io::File;
use std::fmt::{Formatter, Show, mod};

use stats;

#[deriving(Decodable, Encodable, PartialEq)]
pub struct Estimate {
    pub confidence_interval: ConfidenceInterval<f64>,
    pub point_estimate: f64,
    pub standard_error: f64,
}

impl Estimate {
    pub fn new(distributions: &Distributions, points: &[f64], cl: f64) -> Estimates {
        distributions.iter().zip(points.iter()).map(|((&statistic, distribution), &point)| {
            (statistic, Estimate {
                confidence_interval: distribution.confidence_interval(cl),
                point_estimate: point,
                standard_error: distribution.standard_error(),
            })
        }).collect()
    }

    pub fn load(path: &Path) -> Option<Estimates> {
        match File::open(path).read_to_string() {
            Err(_) => None,
            Ok(string) => match json::decode(string.as_slice()) {
                Err(_) => None,
                Ok(estimates) => Some(estimates),
            },
        }
    }
}

#[deriving(Decodable, Eq, Encodable, Ord, PartialEq, PartialOrd)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    Slope,
    StdDev,
}

impl Statistic {
    pub fn abs_fn(&self) -> fn(&[f64]) -> f64 {
        match *self {
            Mean => stats::mean,
            Median => stats::median,
            MedianAbsDev => stats::median_abs_dev,
            StdDev => stats::std_dev,
            _ => unimplemented!(),
        }
    }

    pub fn rel_fn(&self) -> fn(&[f64], &[f64]) -> f64 {
        fn rel_diff_mean(x: &[f64], y: &[f64]) -> f64 {
            stats::mean(x) / stats::mean(y) - 1.
        }

        fn rel_diff_median(x: &[f64], y: &[f64]) -> f64 {
            stats::median(x) / stats::median(y) - 1.
        }

        match *self {
            Mean => rel_diff_mean,
            Median => rel_diff_median,
            _ => unimplemented!(),
        }
    }
}

impl Show for Statistic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Mean => f.pad("mean"),
            Median => f.pad("median"),
            MedianAbsDev => f.pad("MAD"),
            Slope => f.pad("slope"),
            StdDev => f.pad("SD"),
        }
    }
}

pub type Estimates = TreeMap<Statistic, Estimate>;

pub type Distributions = TreeMap<Statistic, Distribution<f64>>;
