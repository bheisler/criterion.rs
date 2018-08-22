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

#[derive(Debug, Fail)]
#[fail(display = "Failed to copy file {:?} to {:?}: {}", from, to, inner)]
pub struct CopyError {
    pub from: PathBuf,
    pub to: PathBuf,
    #[cause]
    pub inner: io::Error,
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub(crate) fn log_error(e: &Error) {
    error!("error: {}", e.as_fail());
    for cause in e.iter_chain() {
        error!("caused by: {}", cause);
    }

    debug!("backtrace: {}", e.backtrace());
}
