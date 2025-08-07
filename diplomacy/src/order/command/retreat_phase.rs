use super::Command;
use crate::{ShortName, geo::Location};
use std::fmt;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RetreatCommand<L> {
    Hold,
    Move(L),
}

impl<L: Location> Command<L> for RetreatCommand<L> {
    fn move_dest(&self) -> Option<&L> {
        match *self {
            RetreatCommand::Move(ref dst) => Some(dst),
            RetreatCommand::Hold => None,
        }
    }
}

impl<L: ShortName> fmt::Display for RetreatCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RetreatCommand::Hold => write!(f, "hold"),
            RetreatCommand::Move(region) => write!(f, "-> {}", region.short_name()),
        }
    }
}
