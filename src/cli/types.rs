use std::{convert::Infallible, fmt, str::FromStr};

#[derive(Debug)]
pub struct TypeParseError(String);

impl fmt::Display for TypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unexpected value: {}", self.0)
    }
}

impl std::error::Error for TypeParseError {}

const DEFAULT_SAVE_BASELINE: &str = "base";

#[derive(Debug, PartialEq)]
pub struct SaveBaseline(pub String);

impl SaveBaseline {
    pub fn is_default(&self) -> bool {
        self.0 == DEFAULT_SAVE_BASELINE
    }
}

impl Default for SaveBaseline {
    fn default() -> Self {
        Self(String::from(DEFAULT_SAVE_BASELINE))
    }
}

impl FromStr for SaveBaseline {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from(s)))
    }
}

#[derive(Debug, PartialEq)]
pub enum Color {
    Auto,
    Always,
    Never,
}

impl Default for Color {
    fn default() -> Self {
        Self::Auto
    }
}

impl FromStr for Color {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            invalid => Err(TypeParseError(invalid.to_owned())),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlottingBackend {
    GnuPlot,
    Plotters,
}

impl From<PlottingBackend> for crate::PlottingBackend {
    fn from(cli_backend: PlottingBackend) -> Self {
        match cli_backend {
            PlottingBackend::GnuPlot => Self::Gnuplot,
            PlottingBackend::Plotters => Self::Plotters,
        }
    }
}

impl FromStr for PlottingBackend {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gnuplot" => Ok(Self::GnuPlot),
            "plotters" => Ok(Self::Plotters),
            invalid => Err(TypeParseError(invalid.to_owned())),
        }
    }
}

impl fmt::Display for PlottingBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::GnuPlot => "gnuplot",
            Self::Plotters => "plotters",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputFormat {
    Criterion,
    Bencher,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Criterion
    }
}

impl FromStr for OutputFormat {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "criterion" => Ok(Self::Criterion),
            "bencher" => Ok(Self::Bencher),
            invalid => Err(TypeParseError(invalid.to_owned())),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Criterion => "criterion",
            Self::Bencher => "bencher",
        })
    }
}
