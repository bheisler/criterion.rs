use std::num::Float;
use std::rand::{Rand, Rng, XorShiftRng, self};

pub const BENCH_SIZE: uint = 1_000_000;

pub fn vec<T>(size: uint) -> Option<Vec<T>> where T: Rand {
    if size > 1 {
        let mut rng: XorShiftRng = rand::thread_rng().gen();

        Some(range(0, size).map(|_| rng.gen()).collect())
    } else {
        None
    }
}

pub trait ApproxEq {
    fn approx_eq(self, other: Self) -> bool;
}

macro_rules! approx {
    ($($ty:ty),+) => {$(
        impl ApproxEq for $ty {
            fn approx_eq(self, other: $ty) -> bool {
                const EPS: $ty = 1e-5;

                if other == 0. {
                    self.abs() < EPS
                } else {
                    (self / other - 1.) < EPS
                }
            }
        }

        impl ApproxEq for ($ty, $ty, $ty) {
            fn approx_eq(self, other: ($ty, $ty, $ty)) -> bool {
                self.0.approx_eq(other.0) && self.1.approx_eq(other.1) && self.2.approx_eq(other.2)
            }
        })+
    }
}

approx!(f32, f64);
