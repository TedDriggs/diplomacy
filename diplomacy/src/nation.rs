use crate::ShortName;
use std::borrow::Cow;
use std::fmt;

/// An actor in the game. Nations can own units and issue orders.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nation(String);

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

impl From<&str> for Nation {
    fn from(s: &str) -> Self {
        Nation(String::from(s))
    }
}
