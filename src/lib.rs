#![crate_id="criterion"]
#![feature(macro_rules)]

extern crate serialize;
extern crate test;
extern crate time;

pub use bencher::Bencher;
pub use criterion::{Criterion,CriterionConfig};

mod bencher;
mod bootstrap;
mod clock;
mod common;
mod criterion;
mod metrics;
mod outlier;
mod sample;
mod units;
