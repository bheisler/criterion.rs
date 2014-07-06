use bencher::Bencher;
use time::precise_time_ns;

pub fn run_for_at_least(how_long: u64,
                        seed: u64,
                        f: |&mut Bencher|)
    -> (u64, u64)
{
    let mut b = Bencher::new();
    let mut iters = seed;
    let mut tries = 0u;

    let init = precise_time_ns();
    loop {
        b.bench_n(iters, |x| f(x));

        let elapsed = b.ns_elapsed();

        if elapsed > how_long {
            return (elapsed, iters);
        }

        iters *= 2;
        tries += 1;

        if precise_time_ns() - init > 10 * how_long {
            fail!("took too long to run: seed {}, tries {}", seed, tries);
        }
    }
}
