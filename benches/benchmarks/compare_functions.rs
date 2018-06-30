use criterion::Criterion;
use criterion::Fun;
use criterion::ParameterizedBenchmark;

fn fibonacci_slow(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci_slow(n - 1) + fibonacci_slow(n - 2),
    }
}

fn fibonacci_fast(n: u64) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut c: u64;

    if n == 0 {
        return 0;
    }

    for _ in 0..(n + 1) {
        c = a + b;
        a = b;
        b = c;
    }
    b
}

fn compare_fibonaccis(c: &mut Criterion) {
    let fib_slow = Fun::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)));
    let fib_fast = Fun::new("Iterative", |b, i| b.iter(|| fibonacci_fast(*i)));

    let functions = vec![fib_slow, fib_fast];

    c.bench_functions("Fibonacci", functions, 20);
}
fn compare_fibonaccis_builder(c: &mut Criterion) {
    c.bench(
        "Fibonacci2",
        ParameterizedBenchmark::new(
            "Recursive",
            |b, i| b.iter(|| fibonacci_slow(*i)),
            vec![20u64, 21u64],
        ).with_function("Iterative", |b, i| b.iter(|| fibonacci_fast(*i))),
    );
}

fn compare_looped(c: &mut Criterion) {
    use criterion::ParameterizedBenchmark;
    use criterion::black_box;

    c.bench(
        "small",
        ParameterizedBenchmark::new("unlooped", |b, i| b.iter(|| i + 10), vec![10]).with_function(
            "looped",
            |b, i| {
                b.iter(|| {
                    for _ in 0..10_000 {
                        black_box(i + 10);
                    }
                })
            },
        ),
    );
}

criterion_group!(
    fibonaccis,
    compare_fibonaccis,
    compare_fibonaccis_builder,
    compare_looped
);
