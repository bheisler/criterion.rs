use std::io;
use std::path::PathBuf;

use failure::Error;

#[derive(Debug, Fail)]
#[fail(display = "Failed to access file {:?}: {}", path, inner)]
pub struct AccessError {
    pub path: PathBuf,
    #[cause]
    pub inner: io::Error,
}

pub type Result<T> = ::std::result::Result<T, Error>;
