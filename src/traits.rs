//! Traits

use std::slice;

/// Overloaded `configure` method
pub trait Configure<This> {
    type Properties;

    /// Configure some set of properties
    fn configure<F>(&mut self, This, F) -> &mut Self where
        F: FnOnce(&mut Self::Properties) -> &mut Self::Properties;
}

/// Types that can be plotted
pub trait Data {
    /// Convert the type into a double precision float
    fn f64(self) -> f64;
}

/// Temporary trait until `IntoIterator` lands in stdlib
pub trait IntoIterator {
    type Iter: Iterator;

    /// Converts `Self` into an iterator
    fn into_iter(self) -> Self::Iter;
}

impl<I> IntoIterator for I where I: Iterator {
    type Iter = I;

    fn into_iter(self) -> I {
        self
    }
}

impl<'a, T> IntoIterator for &'a [T] {
    type Iter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

macro_rules! tuple {
    ($($N:expr),+,) => {$(
        impl<'a, T> IntoIterator for &'a [T; $N] {
            type Iter = slice::Iter<'a, T>;

            fn into_iter(self) -> slice::Iter<'a, T> {
                self.iter()
            }
        })+
    }
}

tuple!{
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
}

/// Overloaded `plot` method
pub trait Plot<This> {
    type Properties;

    /// Plots some `data` with some `configuration`
    fn plot<F>(&mut self, This, F) -> &mut Self where
        F: FnOnce(&mut Self::Properties) -> &mut Self::Properties;
}

/// Overloaded `set` method
pub trait Set<T> {
    /// Sets some property
    fn set(&mut self, T) -> &mut Self;
}
