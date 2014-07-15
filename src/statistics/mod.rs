use std::fmt;
use std::fmt::{Formatter,Show};

pub use self::confidence::ConfidenceInterval;
pub use self::distribution::{Distribution,Distributions};
pub use self::estimate::{Estimate,Estimates};
pub use self::sample::Sample;

mod confidence;
mod distribution;
mod estimate;
mod resampler;
mod sample;

#[deriving(Decodable,Encodable, Eq, Hash, PartialEq)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    StdDev,
}

impl Show for Statistic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Mean => f.pad("mean"),
            Median => f.pad("median"),
            MedianAbsDev => f.pad("MAD"),
            StdDev => f.pad("SD"),
        }
    }
}
