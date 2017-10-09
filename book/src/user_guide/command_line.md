# Command-Line Output

Every Criterion-rs benchmark calculates statistics from the measured iterations and produces a report like this:

```
Benchmarking alloc
> Warming up for 1.0000 s
> Collecting 100 samples in estimated 31.601 s
> Found 11 outliers among 99 measurements (11.11%)
  > 4 (4.04%) high mild
  > 7 (7.07%) high severe
> Performing linear regression
  >  slope [5.9892 ms 6.0237 ms]
  >    R^2  0.9821950 0.9812767
> Estimating the statistics of the sample
  >   mean [5.9994 ms 6.1268 ms]
  > median [5.9781 ms 5.9935 ms]
  >    MAD [16.247 us 32.562 us]
  >     SD [61.936 us 538.86 us]
```

## Warmup

Every Criterion-rs benchmark iterates the benchmarked function automatically for a configurable warmup period (by default, for one second). For Rust function benchmarks, this is to warm up the processor caches and (if applicable) file system caches. For external program benchmarks, it can also be used to warm up JIT compilers.

## Collecting Samples

Criterion iterates the function to be benchmarked with a varying number of iterations to generate an estimate of the time taken by each iteration. The number of samples is configurable. It also prints an estimate of the time the sampling process will take based on the time per iteration during the warmup period.

## Detecting Outliers

Criterion-rs attempts to detect unusually high or low samples and reports them as outliers. A large number of outliers suggests that the benchmark results are noisy and should be viewed with appropriate skepticism. In this case, you can see that there are some samples which took much longer than normal. This might be caused by unpredictable load on the computer running the benchmarks, thread or process scheduling, or irregularities in the time taken by the code being benchmarked.

In order to ensure reliable results, benchmarks should be run on a quiet computer and should be designed to do approximately the same amount of work for each iteration. If this is not possible, consider increasing the sample size and/or measurement time to reduce the influence of outliers on the results at the cost of longer benchmarking time. Alternately, the warmup period can be extended (to ensure that any JIT compilers or similar are warmed up) or other iteration loops can be used to perform setup before each benchmark to prevent that from affecting the results.

## Linear Regression 

```
> Performing linear regression
  >  slope [5.9892 ms 6.0237 ms]
  >    R^2  0.9821950 0.9812767
```

Here, Criterion-rs attempts to calculate the time taken per iteration of the benchmark.

The slope represents Criterion-rs' best guess at the time taken for each iteration of the benchmark. More precisely, this shows a 95% confidence interval on the time per iteration. The confidence level is configurable. A greater confidence level (eg. 99%) will widen the interval and thus provide the user with less information about the true slope. On the other hand, a lesser confidence interval (eg. 90%) will narrow the interval but then the user is less confident that the interval contains the true slope. 95% is generally a good balance.

The R^2 line indicates how accurately the linear model fits the measurements. If the measurements aren't too noisy and the benchmark is performing the same amount of work for each iteration, this number should be very close to 1.0. If it is not, the benchmark results may be unreliable.

## Estimating Statistics

```
> Estimating the statistics of the sample
  >   mean [5.9994 ms 6.1268 ms]
  > median [5.9781 ms 5.9935 ms]
  >    MAD [16.247 us 32.562 us]
  >     SD [61.936 us 538.86 us]
```

Criterion-rs performs [bootstrap resampling](https://en.wikipedia.org/wiki/Bootstrapping_(statistics)) to estimate some important statistics about the samples collected. The number of bootstrap samples is configurable, and defaults to 100,000.

#### Mean/Median

These lines show a confidence interval on the mean and median values of the sample and give a good estimate of how long you can expect one iteration of the benchmark to take. The mean and median values should be quite close - if they are not, the benchmark may be unreliable due to a high number of outliers.

#### MAD/SD

These lines report the [Median Absolute Deviation](https://en.wikipedia.org/wiki/Median_absolute_deviation) and [Standard Deviation](https://en.wikipedia.org/wiki/Standard_deviation) of the sample. This is another indicator of noise in the benchmark. In a good benchmark, the MAD and SD will be relatively small compared to the mean and median. Large MAD or SD values indicate that there is a great deal of noise in the sample and the results may be unreliable or Criterion-rs may be unable to detect small optimizations or regressions.

## Comparing To Previous Sample

When a Criterion-rs benchmark is run, it saves statistical information in the `.criterion` directory. Subsequent executions of the benchmark will load this data and compare it with the current sample to show the effects of changes in the code.

```
alloc: Comparing with previous sample
> Performing a two-sample t-test
  > H0: Both samples have the same mean
  > p = 0
  > Strong evidence to reject the null hypothesis
> Estimating relative change of statistics
  >   mean [-34.062% -31.916%]
  > median [-33.186% -32.662%]
  > mean has improved by 33.00%
  > median has improved by 32.84%
```

First, Criterion-rs performs a statistical test to see whether the performance has changed significantly between runs (indicated by the 'Strong evidence...' line). Then it will proceed to estimate by how much the performance has changed. In this case, the performance has improved significantly. The `p = 0` line is an indicator of the chances that the observed differences in mean iteration time could have happened by chance alone. If the difference in means is large compared to the noise (indicated by p being close to zero) then it is likely there is a true difference in performance between the benchmarking runs.

```
alloc: Comparing with previous sample
> Performing a two-sample t-test
  > H0: Both samples have the same mean
  > p = 0.56624
  > Can't reject the null hypothesis
> Estimating relative change of statistics
  >   mean [-2.1989% +1.3191%]
  > median [-1.0305% -0.0042%]
```

In this example, the change in iteration time is much smaller. Note that p is much larger than zero and Criterion-rs reports that it can't reject the null hypothesis (ie. that it can't be confident the different is due to a true change in performance rather than chance). The threshold between 'Strong evidence...' and 'Can't reject...' is configurable, and defaults to 0.05. Increasing this threshold allows Criterion to detect smaller changes in noisier data at the cost of a greater chance of false positives.

```
alloc: Comparing with previous sample
> Performing a two-sample t-test
  > H0: Both samples have the same mean
  > p = 0
  > Strong evidence to reject the null hypothesis
> Estimating relative change of statistics
  >   mean [+54.245% +62.569%]
  > median [+49.228% +50.627%]
  > mean has regressed by 58.13%
  > median has regressed by 49.80%
thread 'alloc' panicked at 'alloc has regressed', src/analysis/compare.rs:58:8
note: Run with `RUST_BACKTRACE=1` for a backtrace.
test alloc ... FAILED
```

In this example, the performance has regressed significantly. When Criterion-rs is confident of a performance regression, it panics in order to fail the test.

## A Note Of Caution

Criterion-rs is designed to produce robust statistics when possible, but it can't account for everything. For example, the performance improvements and regressions listed in the above examples were created just by switching my laptop between battery power and wall power rather than changing the code under test. Care must be taken to ensure that benchmarks are performed under similar conditions in order to produce meaningful results.