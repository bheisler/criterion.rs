pub fn zip3<A: IntoIterator, B: IntoIterator, C: IntoIterator>(
    a: A,
    b: B,
    c: C,
) -> impl Iterator<Item = (A::Item, B::Item, C::Item)> {
    a.into_iter().zip(b).zip(c).map(|((a, b), c)| (a, b, c))
}

pub fn zip4<A: IntoIterator, B: IntoIterator, C: IntoIterator, D: IntoIterator>(
    a: A,
    b: B,
    c: C,
    d: D,
) -> impl Iterator<Item = (A::Item, B::Item, C::Item, D::Item)> {
    a.into_iter()
        .zip(b)
        .zip(c)
        .zip(d)
        .map(|(((a, b), c), d)| (a, b, c, d))
}

pub fn zip5<A: IntoIterator, B: IntoIterator, C: IntoIterator, D: IntoIterator, E: IntoIterator>(
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
) -> impl Iterator<Item = (A::Item, B::Item, C::Item, D::Item, E::Item)> {
    a.into_iter()
        .zip(b)
        .zip(c)
        .zip(d)
        .zip(e)
        .map(|((((a, b), c), d), e)| (a, b, c, d, e))
}
