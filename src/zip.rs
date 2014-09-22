macro_rules! min {
    ($x:expr) => { $x };
    ($x:expr, $($y:expr),+) => { ::std::cmp::min($x, min!($($y),+)) }
}

pub struct Zip3<A, B, C> {
    a: A,
    b: B,
    c: C,
}

impl<A, B, C> Zip3<A, B, C> {
    pub fn new(a: A, b: B, c: C) -> Zip3<A, B, C> {
        Zip3 {
            a: a,
            b: b,
            c: c,
        }
    }
}

impl<A, B, C, AI, BI, CI> Iterator<(A, B, C)>
for Zip3<AI, BI, CI>
where AI: Iterator<A>, BI: Iterator<B>, CI: Iterator<C>
{
    fn next(&mut self) -> Option<(A, B, C)> {
        match (self.a.next(), self.b.next(), self.c.next()) {
            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
            _ => None,
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (a, _) = self.a.size_hint();
        let (b, _) = self.b.size_hint();
        let (c, _) = self.c.size_hint();

        // NB Upper bound can be calculated, but only the lower bound is required in this library
        (min!(a, b, c), None)
    }
}

pub struct Zip4<A, B, C, D> {
    a: A,
    b: B,
    c: C,
    d: D,
}

impl<A, B, C, D> Zip4<A, B, C, D> {
    pub fn new(a: A, b: B, c: C, d: D) -> Zip4<A, B, C, D> {
        Zip4 {
            a: a,
            b: b,
            c: c,
            d: d,
        }
    }
}

impl<A, B, C, D, AI, BI, CI, DI> Iterator<(A, B, C, D)>
for Zip4<AI, BI, CI, DI>
where AI: Iterator<A>, BI: Iterator<B>, CI: Iterator<C>, DI: Iterator<D>
{
    fn next(&mut self) -> Option<(A, B, C, D)> {
        match (self.a.next(), self.b.next(), self.c.next(), self.d.next()) {
            (Some(a), Some(b), Some(c), Some(d)) => Some((a, b, c, d)),
            _ => None,
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        let (a, _) = self.a.size_hint();
        let (b, _) = self.b.size_hint();
        let (c, _) = self.c.size_hint();
        let (d, _) = self.d.size_hint();

        // NB Upper bound can be calculated, but only the lower bound is required in this library
        (min!(a, b, c, d), None)
    }
}
