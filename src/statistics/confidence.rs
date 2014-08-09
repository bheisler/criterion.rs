use super::Distribution;

#[deriving(Decodable, Encodable)]
pub struct ConfidenceInterval {
    confidence_level: f64,
    lower_bound: f64,
    upper_bound: f64,
}

impl ConfidenceInterval {
    // Compute the confidence interval using percentiles
    // TODO Try other methods like the BCA bootstrap
    pub fn new(distribution: &Distribution, cl: f64) -> ConfidenceInterval {
        ConfidenceInterval {
            confidence_level: cl,
            lower_bound: distribution.percentile(50.0 * (1.0 - cl)),
            upper_bound: distribution.percentile(50.0 * (1.0 + cl)),
        }
    }

    pub fn confidence_level(&self) -> f64 {
        self.confidence_level
    }

    pub fn lower_bound(&self) -> f64 {
        self.lower_bound
    }

    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }
}
