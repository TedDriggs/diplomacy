use std::fmt;

use geo::Location;
use super::Command;


/// A command issued during the build/disband turn (typically "Winter").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildCommand {
    /// The recipient region is to spawn a new unit of the ordered type
    /// owned by the issuing nation. This will require that the province
    /// is a home supply center for the issuing nation and that the nation
    /// has sufficient centers to support the unit.
    Build,

    /// The recipient unit is to disband, ceasing to exist for the following turn.
    Disband,
}

/// A build command is never a move.
impl<L: Location> Command<L> for BuildCommand {
    fn move_dest<'a>(&'a self) -> Option<&'a L> {
        None
    }
}

impl fmt::Display for BuildCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match self {
                   &BuildCommand::Build => "build",
                   &BuildCommand::Disband => "disband",
               })
    }
}