use std::{ffi::OsString, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DisplayHelp,
    DisplayVersion,
    LexOpt(lexopt::Error),
    UnexpectedArg(String),
    ConflictingFlags(&'static [&'static str]),
    MissingRequires(&'static str, &'static str),
    FlagMissingValue(&'static str),
    InvalidFlagValue(&'static str, OsString),
}

impl From<lexopt::Error> for Error {
    fn from(e: lexopt::Error) -> Self {
        Self::LexOpt(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplayHelp => f.write_str("Signals to display help"),
            Self::DisplayVersion => f.write_str("Signals to display version"),
            Self::LexOpt(_) => todo!(),
            Self::UnexpectedArg(args) => write!(f, "Extra args that weren't processed: {:?}", args),
            Self::ConflictingFlags(flags) => {
                write!(f, "Multiple of conflicting flags: {:?}", flags)
            }
            Self::MissingRequires(flag, requires) => {
                write!(f, "Flag '{}' missing requires '{}'", flag, requires)
            }
            Self::FlagMissingValue(flag) => write!(f, "Flag '{}' missing a value", flag),
            Self::InvalidFlagValue(flag, bad_val) => {
                write!(f, "Flag '{}' has invalid value: {:?}", flag, bad_val)
            }
        }
    }
}
