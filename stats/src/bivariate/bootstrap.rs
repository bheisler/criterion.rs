#[cfg(test)]
macro_rules! test {
    ($ty:ident) => {
        mod $ty {
            use quickcheck::TestResult;

            use bivariate::Data;
            use bivariate::regression::Slope;

            quickcheck!{
                fn means(size: usize, start: usize,
                         offset: usize, nresamples: usize) -> TestResult {
                    if let Some(x) = ::test::vec::<$ty>(size, start) {
                        let y = ::test::vec::<$ty>(size + offset, start + offset).unwrap();
                        let data = Data::new(&x[start..], &y[start+offset..]);

                        let (x_means, y_means) = if nresamples > 0 {
                            data.bootstrap(nresamples, |d| (d.x().mean(), d.y().mean()))
                        } else {
                            return TestResult::discard();
                        };

                        let x_min = data.x().min();
                        let x_max = data.x().max();
                        let y_min = data.y().min();
                        let y_max = data.y().max();

                        TestResult::from_bool(
                            // Computed the correct number of resamples
                            x_means.as_slice().len() == nresamples &&
                            y_means.as_slice().len() == nresamples &&
                            // No uninitialized values
                            x_means.as_slice().iter().all(|&x| {
                                (x > x_min || relative_eq!(x, x_min)) &&
                                (x < x_max || relative_eq!(x, x_max))
                            }) &&
                            y_means.as_slice().iter().all(|&y| {
                                (y > y_min || relative_eq!(y, y_min)) &&
                                (y < y_max || relative_eq!(y, y_max))
                            })
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }

            quickcheck!{
                fn slope(size: usize, start: usize,
                         offset: usize, nresamples: usize) -> TestResult {
                    if let Some(x) = ::test::vec::<$ty>(size, start) {
                        let y = ::test::vec::<$ty>(size + offset, start + offset).unwrap();
                        let data = Data::new(&x[start..], &y[start+offset..]);

                        let slopes = if nresamples > 0 {
                            data.bootstrap(nresamples, |d| (Slope::fit(d),)).0
                        } else {
                            return TestResult::discard();
                        };

                        TestResult::from_bool(
                            // Computed the correct number of resamples
                            slopes.as_slice().len() == nresamples &&
                            // No uninitialized values
                            slopes.as_slice().iter().all(|s| s.0 > 0.)
                        )
                    } else {
                        TestResult::discard()
                    }
                }
            }

        }
    };
}

#[cfg(test)]
mod test {
    test!(f32);
    test!(f64);
}
