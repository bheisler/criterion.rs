#![allow(dead_code)]
use criterion::Criterion;
use rand::{thread_rng, Rand, Rng, XorShiftRng};

pub fn vec<T>() -> Vec<T>
where
    T: Rand,
{
    const SIZE: usize = 1_000_000;

    vec_sized(SIZE).unwrap()
}

pub fn vec_sized<T>(size: usize) -> Option<Vec<T>>
where
    T: Rand,
{
    let mut rng: XorShiftRng = thread_rng().gen();

    Some((0..size).map(|_| rng.gen()).collect())
}

pub fn reduced_samples() -> Criterion {
    Criterion::default().sample_size(20)
}
