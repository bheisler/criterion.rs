pub fn chunk_by<'c, T, Key: 'c + Copy + Eq>(
    c: &'c [T],
    by: impl Copy + for<'a> Fn(&'a T) -> Key,
) -> impl Iterator<Item = (Key, &'c [T])> {
    c.chunk_by(move |a, b| {
        let a_key = by(a);
        let b_key = by(b);
        a_key == b_key
    })
    .map(move |chunk| {
        let key = by(&chunk[0]); // Chunks are never empty.
        (key, chunk)
    })
}
