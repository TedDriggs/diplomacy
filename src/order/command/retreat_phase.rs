use geo::Location;
use super::Command;

use std::fmt;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetreatCommand<L : Location> {
    Hold,
    Move(L)
}

impl<L : Location> Command<L> for RetreatCommand<L> {
    fn move_dest<'a>(&'a self) -> Option<&'a L> {
        match *self {
            RetreatCommand::Move(ref dst) => Some(dst),
            RetreatCommand::Hold => None
        }
    }
}

impl<L : Location> fmt::Display for RetreatCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RetreatCommand::Hold => write!(f, "hold"),
            &RetreatCommand::Move(ref region) => write!(f, "-> {}", region.short_name()),
        }
    }
}

#[cfg(test)]
mod test {
    
}