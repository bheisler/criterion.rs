# Timing Loops

The [`Bencher`](https://bheisler.github.io/criterion.rs/criterion/struct.Bencher.html) structure
provides a number of functions which implement different timing loops for measuring the performance
of a function. This page discusses how these timing loops work and which one is appropriate for
different situations.

## `iter`

The simplest timing loop is `iter`. This loop should be the default for most benchmarks. `iter`
calls the benchmark N times in a tight loop and records the elapsed time for the entire loop.
Because it takes only two measurements (the time before and after the loop) and does nothing else in
the loop `iter` has effectively zero measurement overhead - meaning it can accurately measure the
performance of functions as small as a single processor instruction.

However, `iter` has limitations as well. If the benchmark returns a value which implements Drop, it
will be dropped inside the loop and the drop function's time will be included in the measurement.
Additionally, some benchmarks need per-iteration setup. A benchmark for a sorting algorithm
might require some unsorted data to operate on, but we don't want the generation of the unsorted
data to affect the measurement. `iter` provides no way to do this.

## `iter_with_large_drop`

`iter_with_large_drop` is an answer to the first problem. In this case, the values returned by the
benchmark are collected into a `Vec` to be dropped after the measurement is complete. This
introduces a small amount of measurement overhead, meaning that the measured value will be slightly
higher than the true runtime of the function. This overhead is almost always negligible, but it's
important to be aware that it exists. Extremely fast benchmarks (such as those in the
hundreds-of-picoseconds range or smaller) or benchmarks that return very large structures may incur
more overhead.

Aside from the measurement overhead, `iter_with_large_drop` has its own limitations. Collecting the
returned values into a `Vec` uses heap memory, and the amount of memory used is not under the
control of the user. Rather, it depends on the iteration count which in turn depends on the
benchmark settings and the runtime of the benchmarked function. It is possible that a benchmark
could run out of memory while collecting the values to drop.

## `iter_batched/iter_batched_ref`

`iter_batched` and `iter_batched_ref` are the next step up in complexity for timing loops. These
timing loops take two closures rather than one. The first closure takes no arguments and returns
a value of type `T` - this is used to generate setup data. For example, the setup function might
clone a vector of unsorted data for use in benchmarking a sorting function. The second closure
is the function to benchmark, and it takes a `T` (for `iter_batched`) or `&mut T` (for 
`iter_batched_ref`).

These two timing loops generate a batch of inputs and measure the time to execute the benchmark on
all values in the batch. As with `iter_with_large_drop` they also collect the values returned from
the benchmark into a `Vec` and drop it later without timing the drop. Then another batch of inputs
is generated and the process is repeated until enough iterations of the benchmark have been measured.
Keep in mind that this is only necessary if the benchmark modifies the input - if the input is 
constant then one input value can be reused and the benchmark should use `iter` instead.

Both timing loops accept a third parameter which controls how large a batch is. If the batch size
is too large, we might run out of memory generating the inputs and collecting the outputs. If it's
too small, we could introduce more measurement overhead than is necessary. For ease of use, Criterion
provides three pre-defined choices of batch size, defined by the 
[`BatchSize`](https://bheisler.github.io/criterion.rs/criterion/enum.BatchSize.html) enum - 
`SmallInput`, `LargeInput` and `PerIteration`. It is also possible (though not recommended) to set
the batch size manually.

`SmallInput` should be the default for most benchmarks. It is tuned for benchmarks where the setup
values are small (small enough that millions of values can safely be held in memory) and the output
is likewise small or nonexistent. `SmallInput` incurs the least measurement overhead (equivalent to
that of `iter_with_large_drop` and therefore negligible for nearly all benchmarks), but also uses
the most memory.

`LargeInput` should be used if the input or output of the benchmark is large enough that `SmallInput`
uses too much memory. `LargeInput` incurs slightly more measurement overhead than `SmallInput`, but
the overhead is still small enough to be negligible for almost all benchmarks.

`PerIteration` forces the batch size to one. That is, it generates a single setup input, times the
execution of the function once, discards the setup and output, then repeats. This results in a
great deal of measurement overhead - several orders of magnitude more than the other options. It
can be enough to affect benchmarks into the hundreds-of-nanoseconds range. Using `PerIteration`
should be avoided wherever possible. However, it is sometimes necessary if the input or output of
the benchmark is extremely large or holds a limited resource like a file handle.

Although sticking to the pre-defined settings is strongly recommended, Criterion.rs does allow
users to choose their own batch size if necessary. This can be done with `BatchSize::NumBatches` or
`BatchSize::NumIterations`, which specify the number of batches per sample or the number of
iterations per batch respectively. These options should be used only when necessary, as they require
the user to tune the settings manually to get accurate results. However, they are provided as an
option in case the pre-defined options are all unsuitable. `NumBatches` should be preferred over
`NumIterations` as it will typically have less measurement overhead, but `NumIterations` provides
more control over the batch size which may be necessary in some situations.

## `iter_custom`

This is a special "timing loop" that relies on you to do your own timing. Where the other timing
loops take a lambda to call N times in a loop, this takes a lambda of the form 
`FnMut(iters: u64) -> M::Value` - meaning that it accepts the number of iterations and returns
the measured value. Typically, this will be a `Duration` for the default `WallTime` measurement,
but it may be other types for other measurements (see the
[Custom Measurements](./custom_measurements.md) page for more details). The lambda
can do whatever is needed to measure the value.

Use `iter_custom` when you need to do something that doesn't fit into the usual approach of calling
a function in a loop. For example, this might be used for:

* Benchmarking external processes by sending the iteration count and receiving the elapsed time
* Measuring how long a thread pool takes to execute N jobs, to see how lock contention or pool-size
  affects the wall-clock time

Try to keep the overhead in the measurement routine to a minimum; Criterion.rs will still use its
normal warm-up/target-time logic, which is based on wall-clock time. If your measurement routine
takes a long time to perform each measurement it could mess up the calculations and cause
Criterion.rs to run too few iterations (not to mention that the benchmarks would take a long time).
Because of this, it's best to do heavy setup like starting processes or threads before running the
benchmark.

## What do I do if my function's runtime is smaller than the measurement overhead?

Criterion.rs' timing loops are carefully designed to minimize the measurement overhead as much as
possible. For most benchmarks the measurement overhead can safely be ignored because the true
runtime of most benchmarks will be very large relative to the overhead. However, benchmarks with a
runtime that is not much larger than the overhead can be difficult to measure.

If you believe that your benchmark is small compared to the measurement overhead, the first option
is to adjust the timing loop to reduce the overhead. Using `iter` or `iter_batched` with `SmallInput`
should be the first choice, as these options incur a minimum of measurement overhead. In general,
using `iter_batched` with larger batches produces less overhead, so replacing `PerIteration` with
`NumIterations` with a suitable batch size will typically reduce the overhead. It is possible for
the batch size to be too large, however, which will increase (rather than decrease) overhead.

If this is not sufficient, the only recourse is to benchmark a larger function. It's tempting to do
this by manually executing the routine a fixed number of times inside the benchmark, but this is
equivalent to what `NumIterations` already does. The only difference is that Criterion.rs can
account for `NumIterations` and show the correct runtime for one iteration of the function rather
than many. Instead, consider benchmarking at a higher level.

It's important to stress that measurement overhead only matters for very fast functions which
modify their input. For slower functions (roughly speaking, anything at the nanosecond level or
larger, or the microsecond level for `PerIteration`, assuming a reasonably modern x86_64 processor
and OS or equivalent) are not meaningfully affected by measurement overhead. For functions which
only read their input and do not modify or consume it, one value can be shared by all iterations
using the `iter` loop which has effectively no overhead.

## Deprecated Timing Loops

In older Criterion.rs benchmarks (pre 2.10), one might see two more timing loops, called
`iter_with_setup` and `iter_with_large_setup`. `iter_with_setup` is equivalent to `iter_batched`
with `PerIteration`. `iter_with_large_setup` is equivalent to `iter_batched` with `NumBatches(1)`.
Both produce much more measurement overhead than `SmallInput`. Additionally. `large_setup` also
uses much more memory. Both should be updated to use `iter_batched`, preferably with `SmallInput`.
They are kept for backwards-compatibility reasons, but no longer appear in the API documentation.
