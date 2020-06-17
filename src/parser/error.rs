use std::error as err;
use std::fmt;

/// The error type for order parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    input: String,
}

impl Error {
    /// Creates a new error of the given `kind` for the relevant input string.
    /// `input` should be the smallest part of the string where the error was
    /// found, not the entire order.
    pub fn new<IS: Into<String>>(kind: ErrorKind, input: IS) -> Self {
        Error {
            kind: kind,
            input: input.into(),
        }
    }

    /// Gets the kind of error observed.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl err::Error for Error {
    fn description(&self) -> &str {
        "Parsing error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: '{}'", self.kind, self.input)
    }
}

/// Different kinds of parsing error; this is not meant to be exhaustive.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    InvalidUnitType,
    UnknownCommand,
    BadCoast,
    MalformedRegion,
    MalformedSupport,
    MalformedConvoy,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::InvalidUnitType => write!(f, "Invalid unit type"),
            ErrorKind::UnknownCommand => write!(f, "Unknown command"),
            ErrorKind::BadCoast => write!(f, "Bad coast"),
            ErrorKind::MalformedRegion => write!(f, "Malformed region key"),
            ErrorKind::MalformedSupport => write!(f, "Malformed support command"),
            ErrorKind::MalformedConvoy => write!(f, "Malformed convoy command"),
        }
    }
}
