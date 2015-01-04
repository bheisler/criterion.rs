//! Classification of outliers

use std::num;

use outliers::Label::{HighMild, HighSevere, LowMild, LowSevere, NotAnOutlier};
use {Simd, Stats};

// TODO Add more outlier classification methods

/// Classification of outliers using Tukey's fences
#[derive(RustcEncodable)]
#[experimental]
pub struct Outliers<A> {
    pub count: (uint, uint, uint, uint, uint),
    pub fences: (A, A, A, A),
    pub labels: Vec<Label>,
}

/// Labels used to classify outliers
#[derive(Copy, RustcEncodable, PartialEq)]
pub enum Label {
    HighMild,
    HighSevere,
    LowMild,
    LowSevere,
    NotAnOutlier,
}

impl Label {
    pub fn is_outlier(&self) -> bool {
        match *self {
            NotAnOutlier => false,
            _ => true,
        }
    }

    pub fn is_mild(&self) -> bool {
        match *self {
            LowMild | HighMild => true,
            _ => false,
        }
    }

    pub fn is_severe(&self) -> bool {
        match *self {
            LowSevere | HighSevere => true,
            _ => false,
        }
    }

    pub fn is_low(&self) -> bool {
        match *self {
            LowMild | LowSevere => true,
            _ => false,
        }
    }

    pub fn is_high(&self) -> bool {
        match *self {
            HighMild | HighSevere => true,
            _ => false,
        }
    }
}

impl<A> Outliers<A> where A: Simd {
    /// Returns the filtered sample, and the classified outliers
    pub fn classify(sample: &[A]) -> Outliers<A> {
        let (q1, _, q3) = sample.percentiles().quartiles();
        let iqr = q3 - q1;

        let k_h: A = num::cast(3f64).unwrap();
        let k_m: A = num::cast(1.5f64).unwrap();

        let (lost, lomt, himt, hist) =
            (q1 - k_h * iqr, q1 - k_m * iqr, q3 + k_m * iqr, q3 + k_h * iqr);

        let (mut los, mut lom, mut nao, mut him, mut his) = (0u, 0u, 0u, 0u, 0u);
        let labels = sample.iter().map(|&x| {
            if x < lost {
                los += 1;
                LowSevere
            } else if x > hist {
                his += 1;
                HighSevere
            } else if x < lomt {
                lom += 1;
                LowMild
            } else if x > himt {
                him += 1;
                HighMild
            } else {
                nao += 1;
                NotAnOutlier
            }
        }).collect();

        Outliers {
            count: (los, lom, nao, him, his),
            fences: (lost, lomt, himt, hist),
            labels: labels,
        }
    }
}
