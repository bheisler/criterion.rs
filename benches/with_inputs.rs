#![feature(path_ext)]

extern crate criterion;

use std::fs::PathExt;
use std::iter;
use std::path::Path;

use criterion::Criterion;

#[test]
fn from_elem() {
    static KB: usize = 1024;

    let can_plot = Criterion::default().bench_with_inputs("from_elem", |b, &&size| {
        b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
    }, &[KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB]).can_plot();

    if can_plot {
        // Check that the summary plots have been generated
        let summ_dir = Path::new(".criterion/from_elem/summary/new");
        assert!(summ_dir.join("means.svg").exists() && summ_dir.join("medians.svg").exists())
    }
}
