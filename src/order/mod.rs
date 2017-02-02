//! The model for an order issued to a unit.

use std::fmt;

use ShortName;
use Nation;
use geo::Location;
use UnitType;

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
        first.move_dest() != Some(&first.region) &&
        first.move_dest().map(|d| d.province()) == Some(other.region.province()) &&
        other.move_dest().map(|d| d.province()) == Some(first.region.province())
    }
    
    /// Write the canonical form of the order to the formatter.
    ///  
    /// For readability, this is used by both the Debug and Display traits.
    fn write_short(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: {} {} {}",
               self.nation.short_name(),
               self.unit_type.short_name(),
               self.region.short_name(),
               self.command)
    }
}

impl<L: Location, C: Command<L>> Command<L> for Order<L, C> {
    fn move_dest(&self) -> Option<&L> {
        self.command.move_dest()
    }
    
    fn is_move(&self) -> bool {
        self.command.is_move()
    }
    
    fn is_move_to_province(&self, p: &L::Province) -> bool {
        self.command.is_move_to_province(p)
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

pub type MainOrder<L> = Order<L, MainCommand<L>>;

pub type RetreatOrder<L> = Order<L, RetreatCommand<L>>;

pub type BuildOrder<L> = Order<L, BuildCommand>;