extern crate criterion;

use criterion::Criterion;

#[test]
fn from_elem() {
    Criterion::default().bench_with_inputs("from_elem", |b, &size| {
        b.iter(|| Vec::from_elem(size, 0u8));
    }, [1024, 2048, 4096]);

    // Check that the summary plots have been generated
    let summary_dir = Path::new(".criterion/from_elem/summary/new");
    assert!(summary_dir.join("means.svg").exists() && summary_dir.join("medians.svg").exists())
}
