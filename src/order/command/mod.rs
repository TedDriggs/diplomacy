use geo::Location;

use std::fmt;

mod build_phase;
mod main_phase;
mod retreat_phase;

pub use self::build_phase::BuildCommand;
pub use self::retreat_phase::RetreatCommand;
pub use self::main_phase::{MainCommand, SupportedOrder, ConvoyedMove};

/// Marker trait for a command which uses a single concrete location type of `L`.
pub trait Command<L : Location> : fmt::Display {
    
}