use super::Command;
use crate::{geo::Location, ShortName};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            RetreatCommand::Move(ref region) => write!(f, "-> {}", region.short_name()),
        }
    }
}
