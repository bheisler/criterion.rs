extern crate test;
extern crate time;

use std::io::stdio;
use std::os;

fn main() {
    let args = os::args();
    let input = match args.as_slice() {
        [_, ref arg] => from_str(arg.as_slice()).expect("Expected an integer as input"),
        _ => fail!("Expected one input"),
    };

    for line in stdio::stdin().lines() {
        let iters: u64 = from_str(line.unwrap().as_slice().trim()).unwrap();

        let start = time::precise_time_ns();
        for _ in range(0, iters) {
            test::black_box(Vec::from_elem(input, 0u));
        }
        let end = time::precise_time_ns();

        println!("{}", end - start);
    }
}
