#![allow(dead_code)]
use criterion::Criterion;
use rand::{
    distributions::{Distribution, Standard},
    rngs::SmallRng,
    FromEntropy, Rng,
};

pub fn vec<T>() -> Vec<T>
where
    Standard: Distribution<T>,
{
    const SIZE: usize = 1_000_000;

    vec_sized(SIZE).unwrap()
}

pub fn vec_sized<T>(size: usize) -> Option<Vec<T>>
where
    Standard: Distribution<T>,
{
    let mut rng = SmallRng::from_entropy();

    Some((0..size).map(|_| rng.gen()).collect())
}

pub fn reduced_samples() -> Criterion {
    Criterion::default().sample_size(20)
}
