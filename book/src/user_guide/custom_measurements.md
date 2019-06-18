# Custom Measurements

By default, Criterion.rs measures the wall-clock time taken by the benchmarks. However, there are
many other ways to measure the performance of a function, such as hardware performance counters or
POSIX's CPU time. Since version 0.3.0, Criterion.rs has had support for plugging in alternate
timing measurements. This page details how to define and use these custom measurements.

Note that as of version 0.3.0, only timing measurements are supported, and only a single measurement
can be used for one benchmark. These restrictions may be lifted in future versions.

### Defining Custom Measurements

For developers who wish to use custom measurements provided by an existing crate, skip to 
["Using Custom Measurements"](#using-custom-measurements) below.

Custom measurements are defined by a pair of traits, both defined in `criterion::measurement`.

#### Measurement
First, we'll look at the main trait, `Measurement`.

```rust
pub trait Measurement {
    type Intermediate;
    type Value: MeasuredValue;

    fn start(&self) -> Self::Intermediate;
    fn end(&self, i: Self::Intermediate) -> Self::Value;

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value;
    fn zero(&self) -> Self::Value;
    fn to_f64(&self, val: &Self::Value) -> f64;

    fn formatter(&self) -> &dyn ValueFormatter;
}
```

The most important methods here are `start` and `end` and their associated types, `Intermediate`
and `Value`. `start` is called to start a measurement and `end` is called to complete it. As an
example, the `start` method of the wall-clock time measurement returns the value of the system
clock at the moment that `start` is called. This starting time is then passed to the `end` function,
which reads the system clock again and calculates the elapsed time between the two calls. This
pattern - reading some system counter before and after the benchmark and reporting the difference - 
is a common way for code to measure performance.

The next two functions, `add` and `zero` are pretty simple; Criterion.rs sometimes needs to be able
to break up a sample into batches that are added together (eg. in `Bencher::iter_batched`) and so
we need to have a way to calculate the sum of the measurements for each batch to get the overall
value for the sample. 

`to_f64` is used to convert the measured value to an `f64` value so that Criterion can perform its
analysis. As of 0.3.0, only a single value can be returned for analysis per benchmark. Since `f64`
doesn't carry any unit information, the implementor should be careful to choose their units to avoid
having extremely large or extremely small values that may have floating-point precision issues. For
wall-clock time, we convert to nanoseconds.

Finally, we have `formatter`, which just returns a trait-object reference to a `ValueFormatter` 
(more on this later).

For our half-second measurement, this is all pretty straightforward; we're still measuring
wall-clock time so we can just use `Instant` and `Duration` like `WallTime` does:

```rust
/// Silly "measurement" that is really just wall-clock time reported in half-seconds.
struct HalfSeconds;
impl Measurement for HalfSeconds {
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
        let nanos = val.as_secs() * NANOS_PER_SEC + u64::from(val.subsec_nanos());
        nanos as f64
    }
    fn formatter(&self) -> &dyn ValueFormatter {
        &HalfSecFormatter
    }
}
```

#### ValueFormatter

The next trait is `ValueFormatter`, which defines how a measurement is displayed to the user.

```rust
pub trait ValueFormatter {
    fn format_value(&self, value: f64) -> String;
    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String;
    fn scale_and_unit(&self, value: f64) -> (f64, &'static str);
}
```

All of these functions accept a value to format in f64 form; the values passed in will be in the
same scale as the values returned from `to_f64`, but may not be the exact same values. That is, if
`to_f64` returns values scaled to "thousands of cycles", the values passed to `format_value` and
the other functions will be in the same units, but may be different numbers (eg. the mean of all
sample times).

Implementors should try to format the values in a way that will make sense to humans. 
"1,500,000 ns" is needlessly confusing while "1.5 ms" is much clearer. If you can, try to use SI
prefixes to simplify the numbers. An easy way to do this is to have a series of conditionals like so:

```rust
if ns < 1.0 {  // ns = time in nanoseconds
    format!("{:>6} ps", ns * 1e3)
} else if ns < 10f64.powi(3) {
    format!("{:>6} ns", ns)
} else if ns < 10f64.powi(6) {
    format!("{:>6} us", ns / 1e3)
} else if ns < 10f64.powi(9) {
    format!("{:>6} ms", ns / 1e6)
} else {
    format!("{:>6} s", ns / 1e9)
}
```

It's also a good idea to limit the amount of precision in floating-point output - after a few
digits the numbers don't matter much anymore but add a lot of visual noise and make the results
harder to interpret. For example, it's very unlikely that anyone cares about the difference between
`10.2896653s` and `10.2896654s` - it's much more salient that their function takes "about 10
seconds per iteration".

With that out of the way, `format_value` is pretty straightforward. `format_throughput` is also not
too difficult; match on `Throughput::Bytes` or `Throughput::Elements` and generate an appropriate
description. For wall-clock time, that would likely take the form of "bytes per second", but a
measurement that read CPU performance counters might want to display throughput in terms of "cycles
per byte".

`scale_and_unit` is a bit more complex. This is primarily used for plotting. This returns two
values; an `f64` scale and a `&str` unit. The measured values will be multiplied by the scale
and the unit will be inserted into the axis labels when generating plots. So, for our wall-clock
times where the measured values are in nanoseconds, if we wanted to display plots in milliseconds
we would return `(10.0f64.powi(-6), "ms")`, because multiplying a value in nanoseconds by 10^-6
gives a value in milliseconds.

Our half-second measurement formatter thus looks like this:

```rust
struct HalfSecFormatter;
impl ValueFormatter for HalfSecFormatter {
    fn format_value(&self, value: f64) -> String {
        // The value will be in nanoseconds so we have to convert to half-seconds.
        format!("{} s/2", value * 2f64 * 10f64.powi(-9))
    }

    fn format_throughput(&self, throughput: &Throughput, value: f64) -> String {
        match *throughput {
            Throughput::Bytes(bytes) => format!(
                "{} b/s/2",
                f64::from(bytes) / (value * 2f64 * 10f64.powi(-9))
            ),
            Throughput::Elements(elems) => format!(
                "{} elem/s/2",
                f64::from(elems) / (value * 2f64 * 10f64.powi(-9))
            ),
        }
    }

    fn scale_and_unit(&self, _value: f64) -> (f64, &'static str) {
        (2f64 * 10f64.powi(-9), "s/2")
    }
}
```

### Using Custom Measurements

Once you (or an external crate) have defined a custom measurement, using it is relatively easy.
You will need to override the `Criterion` struct (which defaults to `WallTime`) by providing your
own measurement using the `with_measurement` function and overriding the default `Criterion` object
configuration. Your benchmark functions will also have to declare the measurement type they work
with.

```rust
fn fibonacci_cycles(criterion: &mut Criterion<HalfSeconds>) {
    // Use the criterion struct as normal here.
}

fn alternate_measurement() -> Criterion<HalfSeconds> {
    Criterion::default().with_measurement(HalfSeconds)
}

criterion_group! {
    name = benches;
    config = alternate_measurement();
    targets = fibonacci_cycles
}
```
