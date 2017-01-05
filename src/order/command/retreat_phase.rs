use geo::Region;
use ShortName;

use std::fmt;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetreatCommand<'a> {
    Hold,
    Move(&'a Region<'a>)
}

impl<'a> fmt::Display for RetreatCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RetreatCommand::Hold => write!(f, "hold"),
            &RetreatCommand::Move(ref region) => write!(f, "-> {}", region.short_name()),
        }
    }
}