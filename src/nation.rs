use ShortName;

use std::fmt;

/// An actor in the game. Nations can own units and issue orders.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nation;

impl ShortName for Nation {
    fn short_name(&self) -> String {
        "".to_string()
    }
}

impl fmt::Display for Nation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}