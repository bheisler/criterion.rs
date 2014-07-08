#![feature(macro_rules)]

extern crate serialize;
extern crate simplot;
extern crate test;
extern crate time;

pub use bencher::Bencher;
pub use criterion::Criterion;

mod analyze;
mod bencher;
mod clock;
mod common;
mod criterion;
mod file;
mod fs;
mod math;
mod outliers;
mod plot;
mod sample;
mod statistics;
mod units;
