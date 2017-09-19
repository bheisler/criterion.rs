extern crate criterion;

use std::mem;

use criterion::Criterion;
use std::time::Duration;

const SIZE: usize = 1024 * 1024;

#[test]
fn dealloc() {
    let mut c = Criterion::default();
    c.warm_up_time(Duration::new(1, 0));
    c.bench_function("large_dealloc", |b| {
            b.iter_with_large_setup(|| (0..SIZE).map(|_| 0u8).collect::<Vec<_>>(),
                                    mem::drop);
        });
}
