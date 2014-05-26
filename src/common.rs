use test::black_box;
use time::precise_time_ns;

pub fn run_for_at_least<'a, T>(how_long: u64,
                               seed: uint,
                               action: ||:'a -> T)
    -> (u64, uint, ||:'a -> T)
{
    let mut iters = seed;
    let mut tries = 0;

    let init = precise_time_ns();
    loop {
        let start = precise_time_ns();
        for _ in range(0, iters) {
            black_box(action());
        }
        let elapsed = precise_time_ns() - start;

        if elapsed > how_long {
            return (elapsed, iters, action);
        }

        iters *= 2;
        tries += 1;

        if precise_time_ns() - init > 10 * how_long {
            fail!("took too long to run: seed {}, tries {}", seed, tries);
        }
    }
}
