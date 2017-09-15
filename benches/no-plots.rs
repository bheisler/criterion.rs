extern crate criterion;
extern crate walkdir;

use criterion::Criterion;

use walkdir::WalkDir;

#[test]
fn no_plots() {
    Criterion::default().without_plots().bench_function("dummy", |b| b.iter(|| {}));

    let has_svg = !WalkDir::new(".criterion/dummy").into_iter().any(|entry| {
        entry.unwrap().path().extension().and_then(|ext| ext.to_str()) == Some("svg")
    });
    assert!(has_svg)
}
