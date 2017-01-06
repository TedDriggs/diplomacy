use geo::Location;
use super::Command;

use std::fmt;

/// Valid orders during build seasons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildCommand {
    Build,
    Disband,
}

impl<L : Location> Command<L> for BuildCommand {
    
}

impl fmt::Display for BuildCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &BuildCommand::Build => "build",
            &BuildCommand::Disband => "disband"
        })
    }
}