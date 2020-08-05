//! An adjudicator for orders in the board game Diplomacy. This adjudicator will
//! be fully compatible with the [Diplomacy Adjudicator Test Cases](http://web.inter.nl.net/users/L.B.Kruijswijk/).

pub mod calendar;
pub mod geo;
pub mod judge;
mod nation;
pub mod order;
pub mod parser;
mod time;
mod unit;

#[doc(inline)]
pub use crate::calendar::{Calendar, Month};
pub use crate::nation::Nation;
#[doc(inline)]
pub use crate::order::{Command, Order};
pub use crate::time::{Phase, Season, Time};
pub use crate::unit::{Unit, UnitPosition, UnitPositions, UnitType};

/// Format trait for short naming of objects in orders.
pub trait ShortName {
    /// This method returns the short display name of the object.
    fn short_name(&self) -> std::borrow::Cow<'_, str>;
}
