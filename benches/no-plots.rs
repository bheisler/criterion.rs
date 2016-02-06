extern crate criterion;
extern crate walkdir;

use criterion::Criterion;

use walkdir::WalkDir;

#[test]
fn no_plots() {
    Criterion::default().without_plots().bench("dummy", |b| b.iter(|| {}));

    assert!(!WalkDir::new(".criterion/dummy").into_iter().any(|entry| {
        entry.unwrap().path().extension().and_then(|ext| ext.to_str()) == Some("svg")
    }))
}
