//! The model for an order issued to a unit.

use crate::{geo::Location, Nation, ShortName, Unit, UnitPosition, UnitType};
use std::{borrow::Cow, fmt};

mod command;
pub use self::command::{
    BuildCommand, Command, ConvoyedMove, MainCommand, MoveCommand, RetreatCommand, SupportedOrder,
};

/// An order is issued by a nation and gives a command to a unit in a region.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Order<L: Location, C: Command<L>> {
    /// The nation to which the commanded unit (or province) belongs.
    pub nation: Nation,

    /// The region in which the addressed unit resides (except for build commands).
    pub region: L,

    /// The type of unit addressed.
    pub unit_type: UnitType,

    /// The command dispatched to the order's region.
    pub command: C,
}

impl<L: Location, C: Command<L>> Order<L, C> {
    /// Create a new order.
    pub fn new(nation: Nation, unit_type: UnitType, region: L, command: C) -> Self {
        Order {
            nation,
            unit_type,
            region,
            command,
        }
    }

    pub fn unit_position(&self) -> UnitPosition<'_, &L> {
        UnitPosition::from(self)
    }

    /// Write the canonical form of the order to the formatter.
    ///
    /// For readability, this is used by both the Debug and Display traits.
    fn write_short(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} {} {}",
            self.nation.short_name(),
            self.unit_type.short_name(),
            self.region.short_name(),
            self.command
        )
    }
}

impl<L: Location, C: Command<L>> Command<L> for Order<L, C> {
    fn move_dest(&self) -> Option<&L> {
        self.command.move_dest()
    }

    fn is_move(&self) -> bool {
        self.command.is_move()
    }
}

impl<L: Location, C: Command<L>> fmt::Display for Order<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_short(f)
    }
}

impl<L: Location, C: Command<L>> fmt::Debug for Order<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_short(f)
    }
}

impl<'a, L: Location, C: Command<L>> From<&'a Order<L, C>> for Unit<'a> {
    fn from(order: &'a Order<L, C>) -> Self {
        Unit::new(Cow::Borrowed(&order.nation), order.unit_type)
    }
}

impl<'a, L: Location, C: Command<L>> From<&'a Order<L, C>> for UnitPosition<'a, &'a L> {
    fn from(order: &'a Order<L, C>) -> Self {
        UnitPosition::new(Unit::from(order), &order.region)
    }
}

pub type MainOrder<L> = Order<L, MainCommand<L>>;

pub type RetreatOrder<L> = Order<L, RetreatCommand<L>>;

pub type BuildOrder<L> = Order<L, BuildCommand>;
