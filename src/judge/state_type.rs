use geo::RegionKey;
use std::convert::From;

/// Struct representing the success or failure of an order.
/// The meaning of success and failure is contextually-dependent,
/// and should be derived from the outcome map of a resolution cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderState {
    /// For move commands, the order results in a move.
    /// For all other commands, the unit is not dislodged.
    Succeeds,
    
    Fails,
}

impl From<bool> for OrderState {
    fn from(b: bool) -> Self {
        match b {
            true => OrderState::Succeeds,
            false => OrderState::Fails,
        }
    }
}

impl From<OrderState> for bool {
    fn from(os: OrderState) -> Self {
        match os {
            OrderState::Succeeds => true,
            OrderState::Fails => false,
        }
    }
}

impl<'a> From<&'a ResolutionState> for OrderState {
    fn from(rs: &'a ResolutionState) -> Self {
        match *rs {
            ResolutionState::Guessing(os) |
            ResolutionState::Known(os) => os,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionState {
    Guessing(OrderState),
    Known(OrderState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OccupationOutcome {
    Holds,
    Moves,
    DislodgedBy(RegionKey),
}