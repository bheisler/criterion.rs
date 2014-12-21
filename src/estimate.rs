use serialize::json;
use stats::{ConfidenceInterval, Distribution};
use std::collections::BTreeMap;
use std::io::File;
use std::fmt::{Formatter, Show, mod};

#[deriving(Copy, Decodable, Encodable, PartialEq)]
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

#[deriving(Copy, Decodable, Eq, Encodable, Ord, PartialEq, PartialOrd)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    Slope,
    StdDev,
}

impl Show for Statistic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Statistic::Mean => f.pad("mean"),
            Statistic::Median => f.pad("median"),
            Statistic::MedianAbsDev => f.pad("MAD"),
            Statistic::Slope => f.pad("slope"),
            Statistic::StdDev => f.pad("SD"),
        }
    }
}

pub type Estimates = BTreeMap<Statistic, Estimate>;

pub type Distributions = BTreeMap<Statistic, Distribution<f64>>;
