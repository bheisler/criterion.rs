use std::iter::AdditiveIterator;
use std::raw::{Repr, self};
use std::simd::{f32x4, f64x2};

use {Simd, Stats};

impl Simd for f32 {
    fn sum(sample: &[f32]) -> f32 {
        let raw::Slice { data, len } = sample.repr();

        if len < 8 {
            sample.iter().map(|&x| x).sum()
        } else {
            let data = data as *const f32x4;

            let mut sum = unsafe { *data };
            for i in range(1, (len / 4) as int) {
                sum += unsafe { *data.offset(i) };
            }

            let tail = sample.iter().rev().take(len % 4).map(|&x| x).sum();

            sum.0 + sum.1 + sum.2 + sum.3 + tail
        }
    }

    fn var(sample: &[f32], mean: Option<f32>) -> f32 {
        let raw::Slice { data, len } = sample.repr();

        assert!(len > 1);

        let mean = mean.unwrap_or_else(|| sample.mean());
        let squared_deviation = |&: &x: &f32| {
            let diff = x - mean;
            diff * diff
        };

        let sum = if len < 8 {
            sample.iter().map(squared_deviation).sum()
        } else {
            let data = data as *const f32x4;

            let mean4 = f32x4(mean, mean, mean, mean);
            let mut sum = f32x4(0., 0., 0., 0.);
            for i in range(0, (len / 4) as int) {
                let diff = unsafe { *data.offset(i) } - mean4;
                sum += diff * diff;
            }

            let tail = sample.iter().rev().take(len % 4).map(squared_deviation).sum();

            sum.0 + sum.1 + sum.2 + sum.3 + tail
        };

        sum / (len - 1) as f32
    }
}

impl Simd for f64 {
    fn sum(sample: &[f64]) -> f64 {
        let raw::Slice { data, len } = sample.repr();

        if len < 4 {
            sample.iter().map(|&x| x).sum()
        } else {
            let data = data as *const f64x2;

            let mut sum = unsafe { *data };
            for i in range(1, (len / 2) as int) {
                sum += unsafe { *data.offset(i) };
            }

            let tail = sample.iter().rev().take(len % 2).map(|&x| x).sum();

            sum.0 + sum.1 + tail
        }
    }

    fn var(sample: &[f64], mean: Option<f64>) -> f64 {
        let raw::Slice { data, len } = sample.repr();

        assert!(len > 1);

        let mean = mean.unwrap_or_else(|| sample.mean());
        let squared_deviation = |&: &x: &f64| {
            let diff = x - mean;
            diff * diff
        };

        let sum = if len < 4 {
            sample.iter().map(squared_deviation).sum()
        } else {
            let data = data as *const f64x2;

            let mean2 = f64x2(mean, mean);
            let mut sum = f64x2(0., 0.);
            for i in range(0, (len / 2) as int) {
                let diff = unsafe { *data.offset(i) } - mean2;
                sum += diff * diff;
            }

            let tail = sample.iter().rev().take(len % 2).map(squared_deviation).sum();

            sum.0 + sum.1 + tail
        };

        sum / (len - 1) as f64
    }
}
