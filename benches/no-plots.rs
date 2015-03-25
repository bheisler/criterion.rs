#![feature(fs_walk)]

extern crate criterion;

use std::fs;
use std::path::Path;

use criterion::Criterion;

#[test]
fn no_plots() {
    Criterion::default().without_plots().bench("dummy", |b| b.iter(|| {}));

    assert!(!fs::walk_dir(&Path::new(".criterion/dummy")).ok().unwrap().any(|entry| {
        entry.unwrap().path().extension().and_then(|ext| ext.to_str()) == Some("svg")
    }))
}
