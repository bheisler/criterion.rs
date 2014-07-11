pub use self::confidence::ConfidenceInterval;
pub use self::distribution::{Distribution,Distributions};
pub use self::estimate::{Estimate,Estimates};
pub use self::sample::Sample;

mod confidence;
mod distribution;
mod estimate;
mod resampler;
mod sample;

#[deriving(Encodable, Eq, Hash, PartialEq, Show)]
pub enum Statistic {
    Mean,
    Median,
    MedianAbsDev,
    StdDev,
}
