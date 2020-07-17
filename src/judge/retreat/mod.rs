//! Types and functions to set up, adjudicate, and report on a retreat phase.
//!
//! The retreat phase happens after every main phase, and allows dislodged units to either
//! disband or move to some other vacant location. Unlike the main phase, convoying and support
//! are not permitted during retreats.

mod resolver;
mod start;

pub use self::start::{DestStatus, Destinations, Start};
pub use self::resolver::{Context, OrderOutcome, Outcome};