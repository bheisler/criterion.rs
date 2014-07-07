use test::stats::Stats;

use units::{AsPercent,AsSignedPercent,AsTime};

#[deriving(Encodable)]
pub struct Estimate {
    confidence_level: f64,
    lower_bound: f64,
    point: f64,
    standard_error: f64,
    upper_bound: f64,
}

impl Estimate {
    // XXX Naive estimate using percentiles, try the BCA boostrap
    pub fn new(point_estimate: f64,
           distribution: &[f64],
           cl: f64)
            -> Estimate {
        assert!(cl > 0.0 && cl < 1.0);

        let standard_error = distribution.std_dev();
        let lower_bound = distribution.percentile(50.0 * (1.0 - cl));
        let upper_bound = distribution.percentile(50.0 * (1.0 + cl));

        Estimate {
            confidence_level: cl,
            lower_bound: lower_bound,
            point: point_estimate,
            standard_error: standard_error,
            upper_bound: upper_bound,
        }
    }
}

impl AsPercent for Estimate {
    fn as_percent(&self) -> String {
        format!("{} ± {} [{} {}] {}% CI",
                self.point.as_signed_percent(),
                self.standard_error.as_percent(),
                self.lower_bound.as_signed_percent(),
                self.upper_bound.as_signed_percent(),
                self.confidence_level * 100.0)
    }
}

impl AsTime for Estimate {
    fn as_time(&self) -> String {
        format!("{} ± {} [{} {}] {}% CI",
                self.point.as_time(),
                self.standard_error.as_time(),
                self.lower_bound.as_time(),
                self.upper_bound.as_time(),
                self.confidence_level * 100.0)
    }
}
