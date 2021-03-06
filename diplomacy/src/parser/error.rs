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
    pub fn new(kind: ErrorKind, input: impl Into<String>) -> Self {
        Error {
            kind,
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
#[non_exhaustive]
pub enum ErrorKind {
    InvalidUnitType,
    UnknownCommand,
    BadCoast,
    /// The order was recognized as a move, but the destination could not be parsed.
    /// Move commands must use the one-word destination code and may include "via convoy".
    MalformedMove,
    MalformedRegion,
    MalformedSupport,
    MalformedConvoy,
    TooFewWords(usize),
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
            ErrorKind::MalformedMove => write!(f, "Malformed move command"),
            ErrorKind::TooFewWords(min) => write!(f, "Too few words, expected {}", min),
        }
    }
}
