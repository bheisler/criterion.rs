#![crate_name = "criterion"]
#![feature(macro_rules)]

extern crate serialize;
extern crate simplot;
extern crate test;

pub use bencher::Bencher;
pub use criterion::Criterion;
pub use time::traits::{Milisecond,Second};

mod bencher;
mod criterion;
mod fs;
mod math;
mod outliers;
mod plot;
mod statistics;
mod stream;
mod target;
mod time;
