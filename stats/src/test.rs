use rand::{Rand, Rng, XorShiftRng, self};

pub fn vec<T>(size: usize, start: usize) -> Option<Vec<T>> where T: Rand {
    if size > start + 1 {
        let mut rng: XorShiftRng = rand::thread_rng().gen();

        Some((0..size).map(|_| rng.gen()).collect())
    } else {
        None
    }
}
