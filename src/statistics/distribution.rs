use std::collections::hashmap::{Entries,HashMap};
use test::stats::Stats;

use super::Statistic;

// The bootstrap distribution of an estimate
#[deriving(Clone)]
pub struct Distribution(Vec<f64>);

impl Distribution {
    pub fn new(data: Vec<f64>) -> Distribution {
        Distribution(data)
    }

    pub fn percentile(&self, pct: f64) -> f64 {
        use test::stats::Stats;

        self.as_slice().percentile(pct)
    }

    pub fn standard_error(&self) -> f64 {
        self.as_slice().std_dev()
    }
}

impl Vector<f64> for Distribution {
    fn as_slice(&self) -> &[f64] {
        let &Distribution(ref distribution) = self;
        distribution.as_slice()
    }
}

pub struct Distributions(HashMap<Statistic, Distribution>);

impl Distributions {
    pub fn new(statistics: &[Statistic], distributions: Vec<Distribution>) -> Distributions {
        Distributions(statistics.iter().map(|&x| x).zip(distributions.move_iter()).collect())
    }

    pub fn iter<'a>(&'a self) -> Entries<'a, Statistic, Distribution> {
        let &Distributions(ref distributions) = self;

        distributions.iter()
    }
}
