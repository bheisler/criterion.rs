extern crate criterion;

use std::time::Duration;

use criterion::{Criterion, Function};

fn main() {
    Criterion::default()
        .measurement_time(Duration::from_secs(3))
        .inputs(vec![1024, 32 * 1024, 1024 * 1024])
        .function(Function("alloc", |b, i| {}))
        .bench()
/* Output
Benchmarking alloc with input 1024
Benchmarking alloc with input 32768
Benchmarking alloc with input 1048576
*/
        .inputs(vec![7, 11, 13])
        .functions(vec![Function("par_fib", |b, i| {}), Function("seq_fib", |b, _| {})])
        .bench()
/* Output
Benchmarking par_fib with input 7
Benchmarking par_fib with input 11
Benchmarking par_fib with input 13
Benchmarking seq_fib with input 7
Benchmarking seq_fib with input 11
Benchmarking seq_fib with input 13
*/
        .function(Function("no_input", |b, &()| {}))
        //^ FIXME second argument of the closure is unnecessary
        .bench()
/* Output
Benchmarking no_input
*/
        .functions(vec![Function("A", |b, &()| {}), Function("B", |b, &()| {})])
        .bench();
/* Output
Benchmarking A
Benchmarking B
*/
}
