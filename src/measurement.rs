use std::time::{Duration, Instant};
use DurationExt;

/// Trait for types that represent measured values (eg. std::time::Duration is the measured value
/// for [WallTime](struct.WallTime.html)) which provides Criterion.rs the ability to convert the
/// structured value to/from f64 for analysis. It also provides functions to format the value to
/// string so that it can be displayed on the command-line and in the reports.
pub trait MeasuredValue {
    /// Constructs a MeasuredValue from an f64.
    ///
    /// Implementors can assume that the number will be in the same scale as that returned by
    /// `to_f64` (eg. if to_f64 returns nanoseconds, this value will be in nanoseconds as well).
    fn from_f64(val: f64) -> Self;

    /// Converts the measured value to f64 so that it can be used in statistical analysis.
    fn to_f64(&self) -> f64;
}

/// Trait for all types which define something Criterion.rs can measure. The only measurement
/// currently provided is [Walltime](struct.WallTime.html), but third party crates or benchmarks
/// may define more.
///
/// This trait defines two core methods, `start` and `end`. `start` is called at the beginning of
/// a measurement to produce some intermediate value (for example, the wall-clock time at the start
/// of that set of iterations) and `end` is called at the end of the measurement with the value
/// returned by `start`.
///
pub trait Measurement {
    /// This type represents an intermediate value for the measurements. It will be produced by the
    /// start function and passed to the end function. An example might be the wall-clock time as
    /// of the `start` call.
    type Intermediate;

    /// This type is the measured value. An example might be the elapsed wall-clock time between the
    /// `start` and `end` calls.
    type Value: MeasuredValue;

    /// Criterion.rs will call this before iterating the benchmark.
    fn start(&self) -> Self::Intermediate;

    /// Criterion.rs will call this after iterating the benchmark to get the measured value.
    fn end(&self, i: Self::Intermediate) -> Self::Value;

    /// Combine two values. Criterion.rs sometimes needs to perform measurements in multiple batches
    /// of iterations, so the value from one batch must be added to the sum of the previous batches.
    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value;

    /// Return a "zero" value for the Value type which can be added to another value.
    fn zero(&self) -> Self::Value;
}

impl MeasuredValue for Duration {
    fn to_f64(&self) -> f64 {
        self.to_nanos() as f64
    }

    fn from_f64(val: f64) -> Duration {
        Duration::from_nanos(val as u64)
    }
}

/// `WallTime` is the default measurement in Criterion.rs. It measures the elapsed time from the
/// beginning of a series of iterations to the end.
pub struct WallTime;
impl Measurement for WallTime {
    type Intermediate = Instant;
    type Value = Duration;

    fn start(&self) -> Self::Intermediate {
        Instant::now()
    }
    fn end(&self, i: Self::Intermediate) -> Self::Value {
        i.elapsed()
    }
    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        *v1 + *v2
    }
    fn zero(&self) -> Self::Value {
        Duration::from_secs(0)
    }
}
