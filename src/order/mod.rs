use UnitType;
use ShortName;
use Nation;
use geo::Region;

use std::fmt;

mod command;
pub use self::command::{
    MainCommand,
    SupportedOrder,
    ConvoyedMove,
    RetreatCommand,
    BuildCommand,
};

/// An order is issued by a nation and gives a command to a unit in a region.
pub struct Order<'a, C : fmt::Display> {
    pub nation: Nation,
    
    /// The region in which the addressed unit resides (except for build commands).
    pub region: &'a Region<'a>,
    
    /// The type of unit addressed.
    pub unit_type: UnitType,
    
    /// The command dispatched to the order's region.
    pub command: C,
}

impl<'a, C : fmt::Display> fmt::Display for Order<'a, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} {} {}", 
            self.nation.short_name(), 
            self.unit_type.short_name(), 
            self.region.short_name(), 
            self.command
        )
    }
}