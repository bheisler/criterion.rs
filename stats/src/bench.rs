use rand::Rand;

pub fn vec<T>() -> Vec<T> where T: Rand {
    const SIZE: usize = 1_000_000;

    ::test::vec(SIZE, 0).unwrap()
}
