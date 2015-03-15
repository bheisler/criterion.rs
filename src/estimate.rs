use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use stats::Distribution;
use rustc_serialize::json;

use ConfidenceInterval;

#[derive(Copy, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Estimate {
    pub confidence_interval: ConfidenceInterval,
    pub point_estimate: f64,
    pub standard_error: f64,
}

impl Estimate {
    pub fn new(distributions: &Distributions, points: &[f64], cl: f64) -> Estimates {
        distributions.iter().zip(points.iter()).map(|((&statistic, distribution), &point)| {
            let (lb, ub) = distribution.confidence_interval(cl);

            (statistic, Estimate {
                confidence_interval: ConfidenceInterval {
                    confidence_level: cl,
                    lower_bound: lb,
                    upper_bound: ub,
                },
                point_estimate: point,
                standard_error: distribution.std_dev(None),
            })
        }).collect()
    }

    pub fn load(path: &Path) -> Option<Estimates> {
        let mut string = String::new();

        match File::open(path) {
            Err(_) => None,
            Ok(mut f) => match f.read_to_string(&mut string) {
                Err(_) => None,
                Ok(_) => match json::decode(&string) {
                    Err(_) => None,
                    Ok(estimates) => Some(estimates),
                },
            }
        }
    }
}


#[derive(Copy, Eq, Ord, PartialEq, PartialOrd, RustcDecodable, RustcEncodable)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    Slope,
    StdDev,
}

impl fmt::Display for Statistic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
