use cast::From;

use Float;

/// A "view" into the percentiles of a sample
pub struct Percentiles<A>(Box<[A]>) where A: Float;

// TODO(rust-lang/rfcs#735) move this `impl` into a private percentiles module
impl<A> Percentiles<A> where A: Float, usize: From<A, Output=Option<usize>> {
    /// Returns the percentile at `p`%
    ///
    /// Safety:
    ///
    /// - Make sure that `p` is in the range `[0, 100]`
    unsafe fn at_unchecked(&self, p: A) -> A {
        let _0 = A::from(0);
        let _100 = A::from(100);
        let len = self.0.len() - 1;

        if p == _100 {
            self.0[len]
        } else {
            let rank = (p / _100) * A::from(len);
            let integer = rank.floor();
            let fraction = rank - integer;
            let n = usize::from(integer).unwrap();
            let &floor = self.0.get_unchecked(n);
            let &ceiling = self.0.get_unchecked(n + 1);

            floor + (ceiling - floor) * fraction
        }
    }

    /// Returns the percentile at `p`%
    ///
    /// # Panics
    ///
    /// Panics if `p` is outside the closed `[0, 100]` range
    pub fn at(&self, p: A) -> A {
        let _0 = A::from(0);
        let _100 = A::from(100);

        assert!(p >= _0 && p <= _100);

        unsafe {
            self.at_unchecked(p)
        }
    }

    /// Returns the interquartile range
    pub fn iqr(&self) -> A {
        unsafe {
            let q1 = self.at_unchecked(A::from(25));
            let q3 = self.at_unchecked(A::from(75));

            q3 - q1
        }
    }

    /// Returns the 50th percentile
    pub fn median(&self) -> A {
        unsafe {
            self.at_unchecked(A::from(50))
        }
    }

    /// Returns the 25th, 50th and 75th percentiles
    pub fn quartiles(&self) -> (A, A, A) {
        unsafe {
            (
                self.at_unchecked(A::from(25)),
                self.at_unchecked(A::from(50)),
                self.at_unchecked(A::from(75)),
            )
        }
    }
}

