use std::hint::black_box;

use criterion::{criterion_group, Criterion, Throughput};

fn fibonacci_slow(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

fn fibonacci_vec(n: u64) -> u64 {
    let mut values = Vec::with_capacity((n as usize) + 2);
    values.push(1);
    values.push(1);
    for i in 2..=n as usize {
        let next = values[i - 1] + values[i - 2];
        values.push(next);
    }
    *values.last().unwrap()
}

fn fibonacci_memoized(n: u64, cache: &mut Vec<Option<u64>>) -> u64 {
    if let Some(value) = cache.get(n as usize).and_then(|v| *v) {
        return value;
    }

    let value = match n {
        0 | 1 => 1,
        _ => fibonacci_memoized(n - 1, cache) + fibonacci_memoized(n - 2, cache),
    };

    if let Some(slot) = cache.get_mut(n as usize) {
        *slot = Some(value);
    }

    value
}

fn comparison_demo(c: &mut Criterion) {
    let mut group = c.comparison_benchmark_group("FibonacciComparison");
    let n = 20u64;

    group.throughput(Throughput::Elements(1));

    group.bench_function("Recursive", |b| {
        b.iter(|| black_box(fibonacci_slow(black_box(n))))
    });
    group.bench_function("Iterative", |b| {
        b.iter(|| black_box(fibonacci_fast(black_box(n))))
    });
    group.bench_function("Vec build", |b| {
        b.iter(|| black_box(fibonacci_vec(black_box(n))))
    });
    group.bench_function("Memoized", |b| {
        b.iter(|| {
            let mut cache = vec![None; (n + 1) as usize];
            black_box(fibonacci_memoized(black_box(n), &mut cache))
        })
    });

    group.finish();
}

criterion_group!(comparison_group, comparison_demo);
