use UnitType;
use geo::Region;

mod command;


pub use self::command::{
    Command,
    SupportedOrder,
    ConvoyedMove,
    RetreatCommand,
    BuildCommand,
};

/// An order issued by a nation. An order gives a command to a unit in a region.
pub struct Order<'a, Comm> {
    nation: (),
    
    /// The region in which the addressed unit resides (except for build commands).
    region: &'a Region<'a>,
    
    /// The type of unit addressed.
    unit_type: UnitType,
    command: Comm,
}