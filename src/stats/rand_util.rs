use rand_core::{RngCore, SeedableRng};
use rand_os::OsRng;
use rand_xoshiro::Xoshiro256StarStar;

use std::cell::RefCell;

pub type Rng = Xoshiro256StarStar;

thread_local! {
    static SEED_RAND: RefCell<OsRng> = RefCell::new(OsRng::new().unwrap());
}

pub fn new_rng() -> Rng {
    SEED_RAND.with(|r| {
        let mut seed = [0u8; 32];
        r.borrow_mut().fill_bytes(&mut seed);
        Xoshiro256StarStar::from_seed(seed)
    })
}

/// Source of uniform random numbers within a range of usize values.
pub struct Range {
    low: usize,
    range: usize,
    zone: usize,
}
impl Range {
    pub fn new_exclusive(low: usize, high: usize) -> Range {
        Range::new_inclusive(low, high - 1)
    }

    pub fn new_inclusive(low: usize, high: usize) -> Range {
        let max = ::std::usize::MAX;
        assert!(low < high);
        assert!(high < max);
        let range = high.wrapping_sub(low).wrapping_add(1);
        let ints_to_reject = (max - range + 1) % range;
        let zone = max - ints_to_reject;

        Range { low, range, zone }
    }

    pub fn sample<R: RngCore>(&self, rng: &mut R) -> usize {
        loop {
            let value: usize = rng.next_u64() as usize;

            if value <= self.zone {
                return (value % self.range) + self.low;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;
    use std::cmp;

    quickcheck! {
        fn range_returns_samples_in_range(low: usize, high: usize) -> TestResult {
            let (low, high) = (cmp::min(low, high), cmp::max(low, high));
            if (high - low) < 2 || high == ::std::usize::MAX {
                return TestResult::discard();
            }

            let mut rng = new_rng();
            let range = Range::new_inclusive(low, high);

            for _ in 0..1000 {
                let value = range.sample(&mut rng);
                if !(low <= value && value <= high) {
                    return TestResult::from_bool(false);
                }
            }

            return TestResult::from_bool(true);
        }
    }

    struct CountingRng {
        count: u64,
    }
    impl ::rand_core::RngCore for CountingRng {
        fn next_u32(&mut self) -> u32 {
            self.next_u64() as u32
        }

        fn next_u64(&mut self) -> u64 {
            let value = self.count;
            self.count = self.count.wrapping_add(1);
            value
        }

        fn fill_bytes(&mut self, _dest: &mut [u8]) {
            unimplemented!()
        }

        fn try_fill_bytes(
            &mut self,
            _dest: &mut [u8],
        ) -> ::std::result::Result<(), ::rand_core::Error> {
            unimplemented!()
        }
    }

    // These are arbitrary
    const SIZE: usize = 17;
    const ROUNDS: usize = 200;

    #[test]
    fn range_is_uniform() {
        let mut rng = CountingRng { count: 0 };
        let range = Range::new_exclusive(0, SIZE);
        let mut array = [0usize; SIZE];

        for _ in 0..(ROUNDS * SIZE) {
            let index = range.sample(&mut rng);
            array[index] += 1;
        }

        assert_eq!([ROUNDS; SIZE], array);
    }

    #[test]
    fn range_is_uniform_outside_of_zone() {
        // Check that range is still uniform when provided with numbers near u64::MAX.
        let mut rng = CountingRng {
            count: ::std::u64::MAX - ((SIZE * ROUNDS) / 2) as u64,
        };
        let range = Range::new_exclusive(0, SIZE);
        let mut array = [0usize; SIZE];

        for _ in 0..(ROUNDS * SIZE) {
            let index = range.sample(&mut rng);
            array[index] += 1;
        }

        assert_eq!([ROUNDS; SIZE], array);
    }
}
