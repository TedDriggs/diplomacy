use super::Command;
use crate::geo::Location;
use crate::order::Order;
use crate::ShortName;
use crate::UnitType;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;

pub type MainOrder<L> = Order<L, MainCommand<L>>;

/// A command that is issued to a unit at a location during the main phase of a season.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MainCommand<L> {
    /// The unit is to remain in place and do nothing else.
    Hold,

    /// The unit is to attempt to move from its current location to `Location`.
    Move(L),

    /// The unit is to remain in place and support another order.
    Support(SupportedOrder<L>),

    /// The fleet is to attempt to help move an army to a specified locatio.
    Convoy(ConvoyedMove<L>),
}

impl<L: Location> Command<L> for MainCommand<L> {
    fn move_dest(&self) -> Option<&L> {
        match *self {
            MainCommand::Move(ref dst) => Some(dst),
            _ => None,
        }
    }

    fn is_move(&self) -> bool {
        match *self {
            MainCommand::Move(..) => true,
            _ => false,
        }
    }
}

impl<L: Location> fmt::Display for MainCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MainCommand::*;
        match self {
            Hold => write!(f, "holds"),
            Move(ref dest) => write!(f, "-> {}", dest.short_name()),
            Support(ref order) => write!(f, "supports {}", order),
            Convoy(ref mv) => write!(f, "convoys {}", mv),
        }
    }
}

/// Declaration of the order to be supported by a support command.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedOrder<L> {
    /// The supporting unit will attempt to keep the unit in `Region` in place.
    /// A "hold" support covers units that have hold, support, or convoy commands.
    Hold(UnitType, L),

    /// The supporting unit will attempt to help the unit move from the first
    /// region to the second.
    Move(UnitType, L, L),
}

impl<L: Location> SupportedOrder<L> {
    pub fn is_legal(&self) -> bool {
        match *self {
            SupportedOrder::Hold(..) => true,
            SupportedOrder::Move(_, ref fr, ref to) => fr != to,
        }
    }
}

impl<L: ShortName> fmt::Display for SupportedOrder<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SupportedOrder::Hold(ref ut, ref region) => {
                write!(f, "{} {}", ut.short_name(), region.short_name())
            }
            SupportedOrder::Move(ref ut, ref fr, ref to) => write!(
                f,
                "{} {} -> {}",
                ut.short_name(),
                fr.short_name(),
                to.short_name()
            ),
        }
    }
}

impl<L> From<SupportedOrder<L>> for MainCommand<L> {
    fn from(support: SupportedOrder<L>) -> Self {
        MainCommand::Support(support)
    }
}

impl<L: Location> PartialEq<Order<L, MainCommand<L>>> for SupportedOrder<L> {
    fn eq(&self, other: &Order<L, MainCommand<L>>) -> bool {
        match self {
            SupportedOrder::Hold(ref ut, ref loc) => {
                !other.command.is_move() && loc == &other.region && ut == &other.unit_type
            }
            SupportedOrder::Move(ref ut, ref fr, ref to) => {
                if let MainCommand::Move(ref dst) = other.command {
                    ut == &other.unit_type && fr == &other.region && to == dst
                } else {
                    false
                }
            }
        }
    }
}

/// An army's move which a fleet should convoy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConvoyedMove<L>(L, L);

impl<L> ConvoyedMove<L> {
    /// Create a new convoyed move
    pub fn new(from: L, to: L) -> Self {
        ConvoyedMove(from, to)
    }

    pub fn from(&self) -> &L {
        &self.0
    }

    pub fn to(&self) -> &L {
        &self.1
    }
}

impl<L: ShortName> fmt::Display for ConvoyedMove<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A {} -> {}", self.0.short_name(), self.1.short_name())
    }
}

impl<L: Location> PartialEq<MainOrder<L>> for ConvoyedMove<L> {
    fn eq(&self, rhs: &MainOrder<L>) -> bool {
        if rhs.unit_type == UnitType::Army {
            match &rhs.command {
                MainCommand::Move(ref dst) => self.from() == &rhs.region && self.to() == dst,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl<L> From<ConvoyedMove<L>> for MainCommand<L> {
    fn from(cm: ConvoyedMove<L>) -> Self {
        MainCommand::Convoy(cm)
    }
}
