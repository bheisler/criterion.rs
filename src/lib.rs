#![crate_id="criterion#0.11-pre"]
#![crate_type="lib"]

extern crate rand;
extern crate serialize;
extern crate test;
extern crate time;

pub use bencher::{Bencher,BencherConfig};

mod bencher;
mod bootstrap;
mod clock;
mod common;
mod outlier;
mod sample;
mod units;
