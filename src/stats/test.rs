use rand::distributions::{Distribution, Standard};
use rand::{self, FromEntropy, Rng};
use rand_xoshiro::Xoshiro256StarStar;

pub fn vec<T>(size: usize, start: usize) -> Option<Vec<T>>
where
    Standard: Distribution<T>,
{
    if size > start + 2 {
        let mut rng = Xoshiro256StarStar::from_entropy();

        Some((0..size).map(|_| rng.gen()).collect())
    } else {
        None
    }
}
