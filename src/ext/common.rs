use time;

use ext::bencher::Bencher;

pub fn run_for_at_least(how_long: u64,
                        seed: u64,
                        b: &mut Bencher)
                        -> (u64, u64) {
    let mut iters = seed;
    let mut tries = 0u;

    let init = time::precise_time_ns();
    loop {
        b.bench_n(iters);

        let elapsed = b.ns_elapsed();

        if elapsed > how_long {
            return (elapsed, iters);
        }

        iters *= 2;
        tries += 1;

        if time::precise_time_ns() - init > 10 * how_long {
            fail!("took too long to run: seed {}, tries {}", seed, tries);
        }
    }
}
