//! Generic constructors for newtypes

#![allow(non_snake_case)]

use std::borrow::IntoCow;
use std::path::Path;

/// Generic constructor for `Font`
#[inline(always)]
pub fn Font<S>(string: S) -> ::Font where S: IntoCow<'static, str> {
    ::Font(string.into_cow())
}

/// Generic constructor for `Label`
#[inline(always)]
pub fn Label<S>(string: S) -> ::Label where S: IntoCow<'static, str> {
    ::Label(string.into_cow())
}

/// Generic constructor for `Title`
#[inline(always)]
pub fn Title<S>(string: S) -> ::Title where S: IntoCow<'static, str> {
    ::Title(string.into_cow())
}

/// Generic constructor for `Output`
#[inline(always)]
pub fn Output<P>(path: P) -> ::Output where P: IntoCow<'static, Path> {
    ::Output(path.into_cow())
}
