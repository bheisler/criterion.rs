//! Classification of outliers

use std::num;
use test::stats::Stats;

// TODO Add more outlier classification methods

/// Outliers classified by Tukey's fences
#[deriving(Encodable)]
#[experimental]
pub struct Outliers<A> {
    pub high_mild: Vec<A>,
    pub high_severe: Vec<A>,
    pub low_mild: Vec<A>,
    pub low_severe: Vec<A>,
    /// Threshold (fences) used to classify
    pub thresholds: (A, A, A, A),
}

impl<A: FloatMath + FromPrimitive> Outliers<A> {
    /// Returns the filtered sample, and the classified outliers
    pub fn classify(sample: &[A]) -> (Vec<A>, Outliers<A>) {
        let (q1, _, q3) = sample.quartiles();
        let iqr = q3 - q1;

        let k_h: A = num::cast(3f64).unwrap();
        let k_m: A = num::cast(1.5f64).unwrap();

        let (lost, lomt, himt, hist) =
            (q1 - k_h * iqr, q1 - k_m * iqr, q3 + k_m * iqr, q3 + k_h * iqr);

        let (mut his, mut him, mut lom, mut los, mut normal) =
            (vec![], vec![], vec![], vec![], Vec::with_capacity(sample.len()));

        for &x in sample.iter() {
            if x < lost {
                los.push(x);
            } else if x > hist {
                his.push(x);
            } else if x < lomt {
                lom.push(x);
            } else if x > himt {
                him.push(x);
            } else {
                normal.push(x);
            }

        }

        (normal, Outliers {
            high_mild: him,
            high_severe: his,
            low_mild: lom,
            low_severe: los,
            thresholds: (lost, lomt, himt, hist),
        })
    }
}

impl<A> Collection for Outliers<A> {
    /// The total number of outliers
    fn len(&self) -> uint {
        self.low_severe.len() + self.low_mild.len() + self.high_mild.len() + self.high_severe.len()
    }
}
