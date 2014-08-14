[![Build Status](https://travis-ci.org/japaric/stats.rs.svg?branch=master)](https://travis-ci.org/japaric/stats.rs)

# stats.rs

A Rust library to do statistics.

*Note* Only relatively simple routines are available at the moment, more
complex statistics require a matrix library. Plus, the main user of this
library is [criterion.rs][criterion], which only requires these routines.

## [API Docs][docs]

## Features

* bootstrap primitives via case resampling
* classification of outliers
* simple linear regression
* thread parallelized routines
* two sample t-test
* univariate kernel density estimation

## License

stats.rs is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.

[criterion]: https://github.com/japaric/criterion.rs
[docs]: https://github.com/japaric/stats.rs/stats/index.html
