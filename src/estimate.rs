use std::collections::BTreeMap;
use std::fmt;

use stats::Distribution;

use Estimate;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize, Debug)]
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

pub(crate) type Estimates = BTreeMap<Statistic, Estimate>;

pub(crate) type Distributions = BTreeMap<Statistic, Distribution<f64>>;
