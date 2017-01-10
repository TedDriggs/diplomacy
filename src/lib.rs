//! An adjudicator for orders in the board game Diplomacy. This adjudicator will
//! be fully compatible with the [Diplomacy Adjudicator Test Cases](http://web.inter.nl.net/users/L.B.Kruijswijk/).

pub mod geo;
pub mod order;
pub mod parser;

mod unit;
pub use unit::UnitType;

mod nation;
pub use nation::Nation;

mod judge;

/// Format trait for short naming of objects in orders.
pub trait ShortName {
    /// This method returns the short display name of the object.
    fn short_name(&self) -> String;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}