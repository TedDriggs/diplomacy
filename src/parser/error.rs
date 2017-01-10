use std::error as err;
use std::fmt;

/// The error type for order parsing.
#[derive(Debug, Clone, Default)]
pub struct Error {
    
}

impl err::Error for Error {
    fn description(&self) -> &str {
        "Parsing error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parsing error")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind<'a> {
    InvalidUnitType(&'a str),
    UnknownCommand(&'a str),
}

impl<'a> fmt::Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::InvalidUnitType(ref ut) => write!(f, "Invalid unit type: '{}'", ut),
            ErrorKind::UnknownCommand(ref cmd) => write!(f, "Unknown command: {}", cmd),
        }
    }
}