### Comparison with Criterion-rs

I intend Iai to be a complement to Criterion-rs, not a competitor. The two projects measure different
things in different ways and have different pros, cons, and limitations, so for most projects the
best approach is to use both.

Here's an overview of the important differences:
- **Temporary Con:** Right now, Iai is lacking many features of Criterion-rs, including reports and configuration of any kind.
    - The current intent is to add support to [Cargo-criterion] for configuring and reporting on Iai benchmarks.
- **Pro:** Iai can reliably detect much smaller changes in performance than Criterion-rs can.
- **Pro:** Iai can work reliably in noisy CI environments or even cloud CI providers like GitHub Actions or Travis-CI, where Criterion-rs cannot.
- **Pro:** Iai also generates profile output from the benchmark without further effort.
- **Pro:** Although Cachegrind adds considerable runtime overhead, running each benchmark exactly once is still usually faster than Criterion-rs' statistical measurements.
- **Mixed:** Because Iai can detect such small changes, it may report performance differences from changes to the order of functions in memory and other compiler details.
- **Con:** Iai's measurements merely correlate with wall-clock time (which is usually what you actually care about), where Criterion-rs measures it directly.
- **Con:** Iai cannot exclude setup code from the measurements, where Criterion-rs can.
- **Con:** Because Cachegrind does not measure system calls, IO time is not accurately measured.
- **Con:** Because Iai runs the benchmark exactly once, it cannot measure variation in the performance such as might be caused by OS thread scheduling or hash-table randomization.
- **Limitation:** Iai can only be used on platforms supported by Valgrind. Notably, this does not include Windows.

For benchmarks that run in CI (especially if you're checking for performance regressions in pull 
requests on cloud CI) you should use Iai. For benchmarking on Windows or other platforms that
Valgrind doesn't support, you should use Criterion-rs. For other cases, I would advise using both.
Iai gives more precision and scales better to larger benchmarks, while Criterion-rs allows for
excluding setup time and gives you more information about the actual time your code takes and how
strongly that is affected by non-determinism like threading or hash-table randomization. If you
absolutely need to pick one or the other though, Iai is probably the one to go with.
