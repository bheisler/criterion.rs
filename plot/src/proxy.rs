//! Generic constructors for newtypes

#![allow(non_snake_case)]

use std::borrow::Cow;
use std::path::Path;

/// Generic constructor for `Font`
#[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
#[inline(always)]
pub fn Font<S>(string: S) -> ::Font
where
    S: Into<Cow<'static, str>>,
{
    ::Font(string.into())
}

/// Generic constructor for `Label`
#[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
#[inline(always)]
pub fn Label<S>(string: S) -> ::Label
where
    S: Into<Cow<'static, str>>,
{
    ::Label(string.into())
}

/// Generic constructor for `Title`
#[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
#[inline(always)]
pub fn Title<S>(string: S) -> ::Title
where
    S: Into<Cow<'static, str>>,
{
    ::Title(string.into())
}

/// Generic constructor for `Output`
#[cfg_attr(feature = "cargo-clippy", allow(inline_always))]
#[inline(always)]
pub fn Output<P>(path: P) -> ::Output
where
    P: Into<Cow<'static, Path>>,
{
    ::Output(path.into())
}
