extern crate test;
extern crate time;

use std::io::stdio;

fn main() {
    for line in stdio::stdin().lines() {
        let iters: u64 = from_str(line.unwrap().as_slice().trim()).unwrap();

        let start = time::precise_time_ns();
        for _ in range(0, iters) {
            let mut n = 15;
            test::black_box(&mut n);
            test::black_box(fibonacci(n));
        }
        let end = time::precise_time_ns();

        println!("{}", end - start);
    }
}

fn fibonacci(n: uint) -> uint {
    if n > 1 { fibonacci(n - 1) + fibonacci(n - 2) } else { n + 1 }
}
