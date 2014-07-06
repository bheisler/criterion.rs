#![crate_id="criterion"]
#![feature(macro_rules)]

extern crate serialize;
extern crate simplot;
extern crate test;
extern crate time;

pub use bencher::Bencher;
pub use criterion::Criterion;

mod bencher;
mod bootstrap;
mod clock;
mod common;
mod criterion;
mod math;
mod outlier;
mod sample;
mod units;
