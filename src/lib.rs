#![deny(warnings)]
#![feature(overloaded_calls, phase)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;
extern crate serialize;
// FIXME Drop dependency on the `test` crate
extern crate test;

pub use ci::ConfidenceInterval;
pub use distribution::Distribution;
pub use sample::Sample;
pub use stats::{mean, median, median_abs_dev, std_dev, t, var};

pub mod kde;
pub mod outliers;
pub mod regression;
pub mod ttest;

mod ci;
mod distribution;
mod resamples;
mod sample;
mod stats;
#[cfg(test)]
mod tol;
