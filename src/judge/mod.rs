mod resolver;
mod strength;
mod state_type;
mod calc;
mod convoy;
mod prelude;
pub mod support;
mod rulebook;

use std::collections::HashMap;
use std::fmt;

pub use self::state_type::{OrderState, ResolutionState, OccupationOutcome};

use order::{Order, MainCommand};
use geo::{RegionKey, Map};
pub use self::resolver::{ResolverContext, ResolverState};
pub use self::rulebook::Rulebook;

pub type MappedMainOrder = Order<RegionKey, MainCommand<RegionKey>>;

/// Adjudicate a set of orders
pub fn adjudicate<'a, O: IntoIterator<Item = MappedMainOrder>>
    (map: &'a Map,
     orders: O)
     -> HashMap<MappedMainOrder, OrderState> {
    let ctx = ResolverContext::new(map, orders.into_iter().collect());
    ctx.resolve()
}

pub trait Outcome : fmt::Debug {}