//! An adjudicator for orders in the board game Diplomacy. This adjudicator will
//! be fully compatible with the [Diplomacy Adjudicator Test Cases](http://web.inter.nl.net/users/L.B.Kruijswijk/).

pub mod geo;
pub mod order;
pub mod parser;

mod game;
mod unit;
pub use crate::unit::UnitType;

mod nation;
pub use crate::nation::Nation;

pub mod judge;

/// Format trait for short naming of objects in orders.
pub trait ShortName {
    /// This method returns the short display name of the object.
    fn short_name(&self) -> String;
}