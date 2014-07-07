use self::resamples::Resamples;

mod resamples;

pub fn compare(a: &[f64],
               b: &[f64],
               nresamples_sqrt: uint,
               comparators: &[fn (&[f64], &[f64]) -> f64])
               -> Vec<Vec<f64>> {
    let mut a_resamples = Resamples::new(a);
    let mut b_resamples = Resamples::new(b);

    let nresamples = nresamples_sqrt * nresamples_sqrt;
    let mut distributions = range(0, comparators.len()).map(|_| {
        Vec::with_capacity(nresamples)
    }).collect::<Vec<Vec<f64>>>();

    for _ in range(0, nresamples_sqrt) {
        let a_resample = a_resamples.next();

        for _ in range(0, nresamples_sqrt) {
            let b_resample = b_resamples.next();

            for (distribution, comparator) in {
                distributions.mut_iter().zip(comparators.iter())
            } {
                distribution.push((*comparator)(a_resample, b_resample));
            }
        }
    }

    distributions
}

pub fn estimate(sample: &[f64],
                nresamples: uint,
                estimators: &[fn (&[f64]) -> f64])
                -> Vec<Vec<f64>> {
    let mut resamples = Resamples::new(sample);

    let mut distributions = range(0, estimators.len()).map(|_| {
        Vec::with_capacity(nresamples)
    }).collect::<Vec<Vec<f64>>>();

    for _ in range(0, nresamples) {
        let resample = resamples.next();

        for (distribution, estimator) in {
            distributions.mut_iter().zip(estimators.iter())
        } {
            distribution.push((*estimator)(resample));
        }
    }

    distributions
}
