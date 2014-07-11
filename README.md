[![Build Status](https://travis-ci.org/japaric/criterion.rs.svg?branch=master)](https://travis-ci.org/japaric/criterion.rs)

# criterion.rs

This is a port (with a few modifications) of
[Haskell "criterion" benchmarking library](http://www.serpentine.com/blog/2009/09/29/criterion-a-new-benchmarking-library-for-haskell)
to Rust.

Addresses [mozilla/rust#6812](https://github.com/mozilla/rust/issues/6812) and
I hope it'll help with
[mozilla/rust#7532](https://github.com/mozilla/rust/issues/7532)

I encourage you to look at this
[braindump](http://japaric.github.io/criterion-braindump), for an explanation
(with plots!) of how criterion works.

## Run the examples

```
$ make && make test
estimating the cost of precise_time_ns()
> warming up for 1000 ms
> collecting 100 measurements, 671088 iters each in estimated 1.2407 s
> found 3 outliers among 100 measurements (3.00%)
  > 3 (3.00%) high mild
> estimating statistics
  > bootstrapping sample with 100000 resamples
  > mean   18.492 ns ± 13.375 ps [18.467 ns 18.519 ns] 95% CI
  > median 18.483 ns ± 11.900 ps [18.459 ns 18.505 ns] 95% CI
  > MAD    126.40 ps ± 14.781 ps [93.148 ps 148.58 ps] 95% CI
  > SD     134.19 ps ± 14.304 ps [105.21 ps 161.04 ps] 95% CI

benchmarking fib_5
> warming up for 1000 ms
> collecting 100 measurements, 335544 iters each in estimated 1.2664 s
> found 14 outliers among 100 measurements (14.00%)
  > 14 (14.00%) high severe
> estimating statistics
  > bootstrapping sample with 100000 resamples
  > mean   37.810 ns ± 25.282 ps [37.763 ns 37.861 ns] 95% CI
  > median 37.812 ns ± 29.191 ps [37.701 ns 37.838 ns] 95% CI
  > MAD    262.27 ps ± 29.568 ps [191.77 ps 322.27 ps] 95% CI
  > SD     235.87 ps ± 22.632 ps [189.76 ps 277.61 ps] 95% CI
> comparing with previous sample
  > bootstrapping sample with 100000 resamples
  > mean   +0.1189% ± 0.0782% [-0.0324% +0.2753%] 95% CI
  > median +0.0288% ± 0.1395% [-0.2382% +0.3090%] 95% CI

benchmarking fib_10
> warming up for 1000 ms
> collecting 100 measurements, 41943 iters each in estimated 1.9154 s
> found 12 outliers among 100 measurements (12.00%)
  > 2 (2.00%) low mild
  > 2 (2.00%) high mild
  > 8 (8.00%) high severe
> estimating statistics
  > bootstrapping sample with 100000 resamples
  > mean   456.70 ns ± 141.17 ps [456.43 ns 456.98 ns] 95% CI
  > median 456.55 ns ± 146.25 ps [456.37 ns 456.96 ns] 95% CI
  > MAD    1.2904 ns ± 282.12 ps [660.92 ps 1.6421 ns] 95% CI
  > SD     1.3627 ns ± 122.72 ps [1.1165 ns 1.5966 ns] 95% CI
> comparing with previous sample
  > bootstrapping sample with 100000 resamples
  > mean   +0.0601% ± 0.0452% [-0.0280% +0.1487%] 95% CI
  > median +0.0169% ± 0.0459% [-0.0334% +0.1409%] 95% CI

benchmarking fib_15
> warming up for 1000 ms
> collecting 100 measurements, 2621 iters each in estimated 1.3323 s
> found 3 outliers among 100 measurements (3.00%)
  > 3 (3.00%) high severe
> estimating statistics
  > bootstrapping sample with 100000 resamples
  > mean   5.0777 us ± 1.9671 ns [5.0739 us 5.0816 us] 95% CI
  > median 5.0803 us ± 5.8789 ns [5.0661 us 5.0867 us] 95% CI
  > MAD    23.692 ns ± 2.9360 ns [16.897 ns 29.377 ns] 95% CI
  > SD     19.461 ns ± 1.1989 ns [17.061 ns 21.755 ns] 95% CI
> comparing with previous sample
  > bootstrapping sample with 100000 resamples
  > mean   -0.0234% ± 0.0659% [-0.1530% +0.1048%] 95% CI
  > median -0.0161% ± 0.1450% [-0.2547% +0.3206%] 95% CI
(...)
```

## Progress

This section is outdated, see the
[Roadmap](https://github.com/japaric/criterion.rs/issues/2) instead

### Done so far

* Estimation of the cost of reading the clock (`precise_time_ns()`)
* Outlier classification using the box plot method (IQR criteria)
* Removal of severe outliers (this is **not** done in the original criterion)
* Bootstrapping: point estimate, standard error and confidence interval
* Convert to library
* Bencher-like interface
* Bencher configuration
* Benchmark groups
* Some examples
* Save metrics to json file
* ~~Hypothesis testing~~
  * Do the old and new sample belong to the same population?
  * Has the benchmark regressed by at least 3 standard errors?
  * Removed at the moment
    * Not sure if 3 or 5 standard errors is a good metric to determine a
      regression
    * It's hard to relate the standard error to a % change (which the Bencher
      infrastructure uses)
* Report improvement/regression with a confidence interval

### Not (yet?) ported from the original

* outlierVariance, this method computes the influence of the outliers on the
  variance of the sample
  * this still looks too magical to me, using only the sample size, and the
    point estimates of the mean and the standard deviation, the author
    classifies the effect of the outliers on the sample variance
    * there are no references of the method used to do this
  * some rough ideas that might accomplish this:
    * the SEM (standard error of the mean) is the variance of the population
      over the square root of the sample size, I could compute the variance of
      the population and compare it against the bootstrapped variance.
    * Fit the bootstrapped distribution to a normal distribution, and look at
      the R squared.
    * Look at the skewness of the bootstrap distributions.

### TODO

* More testing
* Redirect benchmark stdout to /dev/null
* Compare the results generated by criterion.rs with the results generated by
  Rust Bencher algorithm
  * Rust Bencher reports smaller times in some benchmarks
* Compare the current basic bootstrap against the BCa (bias corrected and
  accelerated) bootstrap
* Check if the sample is garbage
  * may be caused by CPU throttling or CPU usage peaks
    * should translate into high variance in the sample
  * background constant CPU usage should be hard to detect
    * this affects more the mean than the variance
* Documentation
* Ratchet metrics using confidence intervals

# Wishlist

* Plot the [PDF](http://en.wikipedia.org/wiki/Probability_density_function) of
  the sample
  * computing the PDF is expensive
  * PDF from the sample is not too reliable, a PDF from the bootstrap would be
    better, but that would be even more expensive
  * need plotting library
    * gnuplot? is the license compatible with Apache/MIT?
  * How to select the X range of the PDF
* Interface to benchmark external programs (written in other languages)
  * Addresses the last point in
    [mozilla/rust#7532](https://github.com/mozilla/rust/issues/7532)
  * Something like [eulermark.rs](https://github.com/japaric/eulermark.rs)
    * See eulermark results [here](http://japaric.github.io/eulermark.rs)

## Unresolved questions

* Is sensible to remove the severe outliers in **all** the cases?
  * Removing outliers will always reduce the variance in the sample
* Can we continuously remove the severe outliers from the sample, until the box
  plot analysis yields no more severe outliers?
* When performing several benchmarks, heavy benchmark may affect the benchmarks
  that follow (hot CPU?), how do we address this?
  * Add a cooldown time between benchmarks?

## License

criterion.rs is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.
