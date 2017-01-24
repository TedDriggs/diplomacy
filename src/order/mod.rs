use UnitType;
use ShortName;
use Nation;
use geo::Location;

use std::fmt;

mod command;
pub use self::command::{Command, MainCommand, SupportedOrder, ConvoyedMove, RetreatCommand,
                        BuildCommand};

/// An order is issued by a nation and gives a command to a unit in a region.
#[derive(Clone, PartialEq, Eq, Hash)]
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
            nation: nation,
            unit_type: unit_type,
            region: region,
            command: command,
        }
    }

    /// Checks if two orders are moves in direct province opposition.
    /// This will return false if the start and destination are the
    /// same region.
    pub fn is_head_to_head(first: &Self, other: &Self) -> bool {
        first.command.move_dest() != Some(&first.region) &&
        first.command.move_dest().map(|d| d.province()) == Some(other.region.province()) &&
        other.command.move_dest().map(|d| d.province()) == Some(first.region.province())
    }
}

impl<L: Location, C: Command<L>> fmt::Display for Order<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: {} {} {}",
               self.nation.short_name(),
               self.unit_type.short_name(),
               self.region.short_name(),
               self.command)
    }
}

impl<L: Location, C: Command<L>> fmt::Debug for Order<L, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: {} {} {}",
               self.nation.short_name(),
               self.unit_type.short_name(),
               self.region.short_name(),
               self.command)
    }
}