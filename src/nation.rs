use std::convert::From;
use std::fmt;
use serde::{Deserialize, Serialize};
use crate::ShortName;

/// An actor in the game. Nations can own units and issue orders.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nation(pub String);

impl ShortName for Nation {
    fn short_name(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for Nation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl<'a> From<&'a str> for Nation {
    fn from(s: &str) -> Self {
        Nation(String::from(s))
    }
}