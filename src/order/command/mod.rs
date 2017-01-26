use geo::Location;

use std::fmt;

mod build_phase;
mod main_phase;
mod retreat_phase;

pub use self::build_phase::BuildCommand;
pub use self::retreat_phase::RetreatCommand;
pub use self::main_phase::{MainCommand, SupportedOrder, ConvoyedMove};

/// A command issued to a unit or location which uses a single concrete location type of `L`.
pub trait Command<L: Location>: fmt::Display {
    /// Get the destination this order moves to, or `None` if the order is not a move.
    fn move_dest(&self) -> Option<&L>;

    /// Gets whether or not the order attempts to move to another region.
    fn is_move(&self) -> bool {
        self.move_dest().is_some()
    }

    fn is_move_to_province(&self, p: &L::Province) -> bool
    {
        self.move_dest().map(|dst| dst.province() == p).unwrap_or(false)
    }
}