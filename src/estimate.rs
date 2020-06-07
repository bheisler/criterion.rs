use std::collections::BTreeMap;
use std::fmt;

use crate::stats::Distribution;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize, Debug)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    Slope,
    StdDev,
}

impl fmt::Display for Statistic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Statistic::Mean => f.pad("mean"),
            Statistic::Median => f.pad("median"),
            Statistic::MedianAbsDev => f.pad("MAD"),
            Statistic::Slope => f.pad("slope"),
            Statistic::StdDev => f.pad("SD"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
pub struct ConfidenceInterval {
    pub confidence_level: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
pub struct Estimate {
    /// The confidence interval for this estimate
    pub confidence_interval: ConfidenceInterval,
    ///
    pub point_estimate: f64,
    /// The standard error of this estimate
    pub standard_error: f64,
}

pub fn build_estimates(
    distributions: &Distributions,
    points: &BTreeMap<Statistic, f64>,
    cl: f64,
) -> Estimates {
    distributions
        .iter()
        .map(|(&statistic, distribution)| {
            let point_estimate = points[&statistic];
            let (lb, ub) = distribution.confidence_interval(cl);

            (
                statistic,
                Estimate {
                    confidence_interval: ConfidenceInterval {
                        confidence_level: cl,
                        lower_bound: lb,
                        upper_bound: ub,
                    },
                    point_estimate,
                    standard_error: distribution.std_dev(None),
                },
            )
        })
        .collect()
}

pub type Estimates = BTreeMap<Statistic, Estimate>;

pub type Distributions = BTreeMap<Statistic, Distribution<f64>>;
