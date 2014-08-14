#![feature(macro_rules, phase)]

#[phase(plugin, link)]
extern crate log;
extern crate serialize;
extern crate simplot;
extern crate stats;
extern crate test;
extern crate time;

pub use criterion::Criterion;
pub use target::Bencher;

mod criterion;
mod estimate;
mod fs;
mod kde;
mod plot;
mod stream;
mod target;
