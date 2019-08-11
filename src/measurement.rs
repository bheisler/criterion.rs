//! This module defines a set of traits that can be used to plug different measurements (eg.
//! Unix's Processor Time, CPU or GPU performance counters, etc.) into Criterion.rs. It also
//! includes the [WallTime](struct.WallTime.html) struct which defines the default wall-clock time
//! measurement.

use crate::format::short;
use crate::DurationExt;
use crate::Throughput;
use std::time::{Duration, Instant};

/// Trait providing functions to format measured values to string so that they can be displayed on
/// the command line or in the reports. The functions of this trait take measured values in f64
/// form; implementors can assume that the values are of the same scale as those produced by the
/// associated [MeasuredValue](trait.MeasuredValue.html) (eg. if your value produces values in
/// nanoseconds, the values passed to the formatter will be in nanoseconds).
///
/// Implementors are encouraged to format the values in a way that is intuitive for humans and
/// uses the SI prefix system. For example, the format used by [WallTime](struct.Walltime.html)
/// can display the value in units ranging from picoseconds to seconds depending on the magnitude
/// of the elapsed time in nanoseconds.
pub trait ValueFormatter {
    /// Format the value (with appropriate unit) and return it as a string.
    fn format_value(&self, value: f64) -> String;

    /// Format the value as a throughput measurement. The value represents the measurement value;
    /// the implementor will have to calculate bytes per second, iterations per cycle, etc.
    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String;

    /// Return a scale factor and a unit string appropriate for the given value.
    ///
    /// Criterion.rs will multiple the scale factor by the value to produce a value in the returned
    /// unit. For example, if value is in nanoseconds (1 second = 10^9 nanoseconds) but we wish to
    /// display it in seconds, `scale_and_unit` should return `(10.powi(-9), "s")`, because
    /// `value * 10.powi(-9)` will produce a number of seconds.
    fn scale_and_unit(&self, value: f64) -> (f64, &'static str);

    // TODO: I really don't like this scale_and_unit function. There must be a better interface
    // for this...

    /// Scale the values and return a unit string designed for machines.
    ///
    /// For example, this is used for the CSV file output. Implementations should modify the given
    /// values slice to apply the desired scaling (if any) and return a string representing the unit
    /// the modified values are in.
    fn scale_for_machines(&self, values: &mut [f64]) -> &'static str;
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
    type Value;

    /// Criterion.rs will call this before iterating the benchmark.
    fn start(&self) -> Self::Intermediate;

    /// Criterion.rs will call this after iterating the benchmark to get the measured value.
    fn end(&self, i: Self::Intermediate) -> Self::Value;

    /// Combine two values. Criterion.rs sometimes needs to perform measurements in multiple batches
    /// of iterations, so the value from one batch must be added to the sum of the previous batches.
    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value;

    /// Return a "zero" value for the Value type which can be added to another value.
    fn zero(&self) -> Self::Value;

    /// Converts the measured value to f64 so that it can be used in statistical analysis.
    fn to_f64(&self, value: &Self::Value) -> f64;

    /// Return a trait-object reference to the value formatter for this measurement.
    fn formatter(&self) -> &dyn ValueFormatter;
}

pub(crate) struct DurationFormatter;
impl DurationFormatter {
    fn bytes_per_second(&self, bytes_per_second: f64) -> String {
        if bytes_per_second < 1024.0 {
            format!("{:>6}   B/s", short(bytes_per_second))
        } else if bytes_per_second < 1024.0 * 1024.0 {
            format!("{:>6} KiB/s", short(bytes_per_second / 1024.0))
        } else if bytes_per_second < 1024.0 * 1024.0 * 1024.0 {
            format!("{:>6} MiB/s", short(bytes_per_second / (1024.0 * 1024.0)))
        } else {
            format!(
                "{:>6} GiB/s",
                short(bytes_per_second / (1024.0 * 1024.0 * 1024.0))
            )
        }
    }

    fn elements_per_second(&self, elements_per_second: f64) -> String {
        if elements_per_second < 1000.0 {
            format!("{:>6}  elem/s", short(elements_per_second))
        } else if elements_per_second < 1000.0 * 1000.0 {
            format!("{:>6} Kelem/s", short(elements_per_second / 1000.0))
        } else if elements_per_second < 1000.0 * 1000.0 * 1000.0 {
            format!(
                "{:>6} Melem/s",
                short(elements_per_second / (1000.0 * 1000.0))
            )
        } else {
            format!(
                "{:>6} Gelem/s",
                short(elements_per_second / (1000.0 * 1000.0 * 1000.0))
            )
        }
    }
}
impl ValueFormatter for DurationFormatter {
    fn format_value(&self, ns: f64) -> String {
        crate::format::time(ns)
    }

    fn format_throughput(&self, throughput: &Throughput, ns: f64) -> String {
        match *throughput {
            Throughput::Bytes(bytes) => self.bytes_per_second(f64::from(bytes) * (1e9 / ns)),
            Throughput::Elements(elems) => self.elements_per_second(f64::from(elems) * (1e9 / ns)),
        }
    }

    fn scale_and_unit(&self, ns: f64) -> (f64, &'static str) {
        if ns < 10f64.powi(0) {
            (10f64.powi(3), "ps")
        } else if ns < 10f64.powi(3) {
            (10f64.powi(0), "ns")
        } else if ns < 10f64.powi(6) {
            (10f64.powi(-3), "us")
        } else if ns < 10f64.powi(9) {
            (10f64.powi(-6), "ms")
        } else {
            (10f64.powi(-9), "s")
        }
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        // no scaling is needed
        "ns"
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
    fn to_f64(&self, val: &Self::Value) -> f64 {
        val.to_nanos() as f64
    }
    fn formatter(&self) -> &dyn ValueFormatter {
        &DurationFormatter
    }
}
