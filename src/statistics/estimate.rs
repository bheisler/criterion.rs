use serialize::{Encodable,json};
use std::collections::HashMap;
use std::io::File;

use super::Statistic;
use super::confidence::ConfidenceInterval;

#[deriving(Decodable,Encodable)]
pub struct Estimate {
    confidence_interval: ConfidenceInterval,
    point_estimate: f64,
    standard_error: f64,
}

impl Estimate {
    pub fn new(ci: ConfidenceInterval, p: f64, se: f64) -> Estimate {
        Estimate {
            confidence_interval: ci,
            point_estimate: p,
            standard_error: se,
        }
    }

    pub fn confidence_interval(&self) -> ConfidenceInterval {
        self.confidence_interval
    }

    pub fn point_estimate(&self) -> f64 {
        self.point_estimate
    }

    pub fn standard_error(&self) -> f64 {
        self.standard_error
    }
}

pub struct Estimates(HashMap<Statistic, Estimate>);

impl Estimates {
    pub fn new(statistics: &[Statistic], estimates: Vec<Estimate>) -> Estimates {
        Estimates(statistics.iter().map(|&x| x).zip(estimates.move_iter()).collect())
    }

    pub fn load(path: &Path) -> Option<Estimates> {
        match File::open(path).read_to_string() {
            Err(_) => None,
            Ok(string) => match json::decode(string.as_slice()) {
                Err(_) => None,
                Ok(estimates) => Some(Estimates(estimates)),
            }
        }
    }

    pub fn get<'a>(&'a self, statistic: Statistic) -> &'a Estimate {
        let &Estimates(ref estimates) = self;
        estimates.get(&statistic)
    }

    pub fn save(&self, path: &Path) {
        let &Estimates(ref estimates) = self;

        match File::create(path).write_str(json::encode(estimates).as_slice()) {
            Err(e) => fail!("{}", e),
            Ok(_) => {},
        }
    }
}
