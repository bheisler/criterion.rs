# Command-Line Output

The output for this page was produced by running `cargo bench -- --verbose`.
`cargo bench` omits some of this information.

Every Criterion.rs benchmark calculates statistics from the measured iterations and produces a report like this:

```
Benchmarking alloc
Benchmarking alloc: Warming up for 1.0000 s
Benchmarking alloc: Collecting 100 samples in estimated 13.354 s (5050 iterations)
Benchmarking alloc: Analyzing
alloc                   time:   [2.5094 ms 2.5306 ms 2.5553 ms]
                        thrpt:  [391.34 MiB/s 395.17 MiB/s 398.51 MiB/s]
                        change: [-38.292% -37.342% -36.524%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
slope  [2.5094 ms 2.5553 ms] R^2            [0.8660614 0.8640630]
mean   [2.5142 ms 2.5557 ms] std. dev.      [62.868 us 149.50 us]
median [2.5023 ms 2.5262 ms] med. abs. dev. [40.034 us 73.259 us]
```

## Warmup

Every Criterion.rs benchmark iterates the benchmarked function automatically for a configurable warmup period (by default, for three seconds). For Rust function benchmarks, this is to warm up the processor caches and (if applicable) file system caches. For external program benchmarks, it can also be used to warm up JIT compilers.

## Collecting Samples

Criterion iterates the function to be benchmarked with a varying number of iterations to generate an estimate of the time taken by each iteration. The number of samples is configurable. It also prints an estimate of the time the sampling process will take based on the time per iteration during the warmup period.

## Time
```
time:   [2.5094 ms 2.5306 ms 2.5553 ms]
thrpt:  [391.34 MiB/s 395.17 MiB/s 398.51 MiB/s]
```

This shows a confidence interval over the measured per-iteration time for this benchmark. The left and right values show the lower and upper bounds of the confidence interval respectively, while the center value shows Criterion.rs' best estimate of the time taken for each iteration of the benchmarked routine.

The confidence level is configurable. A greater confidence level (eg. 99%) will widen the interval and thus provide the user with less information about the true slope. On the other hand, a lesser confidence interval (eg. 90%) will narrow the interval but then the user is less confident that the interval contains the true slope. 95% is generally a good balance.

Criterion.rs performs [bootstrap resampling](https://en.wikipedia.org/wiki/Bootstrapping_(statistics)) to generate these confidence intervals. The number of bootstrap samples is configurable, and defaults to 100,000.

Optionally, Criterion.rs can also report the throughput of the benchmarked code in units of bytes or elements per second.

## Change

When a Criterion.rs benchmark is run, it saves statistical information in the `target/criterion` directory. Subsequent executions of the benchmark will load this data and compare it with the current sample to show the effects of changes in the code.


```
change: [-38.292% -37.342% -36.524%] (p = 0.00 < 0.05)
Performance has improved.
```

This shows a confidence interval over the difference between this run of the benchmark and the last one, as well as the probability that the measured difference could have occurred by chance. These lines will be omitted if no saved data could be read for this benchmark.

The second line shows a quick summary. This line will indicate that the performance has improved or regressed if Criterion.rs has strong statistical evidence that this is the case. It may also indicate that the change was within the noise threshold. Criterion.rs attempts to reduce the effects of noise as much as possible, but differences in benchmark environment (eg. different load from other processes, memory usage, etc.) can influence the results. For highly-deterministic benchmarks, Criterion.rs can be sensitive enough to detect these small fluctuations, so benchmark results that overlap the range `+-noise_threshold` are assumed to be noise and considered insignificant. The noise threshold is configurable, and defaults to `+-2%`.

Additional examples:

```
alloc                   time:   [1.2421 ms 1.2540 ms 1.2667 ms]
                        change: [+40.772% +43.934% +47.801%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

```
alloc                   time:   [1.2508 ms 1.2630 ms 1.2756 ms]
                        change: [-1.8316% +0.9121% +3.4704%] (p = 0.52 > 0.05)
                        No change in performance detected.
```

```
benchmark               time:   [442.92 ps 453.66 ps 464.78 ps]
                        change: [-0.7479% +3.2888% +7.5451%] (p = 0.04 > 0.05)
                        Change within noise threshold.
```

## Detecting Outliers

```
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
```

Criterion.rs attempts to detect unusually high or low samples and reports them as outliers. A large number of outliers suggests that the benchmark results are noisy and should be viewed with appropriate skepticism. In this case, you can see that there are some samples which took much longer than normal. This might be caused by unpredictable load on the computer running the benchmarks, thread or process scheduling, or irregularities in the time taken by the code being benchmarked.

In order to ensure reliable results, benchmarks should be run on a quiet computer and should be designed to do approximately the same amount of work for each iteration. If this is not possible, consider increasing the measurement time to reduce the influence of outliers on the results at the cost of longer benchmarking period. Alternately, the warmup period can be extended (to ensure that any JIT compilers or similar are warmed up) or other iteration loops can be used to perform setup before each benchmark to prevent that from affecting the results.

## Additional Statistics

```
slope  [2.5094 ms 2.5553 ms] R^2            [0.8660614 0.8640630]
mean   [2.5142 ms 2.5557 ms] std. dev.      [62.868 us 149.50 us]
median [2.5023 ms 2.5262 ms] med. abs. dev. [40.034 us 73.259 us]
```

This shows additional confidence intervals based on other statistics.

Criterion.rs performs a linear regression to calculate the time per iteration. The first line shows the confidence interval of the slopes from the linear regressions, while the R^2 area shows the goodness-of-fit values for the lower and upper bounds of that confidence interval. If the R^2 value is low, this may indicate the benchmark isn't doing the same amount of work on each iteration. You may wish to examine the plot output and consider improving the consistency of your benchmark routine.

The second line shows confidence intervals on the mean and standard deviation of the per-iteration times (calculated naively). If std. dev. is large compared to the time values from above, the benchmarks are noisy. You may need to change your benchmark to reduce the noise.

The median/med. abs. dev. line is similar to the mean/std. dev. line, except that it uses the median and [median absolute deviation](https://en.wikipedia.org/wiki/Median_absolute_deviation). As with the std. dev., if the med. abs. dev. is large, this indicates the benchmarks are noisy.

## A Note Of Caution

Criterion.rs is designed to produce robust statistics when possible, but it can't account for everything. For example, the performance improvements and regressions listed in the above examples were created just by switching my laptop between battery power and wall power rather than changing the code under test. Care must be taken to ensure that benchmarks are performed under similar conditions in order to produce meaningful results.
