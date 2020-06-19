use crate::ShortName;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::From;
use std::fmt;

/// An actor in the game. Nations can own units and issue orders.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nation(pub String);

impl ShortName for Nation {
    fn short_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.0)
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
