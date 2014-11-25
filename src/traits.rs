//! Traits

use std::slice;

/// Temporary trait until rust-lang/rust#19248 lands
pub trait AsStr for Sized? {
    /// Returns a `&str` view into `Self`
    fn as_str(&self) -> &str;
}

impl AsStr for str {
    fn as_str(&self) -> &str { self }
}

impl AsStr for String {
    fn as_str(&self) -> &str { self.as_slice() }
}

impl<'a, Sized? S> AsStr for &'a S where S: AsStr {
    fn as_str(&self) -> &str { AsStr::as_str(*self) }
}

// FIXME (AI) `P` should be an associated output type
/// Overloaded `configure` method
pub trait Configure<T, P> {
    /// Configure some set of properties
    fn configure<F: for<'a> FnOnce(&'a mut P) -> &'a mut P>(&mut self, T, F) -> &mut Self;
}

/// Types that can be plotted
pub trait Data {
    /// Convert the type into a double precision float
    fn f64(self) -> f64;
}

/// Temporary trait until `IntoIterator` lands in stdlib
// FIXME (AI) `T` and `I` should be an associated output types
pub trait IntoIterator<T, I> where I: Iterator<T> {
    /// Converts `Self` into an iterator
    fn into_iter(self) -> I;
}

impl<T, I> IntoIterator<T, I> for I where I: Iterator<T> {
    fn into_iter(self) -> I {
        self
    }
}

impl<'a, T> IntoIterator<&'a T, slice::Items<'a, T>> for &'a [T] {
    fn into_iter(self) -> slice::Items<'a, T> {
        self.iter()
    }
}

macro_rules! tuple {
    ($($N:expr),+,) => {$(
        impl<'a, T> IntoIterator<&'a T, slice::Items<'a, T>> for &'a [T, ..$N] {
            fn into_iter(self) -> slice::Items<'a, T> {
                self.iter()
            }
        })+
    }
}

tuple!{
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
}

// FIXME (AI) `P` should be an associated output type
/// Overloaded `plot` method
pub trait Plot<D, P> {
    /// Plots some `data` with some `configuration`
    fn plot<F>(&mut self, data: D, configure: F) -> &mut Self where
        F: for<'a> FnOnce(&'a mut P) -> &'a mut P;
}

/// Overloaded `set` method
pub trait Set<T> {
    /// Sets some property
    fn set(&mut self, T) -> &mut Self;
}
