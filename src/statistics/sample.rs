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
//struct Sample<V: Vector<f64>>(V);
pub struct Sample<V>(V);

impl<V> Sample<V> {
    pub fn new(data: V) -> Sample<V> {
        Sample(data)
    }

    pub fn load(path: &Path) -> Sample<Vec<f64>> {
        match File::open(path).read_to_string() {
            Err(e) => fail!("{}", e),
            Ok(s) => match json::decode(s.as_slice()) {
                Err(e) => fail!("{}", e),
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
            Err(e) => fail!("{}", e),
            Ok(_) => {},
        }
    }
}

impl<V: Vector<f64>> Sample<V> {
    // Returns the relative difference between the statistic of two samples
    fn compare<W: Vector<f64>>(
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

    // Bootstrap the statistics of the sample using "case resampling"
    // XXX Try other methods, like the smooth bootstrap
    pub fn bootstrap(&self,
                     statistics: &[Statistic],
                     nresamples: uint,
                     _confidence_level@cl: f64)
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
        //            ^~~ FIXME Better inference engine: Replace with ::<Vec<_>>

        (Estimates::new(statistics, estimates), Distributions::new(statistics, distributions))
    }

    // FIXME DRY: This method is *very* similar to `bootstrap`
    pub fn bootstrap_compare<W: Vector<f64>>(
                             &self,
                             other: &Sample<W>,
                             statistics: &[Statistic],
                             nresamples_sqrt: uint,
                             _confidence_level@cl: f64)
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
        //            ^~~ FIXME Better inference engine: Replace with ::<Vec<_>>

        (Estimates::new(statistics, estimates), Distributions::new(statistics, distributions))
    }
}

impl<V: Vector<f64>> Vector<f64> for Sample<V> {
    fn as_slice<'a>(&'a self) -> &'a [f64] {
        let &Sample(ref sample) = self;
        sample.as_slice()
    }
}
