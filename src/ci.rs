/// Estimate interval of a population parameter
#[deriving(Decodable, Encodable, PartialEq)]
#[experimental]
pub struct ConfidenceInterval<A> {
    /// The confidence level used to find the confidence interval
    pub confidence_level: A,
    /// The lower bound of the confidence interval
    pub lower_bound: A,
    /// The upper bound of the confidence interval
    pub upper_bound: A,
}
