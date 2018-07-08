# CSV Output

Criterion.rs saves its measurements in several files, as shown below:

```
$BENCHMARK/
├── base/
│  ├── raw.csv
│  ├── estimates.json
│  ├── sample.json
│  └── tukey.json
├── change/
│  └── estimates.json
├── new/
│  ├── raw.csv
│  ├── estimates.json
│  ├── sample.json
│  └── tukey.json
```

The JSON files are all considered private implementation details of Criterion.rs, and their
structure may change at any time without warning.

However, there is a need for some sort of stable and machine-readable output to enable projects like
[lolbench](https://github.com/anp/lolbench) to keep historical data or perform additional analysis
on the measurements. For this reason, Criterion.rs also writes the `raw.csv` file. The format of
this file is expected to remain stable between different versions of Criterion.rs, so this file is
suitable for external tools to depend on.

The format of `raw.csv` is as follows:

```
group,function,value,sample_time_nanos,iteration_count
Fibonacci,Iterative,,915000,110740
Fibonacci,Iterative,,1964000,221480
Fibonacci,Iterative,,2812000,332220
Fibonacci,Iterative,,3767000,442960
Fibonacci,Iterative,,4785000,553700
Fibonacci,Iterative,,6302000,664440
Fibonacci,Iterative,,6946000,775180
Fibonacci,Iterative,,7815000,885920
Fibonacci,Iterative,,9186000,996660
Fibonacci,Iterative,,9578000,1107400
Fibonacci,Iterative,,11206000,1218140
...
```

This data was taken with this benchmark code:

```rust
fn compare_fibonaccis(c: &mut Criterion) {
    let fib_slow = Fun::new("Recursive", |b, i| b.iter(|| fibonacci_slow(*i)));
    let fib_fast = Fun::new("Iterative", |b, i| b.iter(|| fibonacci_fast(*i)));

    let functions = vec![fib_slow, fib_fast];

    c.bench_functions("Fibonacci", functions, 20);
}
```

`raw.csv` contains the following columns:
 - `group` - This corresponds to the function group name, in this case "Fibonacci" as seen in the
code above. This is the parameter given to the `Criterion::bench` functions.
 - `function` - This corresponds to the function name, in this case "Iterative". When comparing
multiple functions, each function is given a different name. Otherwise, this will be the empty
string.
 - `value` - This is the parameter passed to the benchmarked function when using parameterized
benchmarks. In this case, there is no parameter so the value is the empty string.
 - `iteration_count` - The number of times the benchmark was iterated for this sample.
 - `sample_time_nanos` - The time taken by the measurement for this sample, in nanoseconds. Note
that this is the time for the whole sample, not the time-per-iteration (see 
[Analysis Process](/analysis.html#measurement) for more detail). To calculate the time-per-iteration,
use `sample_time_nanos/iteration_count`.

As you can see, this is the raw measurements taken by the Criterion.rs benchmark process. There is
one record for each sample, and one file for each benchmark.

The results of Criterion.rs' analysis of these measurements are not currently available in
machine-readable form. If you need access to this information, please raise an issue describing
your use case.
