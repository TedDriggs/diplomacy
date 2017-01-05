use geo::Region;
use ShortName;

use std::fmt;

/// A command that is issued to a unit at a location during the main phase of a season.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MainCommand<'a> {
    /// The unit is to remain in place and do nothing else.
    Hold,
    
    /// The unit is to attempt to move from its current location to `Region`.
    Move(&'a Region<'a>),
    
    /// The unit is to remain in place and support another order.
    Support(SupportedOrder<'a>),
    
    /// The fleet is to attempt to help move an army to a specified region.
    Convoy(ConvoyedMove<'a>),
}

impl<'a> fmt::Display for MainCommand<'a> {
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
pub enum SupportedOrder<'a> {
    /// The supporting unit will attempt to keep the unit in `Region` in place.
    /// A "hold" support covers units that have hold, support, or convoy commands.
    Hold(&'a Region<'a>),
    
    /// The supporting unit will attempt to help the unit move from the first 
    /// region to the second.
    Move(&'a Region<'a>, &'a Region<'a>),
}

impl<'a> fmt::Display for SupportedOrder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &SupportedOrder::Hold(ref region) => write!(f, "{}", region.short_name()),
            &SupportedOrder::Move(ref fr, ref to) => write!(f, "{} -> {}", fr.short_name(), to.short_name())
        }
    }
}

/// An army's move which a fleet should convoy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConvoyedMove<'a>(&'a Region<'a>, &'a Region<'a>);

impl<'a> ConvoyedMove<'a> {
    /// Create a new convoyed move
    pub fn new(from: &'a Region<'a>, to: &'a Region<'a>) -> Self {
        ConvoyedMove(from, to)
    }
}

impl<'a> fmt::Display for ConvoyedMove<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.0.short_name(), self.1.short_name())
    }
}