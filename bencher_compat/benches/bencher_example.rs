#[macro_use]
extern crate criterion_bencher_compat;

use criterion_bencher_compat::Bencher;

fn a(bench: &mut Bencher) {
    bench.iter(|| {
        (0..1000).fold(0, |x, y| x + y)
    })
}

fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| {
        vec![0u8; N]
    });

    bench.bytes = N as u64;
}

benchmark_group!(benches, a, b);
benchmark_main!(benches);