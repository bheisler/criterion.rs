//! Kernels

use floaty::Floaty;

/// Kernel function
pub trait Kernel<A>: Copy + Fn(A) -> A + Sync where A: Floaty {}

impl<A, K> Kernel<A> for K where K: Copy + Fn(A) -> A + Sync, A: Floaty {}

/// Gaussian kernel
#[derive(Clone, Copy)]
pub struct Gaussian;

impl<A> Fn<(A,)> for Gaussian where A: Floaty {
    extern "rust-call" fn call(&self, (x,): (A,)) -> A {
        use std::f32::consts::PI;

        (x.powi(2).exp() * A::cast(2. * PI)).sqrt().recip()
    }
}

impl<A> FnMut<(A,)> for Gaussian where A: Floaty {
    extern "rust-call" fn call_mut(&mut self, args: (A,)) -> A {
        self.call(args)
    }
}

impl<A> FnOnce<(A,)> for Gaussian where A: Floaty {
    type Output = A;

    extern "rust-call" fn call_once(self, args: (A,)) -> A {
        self.call(args)
    }
}

#[cfg(test)]
macro_rules! test {
    ($ty:ident) => {
        mod $ty {
            mod gaussian {
                use quickcheck::TestResult;

                use univariate::kde::kernel::Gaussian;

                #[quickcheck]
                fn symmetric(x: $ty) -> bool {
                    relative_eq!(Gaussian(-x), Gaussian(x))
                }

                // Any [a b] integral should be in the range [0 1]
                #[quickcheck]
                fn integral(a: $ty, b: $ty) -> TestResult {
                    const DX: $ty = 1e-3;

                    if a > b {
                        TestResult::discard()
                    } else {
                        let mut acc = 0.;
                        let mut x = a;
                        let mut y = Gaussian(a);

                        while x < b {
                            acc += DX * y / 2.;

                            x += DX;
                            y = Gaussian(x);

                            acc += DX * y / 2.;
                        }

                        TestResult::from_bool(
                            (acc > 0. || relative_eq!(acc, 0.)) && (acc < 1. || relative_eq!(acc, 1.)))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    test!(f32);
    test!(f64);
}

