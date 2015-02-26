//! Traits

use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Overloaded `configure` method
pub trait Configure<This> {
    /// The properties of what's being configured
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

/// Work-around until rust-lang/rust#22810 lands
pub trait IntoCowPath<'a> {
    /// Wrap `self` in a `Cow` pointer
    fn into_cow(self) -> Cow<'a, Path>;
}

impl<'a> IntoCowPath<'a> for &'a Path {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Borrowed(self)
    }
}

impl IntoCowPath<'static> for PathBuf {
    fn into_cow(self) -> Cow<'static, Path> {
        Cow::Owned(self)
    }
}

/// Overloaded `plot` method
pub trait Plot<This> {
    /// The properties associated to the plot
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
