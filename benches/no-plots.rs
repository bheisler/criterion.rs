extern crate criterion;

use std::io::fs;
use criterion::Criterion;

#[test]
fn no_plots() {
    Criterion::default().without_plots().bench("dummy", |b| b.iter(|| {}));

    assert!(!fs::walk_dir(&Path::new(".criterion/dummy")).ok().unwrap().any(|path| {
        path.extension_str() == Some("svg")
    }))
}
