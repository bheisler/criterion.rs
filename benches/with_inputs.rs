use std::iter;

use criterion::Criterion;

fn from_elem(c: &mut Criterion) {
    static KB: usize = 1024;

    c.bench_function_over_inputs(
        "from_elem",
        |b, &size| {
            b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
        },
        vec![KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB],
    );
}

criterion_group!(benches, from_elem);
