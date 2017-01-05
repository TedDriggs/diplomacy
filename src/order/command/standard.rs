use geo::Region;

/// A command that is issued to a unit at a location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command<'a> {
    /// The unit is to remain in place and do nothing else.
    Hold,
    
    /// The unit is to attempt to move from its current location to `Region`.
    Move(&'a Region<'a>),
    
    /// The unit is to remain in place and support another order.
    Support(SupportedOrder<'a>),
    
    /// The fleet is to attempt to help move an army to a specified region.
    Convoy(ConvoyedMove<'a>),
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

/// An army's move which a fleet should convoy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConvoyedMove<'a>(&'a Region<'a>, &'a Region<'a>);

impl<'a> ConvoyedMove<'a> {
    /// Create a new convoyed move
    pub fn new(from: &'a Region<'a>, to: &'a Region<'a>) -> Self {
        ConvoyedMove(from, to)
    }
}