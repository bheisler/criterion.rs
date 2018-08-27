# Benchmarking With Inputs

Criterion.rs can run benchmarks with multiple different input values to investigate how the performance behavior changes with different inputs.

```rust
    static KB: usize = 1024;

    Criterion::default()
    .bench_function_over_inputs("from_elem", |b, &&size| {
        b.iter(|| iter::repeat(0u8).take(size).collect::<Vec<_>>());
    }, &[KB, 2 * KB, 4 * KB, 8 * KB, 16 * KB])
```

In this example, we're benchmarking the time it takes to collect a iterator producing a sequence of N bytes into a Vec. We use the `bench_function_over_inputs` method. Unlike `bench_function`, the lambda here takes a Bencher and a reference to a parameter, in this case `size`. Finally, we provide a slice of potential input values. This generates five benchmarks, named "from_elem/1024" through "from_elem/16384" which individually behave the same as any other benchmark. Criterion.rs also generates some charts in `target/criterion/from_elem/report/` showing how the iteration time changes as a function of the input.

![Line Chart](./user_guide/line.svg)

Here we can see that there is a approximately-linear relationship between the length of an iterator and the time taken to collect it into a Vec.
