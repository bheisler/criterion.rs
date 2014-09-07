extern crate criterion;

use criterion::Criterion;

#[test]
fn from_elem() {
    let can_plot = Criterion::default().bench_with_inputs("from_elem", |b, &size| {
        b.iter(|| Vec::from_elem(size, 0u8));
    }, [1024, 2048, 4096]).can_plot();

    if can_plot {
        // Check that the summary plots have been generated
        let summ_dir = Path::new(".criterion/from_elem/summary/new");
        assert!(summ_dir.join("means.svg").exists() && summ_dir.join("medians.svg").exists())
    }
}
