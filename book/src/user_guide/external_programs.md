# Benchmarking External Programs

Criterion.rs has the ability to benchmark external programs (which may be written in any language) the same way that it can benchmark Rust functions. What follows is an example of how that can be done and some of the pitfalls to avoid along the way.

First, let's define our recursive Fibonacci function, only in Python this time:

```python
def fibonacci(n):
    if n == 0 or n == 1:
        return 1
    return fibonacci(n-1) + fibonacci(n-2)
```

In order to benchmark this with Criterion.rs, we first need to write our own small benchmarking harness. I'll start with the complete code for this and then go over it in more detail:

```python
import time
import sys

MILLIS = 1000
MICROS = MILLIS * 1000
NANOS = MICROS * 1000

def benchmark():
    argument = int(sys.argv[1])

    for line in sys.stdin:
        iters = int(line.strip())

        # Setup

        start = time.perf_counter()
        for x in range(iters):
            fibonacci(argument)
        end = time.perf_counter()

        # Teardown

        delta = end - start
        nanos = int(delta * NANOS)
        print("%d" % nanos)
        sys.stdout.flush()

benchmark()
```

The important part is the `benchmark()` function.

### The Argument

```python
argument = int(sys.argv[1])
```

This example uses the `Criterion::bench_program_over_inputs` function to benchmark our Python Fibonacci function with a variety of inputs. The external program recieves the input value as a command-line argument appended to the command specified in the benchmark, so the very first thing our benchmark harness does is parse that argument into an integer. If we used `bench_program` instead, there would be no argument.

### Reading from stdin

```python
    for line in sys.stdin:
        iters = int(line.strip())
```

Next, our harness reads a line from stdin and parses it into an integer. Starting an external process is slow, and it would mess with our measurements if we had to do so for each iteration of the benchmark. Besides which, it would obscure the results (since we're probably more interested in the performance of the function without the process-creation overhead). Therefore, Criterion.rs starts the process once per input value or benchmark and sends the iteration counts to the external program on stdin. Your external benchmark harness must read and parse this iteration count and call the benchmarked function the appropriate number of times.

### Setup

If your benchmarked code requires any setup, this is the time to do that.

### Timing

```python
        start = time.perf_counter()
        for x in range(iters):
            fibonacci(argument)
        end = time.perf_counter()
```

This is the heart of the external benchmark harness. We measure how long it takes to execute our Fibonacci function with the given argument in a loop, iterating the given number of times. It's important here to use the most precise timer available. We'll need to report the measurement in nanoseconds later, so if you can use a timer that returns a value in nanoseconds (eg. Java's `System.nanoTime()`) we can skip a bit of work later. It's OK if the timer can't measure to nanosecond precision (most PC's can't) but use the best timer you have.

### Teardown

If your benchmarked code requires any teardown, this is the time to do that.

### Reporting

```python
        delta = end - start
        nanos = int(delta * NANOS)
        print("%d" % nanos)
        sys.stdout.flush()
```

To report the measured time, simply print the elapsed number of nanoseconds to stdout. `perf_counter` reports its results as a floating-point number of seconds, so we first convert it to an integer number of nanoseconds before printing it.

**Beware Buffering:** Criterion.rs will wait until it recieves the measurement before sending the next iteration count. If your benchmarks seem to be hanging during the warmup period, it may be because your benchmark harness is buffering the output on stdout, as Python does here. In this example we explicitly force Python to flush the buffer; you may need to do the same in your benchmarks.

## Defining the Benchmark

If you've read the earlier pages, this will be quite familiar.

```rust
use criterion::Criterion;
use std::process::Command;

fn create_command() -> Command {
    let mut command = Command::new("python3");
    command.arg("benches/external_process.py");
    command
}

fn python_fibonacci(c: &mut Criterion) {
    c.bench_program_over_inputs("fibonacci-python",
        create_command,
        &[1, 2, 4, 8, 16]);
}
```

As before, we create a `Criterion` struct and use it to define our benchmark. This time, we use the `bench_program_over_inputs` method. This takes a function (used to create the `Command` which represents our external program) and an iterable containing the inputs to test. Aside from the use of a `Command` rather than a closure, this behaves just like (and produces the same output as) `bench_function_over_inputs`.

If your benchmark doesn't require input, simply omit the input values and use `bench_program` instead, which behaves like `bench_function`.