//! Helper traits for tupling/untupling

// TODO(HKT) make this more "generic", and move into its own crate
// TODO(negative_bounds) bounds should be `: !Drop` instead of `Copy`

use std::ptr;

use Distribution;

/// Any tuple: `(A, B, ..)`
pub trait Tuple: Sized {
    /// A tuple of distributions associated with this tuple
    type Distributions: TupledDistributions<Item=Self>;
}

/// A tuple of distributions: `(Distribution<A>, Distribution<B>, ..)`
pub trait TupledDistributions: Sized {
    /// A tuple that can be pushed/inserted into the tupled distributions
    type Item: Tuple<Distributions=Self>;

    /// Creates a tuple of unitialized distributions, where each distribution has the same `length`
    unsafe fn uninitialized(length: usize) -> Self;

    /// Writes each element of `tuple` into its corresponding distribution
    unsafe fn set_unchecked(&mut self, i: usize, tuple: Self::Item);
}

impl<A> Tuple for (A,) where A: Copy {
    type Distributions = (Distribution<A>,);
}

impl<A> TupledDistributions for (Distribution<A>,) where A: Copy {
    type Item = (A,);

    unsafe fn uninitialized(n: usize) -> (Distribution<A>,) {
        (
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
        )
    }

    unsafe fn set_unchecked(&mut self, i: usize, tuple: (A,)) {
        ptr::write((self.0).0.get_unchecked_mut(i), tuple.0);
    }
}

impl<A, B> Tuple for (A, B) where A: Copy, B: Copy {
    type Distributions = (Distribution<A>, Distribution<B>);
}

impl<A, B> TupledDistributions for (Distribution<A>, Distribution<B>) where A: Copy, B: Copy {
    type Item = (A, B);

    unsafe fn uninitialized(n: usize) -> (Distribution<A>, Distribution<B>) {
        (
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
        )
    }

    unsafe fn set_unchecked(&mut self, i: usize, tuple: (A, B)) {
        ptr::write((self.0).0.get_unchecked_mut(i), tuple.0);
        ptr::write((self.1).0.get_unchecked_mut(i), tuple.1);
    }
}

impl<A, B, C> Tuple for (A, B, C) where A: Copy, B: Copy, C: Copy {
    type Distributions = (Distribution<A>, Distribution<B>, Distribution<C>);
}

impl<A, B, C> TupledDistributions for (Distribution<A>, Distribution<B>, Distribution<C>) where
    A: Copy, B: Copy, C: Copy,
{
    type Item = (A, B, C);

    unsafe fn uninitialized(n: usize) -> (Distribution<A>, Distribution<B>, Distribution<C>) {
        (
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
        )
    }

    unsafe fn set_unchecked(&mut self, i: usize, tuple: (A, B, C)) {
        ptr::write((self.0).0.get_unchecked_mut(i), tuple.0);
        ptr::write((self.1).0.get_unchecked_mut(i), tuple.1);
        ptr::write((self.2).0.get_unchecked_mut(i), tuple.2);
    }
}

impl<A, B, C, D> Tuple for (A, B, C, D) where A: Copy, B: Copy, C: Copy, D: Copy {
    type Distributions = (Distribution<A>, Distribution<B>, Distribution<C>, Distribution<D>);
}

impl<A, B, C, D> TupledDistributions
for (Distribution<A>, Distribution<B>, Distribution<C>, Distribution<D>) where
    A: Copy, B: Copy, C: Copy, D: Copy,
{
    type Item = (A, B, C, D);

    unsafe fn uninitialized(
        n: usize,
    ) -> (Distribution<A>, Distribution<B>, Distribution<C>, Distribution<D>) {
        (
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
            Distribution({
                let mut v = Vec::with_capacity(n); v.set_len(n); v.into_boxed_slice()
            }),
        )
    }

    unsafe fn set_unchecked(&mut self, i: usize, tuple: (A, B, C, D)) {
        ptr::write((self.0).0.get_unchecked_mut(i), tuple.0);
        ptr::write((self.1).0.get_unchecked_mut(i), tuple.1);
        ptr::write((self.2).0.get_unchecked_mut(i), tuple.2);
        ptr::write((self.3).0.get_unchecked_mut(i), tuple.3);
    }
}
