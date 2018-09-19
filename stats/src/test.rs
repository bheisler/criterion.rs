use rand::{
    distributions::{Distribution, Standard},
    rngs::SmallRng,
    FromEntropy, Rng,
};

pub fn vec<T>(size: usize, start: usize) -> Option<Vec<T>>
where
    Standard: Distribution<T>,
{
    if size > start + 2 {
        let mut rng = SmallRng::from_entropy();

        Some((0..size).map(|_| rng.gen()).collect())
    } else {
        None
    }
}
