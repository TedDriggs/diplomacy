use geo::Location;
use super::Command;

use std::fmt;

/// A command that is issued to a unit at a location during the main phase of a season.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainCommand<L : Location> {
    /// The unit is to remain in place and do nothing else.
    Hold,
    
    /// The unit is to attempt to move from its current location to `Location`.
    Move(L),
    
    /// The unit is to remain in place and support another order.
    Support(SupportedOrder<L>),
    
    /// The fleet is to attempt to help move an army to a specified locatio.
    Convoy(ConvoyedMove<L>),
}

impl<L : Location> Command<L> for MainCommand<L> {
    
}

impl<L : Location> fmt::Display for MainCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MainCommand::*;
        match self {
            &Hold => write!(f, "holds"),
            &Move(ref dest) => write!(f, "-> {}", dest.short_name()),
            &Support(ref order) => write!(f, "supports {}", order),
            &Convoy(ref mv) => write!(f, "convoys {}", mv)
        }
    }
}

/// An order supported by a support command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SupportedOrder<L : Location> {
    /// The supporting unit will attempt to keep the unit in `Region` in place.
    /// A "hold" support covers units that have hold, support, or convoy commands.
    Hold(L),
    
    /// The supporting unit will attempt to help the unit move from the first 
    /// region to the second.
    Move(L, L),
}

impl<L : Location> fmt::Display for SupportedOrder<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SupportedOrder::Hold(ref region) => write!(f, "{}", region.short_name()),
            &SupportedOrder::Move(ref fr, ref to) => write!(f, "{} -> {}", fr.short_name(), to.short_name())
        }
    }
}

/// An army's move which a fleet should convoy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConvoyedMove<L : Location>(L, L);

impl<L : Location> ConvoyedMove<L> {
    /// Create a new convoyed move
    pub fn new(from: L, to: L) -> Self {
        ConvoyedMove(from, to)
    }
}

impl<L : Location> fmt::Display for ConvoyedMove<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.0.short_name(), self.1.short_name())
    }
}