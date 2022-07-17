use std::{ffi::OsString, fmt};

#[derive(Debug)]
pub enum Error {
    DisplayHelp,
    DisplayVersion,
    PicoArgs(pico_args::Error),
    TrailingArgs(Vec<OsString>),
    InvalidColor(String),
    InvalidPlottingBackend(String),
    InvalidOutputFormat(String),
    ConflictingFlags(&'static [&'static str]),
    MissingRequires(&'static str, &'static str),
}

impl From<pico_args::Error> for Error {
    fn from(e: pico_args::Error) -> Self {
        Self::PicoArgs(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplayHelp => f.write_str("Signals to display help"),
            Self::DisplayVersion => f.write_str("Signals to display version"),
            Self::PicoArgs(err) => write!(f, "Arg-parse error: {}", err),
            Self::TrailingArgs(args) => write!(f, "Extra args that weren't processed: {:?}", args),
            Self::InvalidColor(s) => write!(f, "Invalid color: {}", s),
            Self::InvalidPlottingBackend(s) => write!(f, "Invalid plotting backend: {}", s),
            Self::InvalidOutputFormat(s) => write!(f, "Invalid output format: {}", s),
            Self::ConflictingFlags(flags) => {
                write!(f, "Multiple of conflicting flags: {:?}", flags)
            }
            Self::MissingRequires(flag, requires) => {
                write!(f, "Flag '{}' missing requires '{}'", flag, requires)
            }
        }
    }
}
