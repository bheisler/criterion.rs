use std::io::{File,IoError};
use serialize::{Encodable,json};

use super::{
    ConfidenceInterval,
    Distribution,
    Distributions,
    Estimate,
    Estimates,
    Mean,
    Median,
    MedianAbsDev,
    Statistic,
    StdDev,
};
use super::resampler::Resampler;

// FIXME RFC#11: Enforce bound on the `struct` rather than on the `impl`s
//struct Sample<V: Slice<f64>>(V);
pub struct Sample<V>(V);

impl<V> Sample<V> {
    pub fn new(data: V) -> Sample<V> {
        Sample(data)
    }

    pub fn load(path: &Path) -> Sample<Vec<f64>> {
        let path_ = path.display();

        match File::open(path).read_to_string() {
            Err(e) => fail!("`open {}`: {}", path_, e),
            Ok(s) => match json::decode(s.as_slice()) {
                Err(e) => fail!("`decode {}`: {}", path_, e),
                Ok(v) => Sample(v),
            }
        }
    }
}

impl<'a, V: Encodable<json::Encoder<'a>, IoError>> Sample<V> {
    pub fn save(&self, path: &Path) {
        let &Sample(ref sample) = self;
        // TODO JSON should be pretty encoded (I wish we had `json::pretty_encode`)
        match File::create(path).write_str(json::encode(sample).as_slice()) {
            Err(e) => fail!("`write {}`: {}", path.display(), e),
            Ok(_) => {},
        }
    }
}

impl<V: Slice<f64>> Sample<V> {
    fn join<W: Slice<f64>>(&self, other: &Sample<W>) -> Sample<Vec<f64>> {
        let mut v = Vec::with_capacity(self.len() + other.len());
        v.push_all(self.as_slice());
        v.push_all(other.as_slice());

        Sample(v)
    }

    fn split_at(&self, n: uint) -> (Sample<&[f64]>, Sample<&[f64]>) {
        assert!(n < self.len());

        let sample = self.as_slice();

        (Sample(sample.slice_to(n)), Sample(sample.slice_from(n)))
    }

    // Returns the relative difference between the statistic of two samples
    fn compare<W: Slice<f64>>(
               &self,
               other: &Sample<W>,
               statistic: Statistic) -> f64 {
        self.compute(statistic) / other.compute(statistic) - 1.0
    }

    pub fn compute(&self, statistic: Statistic) -> f64 {
        use test::stats::Stats;

        let sample = self.as_slice();

        match statistic {
            Mean => sample.mean(),
            Median => sample.median(),
            MedianAbsDev => sample.median_abs_dev(),
            StdDev => sample.std_dev(),
        }
    }

    pub fn t_test<W: Slice<f64>>(&self, other: &Sample<W>) -> f64 {
        let (mu_1, mu_2) = (self.compute(Mean), other.compute(Mean));
        let (sigma_1, sigma_2) = (self.compute(StdDev).powi(2), other.compute(StdDev).powi(2));
        let (n_1, n_2) = (self.len() as f64, other.len() as f64);

        (mu_1 - mu_2) / (sigma_1 / n_1 + sigma_2 / n_2).sqrt()
    }

    // Bootstrap the statistics of the sample using "case resampling"
    // XXX Try other methods, like the smooth bootstrap
    pub fn bootstrap(&self,
                     statistics: &[Statistic],
                     nresamples: uint,
                     cl: f64)
                     -> (Estimates, Distributions) {
        assert!(cl > 0.0 && cl < 1.0);

        let mut resampler = Resampler::new(self);

        let mut distributions: Vec<Vec<f64>> =
            Vec::from_elem(statistics.len(), Vec::with_capacity(nresamples));
        for _ in range(0, nresamples) {
            let resample = resampler.next();

            for (distribution, &statistic) in distributions.mut_iter().zip(statistics.iter()) {
                distribution.push(resample.compute(statistic))
            }
        }

        let distributions: Vec<Distribution> =
            distributions.move_iter().map(|v| Distribution::new(v)).collect();

        let estimates =
            distributions.iter().zip(statistics.iter()).map(|(distribution, &statistic)| {
                Estimate::new(
                    ConfidenceInterval::new(distribution, cl),
                    self.compute(statistic),
                    distribution.standard_error(),
                )
            }).collect::<Vec<Estimate>>();

        (Estimates::new(statistics, estimates), Distributions::new(statistics, distributions))
    }

    // FIXME DRY: This method is *very* similar to `bootstrap`
    pub fn bootstrap_compare<W: Slice<f64>>(
                             &self,
                             other: &Sample<W>,
                             statistics: &[Statistic],
                             nresamples_sqrt: uint,
                             cl: f64)
                             -> (Estimates, Distributions) {
        assert!(cl > 0.0 && cl < 1.0);

        let mut resampler = Resampler::new(self);
        let mut other_resampler = Resampler::new(other);

        let nresamples = nresamples_sqrt * nresamples_sqrt;
        let mut distributions: Vec<Vec<f64>> =
            Vec::from_elem(statistics.len(), Vec::with_capacity(nresamples));
        for _ in range(0, nresamples_sqrt) {
            let resample = resampler.next();

            for _ in range(0, nresamples_sqrt) {
                let other_resample = other_resampler.next();

                for (distribution, statistic) in distributions.mut_iter().zip(statistics.iter()) {
                    distribution.push(resample.compare(&other_resample, *statistic))
                }
            }
        }

        let distributions: Vec<Distribution> =
            distributions.move_iter().map(|v| Distribution::new(v)).collect();

        let estimates =
            distributions.iter().zip(statistics.iter()).map(|(distribution, &statistic)| {
                Estimate::new(
                    ConfidenceInterval::new(distribution, cl),
                    self.compare(other, statistic),
                    distribution.standard_error(),
                )
            }).collect::<Vec<Estimate>>();

        (Estimates::new(statistics, estimates), Distributions::new(statistics, distributions))
    }

    // FIXME DRY: This method is *very* similar to `bootstrap`
    pub fn bootstrap_t_test<W: Slice<f64>>(
                            &self,
                            other: &Sample<W>,
                            nresamples: uint,
                            cl: f64)
                            -> Distribution {
        assert!(cl > 0.0 && cl < 1.0);

        let n = self.len();
        let joint_sample = self.join(other);
        let mut resampler = Resampler::new(&joint_sample);

        let mut distribution = Vec::with_capacity(nresamples);

        for _ in range(0, nresamples) {
            let joint_resample = resampler.next();
            let (resample, other_resample) = joint_resample.split_at(n);

            distribution.push(resample.t_test(&other_resample));
        }

        Distribution::new(distribution)
    }
}

impl<V: Slice<f64>> Slice<f64> for Sample<V> {
    fn as_slice(&self) -> &[f64] {
        let &Sample(ref sample) = self;
        sample.as_slice()
    }
}

impl<V: Slice<f64>> Collection for Sample<V> {
    fn len(&self) -> uint {
        let &Sample(ref sample) = self;
        sample.as_slice().len()
    }
}
