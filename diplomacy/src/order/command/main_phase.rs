use super::Command;
use crate::geo::Location;
use crate::order::Order;
use crate::ShortName;
use crate::UnitType;
use std::cmp::PartialEq;
use std::fmt;

pub type MainOrder<L> = Order<L, MainCommand<L>>;

/// A command that is issued to a unit at a location during the main phase of a season.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MainCommand<L> {
    /// The unit is to remain in place and do nothing else.
    Hold,

    /// The unit is to attempt to move from its current location to another location.
    Move(MoveCommand<L>),

    /// The unit is to remain in place and support another order.
    Support(SupportedOrder<L>),

    /// The unit is to remain in place and attempt to help move an army to a specified location.
    Convoy(ConvoyedMove<L>),
}

impl<L: Location> Command<L> for MainCommand<L> {
    fn move_dest(&self) -> Option<&L> {
        match *self {
            MainCommand::Move(ref cmd) => cmd.move_dest(),
            _ => None,
        }
    }

    fn is_move(&self) -> bool {
        matches!(*self, MainCommand::Move(_))
    }
}

impl<L: Location> fmt::Display for MainCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MainCommand::*;
        match self {
            Hold => write!(f, "holds"),
            Move(ref cmd) => write!(f, "-> {}", cmd),
            Support(ref order) => write!(f, "supports {}", order),
            Convoy(ref mv) => write!(f, "convoys {}", mv),
        }
    }
}

/// Validate that the deserialized value is not `Some(false)`.
#[cfg(feature = "serde")]
fn deserialize_true<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<bool>, D::Error> {
    let value = <Option<bool> as serde::Deserialize>::deserialize(de)?;
    if matches!(value, Some(false)) {
        return Err(serde::de::Error::custom("field must be true or null"));
    }

    Ok(value)
}

/// A move command with a destination and an optional convoy specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoveCommand<L> {
    dest: L,
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "deserialize_true"
        )
    )]
    /// Whether the order required, forbade, or didn't specify convoy usage.
    ///
    /// Right now, the adjudicator doesn't handle the forbid case, so there's no way to construct
    /// such a command.
    use_convoy: Option<bool>,
}

impl<L> MoveCommand<L> {
    /// Create a new move command without providing a convoy preference.
    pub fn new(dest: L) -> Self {
        Self {
            dest,
            use_convoy: None,
        }
    }

    /// Create a new move command which mandates that a convoy be used.
    pub fn with_mandatory_convoy(dest: L) -> Self {
        Self {
            dest,
            use_convoy: Some(true),
        }
    }

    /// Get the move command's destination region.
    pub fn dest(&self) -> &L {
        &self.dest
    }

    /// The order explicitly mandates the use of a convoy. If `true`, direct paths
    /// to the destination should not be considered when choosing a path.
    ///
    /// This can be `false` in two cases: The command didn't specify a convoy preference, or the command
    /// explicitly forbade a convoy. Different rulebooks have different opinions on how to interpret the
    /// absence of "via convoy" so the `MoveCommand` struct avoids forming an opinion on that case.
    pub fn mandates_convoy(&self) -> bool {
        self.use_convoy == Some(true)
    }

    /// The order explicitly mentions convoys, either mandating or forbidding their use.
    pub fn mentions_convoy(&self) -> bool {
        self.use_convoy.is_some()
    }
}

impl<L: Location> From<MoveCommand<L>> for MainCommand<L> {
    fn from(cmd: MoveCommand<L>) -> Self {
        MainCommand::Move(cmd)
    }
}

impl<L: Location> Command<L> for MoveCommand<L> {
    fn is_move(&self) -> bool {
        true
    }

    fn move_dest(&self) -> Option<&L> {
        Some(&self.dest)
    }
}

impl<L: Location> fmt::Display for MoveCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.dest.short_name().fmt(f)?;
        match self.use_convoy {
            Some(true) => write!(f, " via convoy"),
            Some(false) => write!(f, " no convoy"),
            None => Ok(()),
        }
    }
}

/// Declaration of the order to be supported by a support command.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
                if let MainCommand::Move(ref cmd) = other.command {
                    ut == &other.unit_type && fr == &other.region && to == cmd.dest()
                } else {
                    false
                }
            }
        }
    }
}

/// An army's move which a fleet should convoy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
                MainCommand::Move(cmd) => self.from() == &rhs.region && self.to() == cmd.dest(),
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
