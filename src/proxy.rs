//! Generic constructors for newtypes

#![allow(non_snake_case)]

use std::borrow::Cow;
use std::path::Path;

/// Generic constructor for `Font`
#[inline(always)]
pub fn Font<S>(string: S) -> ::Font
    where S: Into<Cow<'static, str>>
{
    ::Font(string.into())
}

/// Generic constructor for `Label`
#[inline(always)]
pub fn Label<S>(string: S) -> ::Label
    where S: Into<Cow<'static, str>>
{
    ::Label(string.into())
}

/// Generic constructor for `Title`
#[inline(always)]
pub fn Title<S>(string: S) -> ::Title
    where S: Into<Cow<'static, str>>
{
    ::Title(string.into())
}

/// Generic constructor for `Output`
#[inline(always)]
pub fn Output<P>(path: P) -> ::Output
    where P: Into<Cow<'static, Path>>
{
    ::Output(path.into())
}
