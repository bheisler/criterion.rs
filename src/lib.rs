#![feature(macro_rules, phase)]

#[phase(plugin, link)]
extern crate log;
extern crate serialize;
extern crate simplot;
extern crate test;

pub use criterion::Criterion;
pub use target::Bencher;

mod criterion;
mod fs;
mod math;
mod outliers;
mod plot;
mod statistics;
mod stream;
mod target;
mod time;
