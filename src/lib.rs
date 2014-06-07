#![crate_id="criterion#0.11-pre"]
#![crate_type="lib"]

#![feature(default_type_params)]

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
