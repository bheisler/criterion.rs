#[cfg(test)]
macro_rules! test {
    ($ty:ident) => {
        mod $ty {
            use quickcheck::TestResult;

            use univariate::{Sample, mixed, self};

            quickcheck!{
                fn mean(size: usize, start: usize, nresamples: usize) -> TestResult {
                    if let Some(v) = ::test::vec::<$ty>(size, start) {
                        let sample = Sample::new(&v[start..]);

                        let means = if nresamples > 0 {
                            sample.bootstrap(nresamples, |s| (s.mean(),)).0
                        } else {
                            return TestResult::discard();
                        };

                        let min = sample.min();
                        let max = sample.max();

                        TestResult::from_bool(
                            // Computed the correct number of resamples
                            means.as_slice().len() == nresamples &&
                            // No uninitialized values
                            means.as_slice().iter().all(|&x| {
                                (x > min || relative_eq!(x, min)) &&
                                (x < max || relative_eq!(x, max))
                            })
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }

            quickcheck!{
                fn mean_median(size: usize, start: usize, nresamples: usize) -> TestResult {
                    if let Some(v) = ::test::vec::<$ty>(size, start) {
                        let sample = Sample::new(&v[start..]);

                        let (means, medians) = if nresamples > 0 {
                            sample.bootstrap(nresamples, |s| (s.mean(), s.median()))
                        } else {
                            return TestResult::discard();
                        };

                        let min = sample.min();
                        let max = sample.max();

                        TestResult::from_bool(
                            // Computed the correct number of resamples
                            means.as_slice().len() == nresamples &&
                            medians.as_slice().len() == nresamples &&
                            // No uninitialized values
                            means.as_slice().iter().all(|&x| {
                                (x > min || relative_eq!(x, min)) &&
                                (x < max || relative_eq!(x, max))
                            }) &&
                            medians.as_slice().iter().all(|&x| {
                                (x > min || relative_eq!(x, min)) &&
                                (x < max || relative_eq!(x, max))
                            })
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }

            quickcheck!{
                fn mixed_two_sample(
                    a_size: usize, a_start: usize,
                    b_size: usize, b_start: usize,
                    nresamples: usize
                ) -> TestResult {
                    if let (Some(a), Some(b)) =
                        (::test::vec::<$ty>(a_size, a_start), ::test::vec::<$ty>(b_size, b_start))
                    {
                        let a = Sample::new(&a);
                        let b = Sample::new(&b);

                        let distribution = if nresamples > 0 {
                            mixed::bootstrap(a, b, nresamples, |a, b| (a.mean() - b.mean(),)).0
                        } else {
                            return TestResult::discard();
                        };

                        let min = <$ty>::min(a.min() - b.max(), b.min() - a.max());
                        let max = <$ty>::max(a.max() - b.min(), b.max() - a.min());

                        TestResult::from_bool(
                            // Computed the correct number of resamples
                            distribution.as_slice().len() == nresamples &&
                            // No uninitialized values
                            distribution.as_slice().iter().all(|&x| {
                                (x > min || relative_eq!(x, min)) &&
                                (x < max || relative_eq!(x, max))
                            })
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }

            quickcheck!{
                fn two_sample(
                    a_size: usize, a_start: usize,
                    b_size: usize, b_start: usize,
                    nresamples: usize
                ) -> TestResult {
                    if let (Some(a), Some(b)) =
                        (::test::vec::<$ty>(a_size, a_start), ::test::vec::<$ty>(b_size, b_start))
                    {
                        let a = Sample::new(&a[a_start..]);
                        let b = Sample::new(&b[b_start..]);

                        let distribution = if nresamples > 0 {
                            univariate::bootstrap(a, b, nresamples, |a, b| (a.mean() - b.mean(),)).0
                        } else {
                            return TestResult::discard();
                        };

                        let min = <$ty>::min(a.min() - b.max(), b.min() - a.max());
                        let max = <$ty>::max(a.max() - b.min(), b.max() - a.min());

                        // Computed the correct number of resamples
                        let pass = distribution.as_slice().len() == nresamples &&
                            // No uninitialized values
                            distribution.as_slice().iter().all(|&x| {
                                (x > min || relative_eq!(x, min)) &&
                                (x < max || relative_eq!(x, max))
                            });

                        if !pass {
                            println!("A: {:?} (len={})", a.as_slice(), a.as_slice().len());
                            println!("B: {:?} (len={})", b.as_slice(), b.as_slice().len());
                            println!("Dist: {:?} (len={})", distribution.as_slice(), distribution.as_slice().len());
                            println!("Min: {}, Max: {}, nresamples: {}",
                                min, max, nresamples);
                        }

                        TestResult::from_bool(pass)
                    } else {
                        TestResult::discard()
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
