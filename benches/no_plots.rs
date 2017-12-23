use criterion::Criterion;
use walkdir::WalkDir;

fn config() -> Criterion {
    let mut c = Criterion::default();
    c.without_plots();
    c
}

fn no_plots(c: &mut Criterion) {
    c.bench_function("dummy", |b| b.iter(|| {}));

    let has_svg = !WalkDir::new(".criterion/dummy").into_iter().any(|entry| {
        let entry = entry.ok();
        entry
            .as_ref()
            .and_then(|entry| entry.path().extension())
            .and_then(|ext| ext.to_str()) == Some("svg")
    });
    assert!(has_svg)
}

criterion_group!(
    name = benches;
    config = config();
    targets = no_plots
);