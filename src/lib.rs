#![crate_id="criterion#0.11-pre"]
#![crate_type="lib"]

#![feature(default_type_params)]

extern crate collections;
extern crate rand;
extern crate serialize;
extern crate test;
extern crate time;

pub use bencher::{Bencher,BencherConfig};

mod bencher;
mod bootstrap;
mod clock;
mod common;
mod metrics;
mod outlier;
mod sample;
mod units;
