mod resolver;
mod strength;
mod state_type;
mod calc;
mod convoy;
mod prelude;
pub mod support;
mod rulebook;

use std::collections::HashMap;

pub use self::state_type::{OrderState, ResolutionState};

use order::{Order, MainCommand};
use geo::{RegionKey, Map};
pub use self::resolver::{ResolverContext, ResolverState};

pub type MappedMainOrder = Order<RegionKey, MainCommand<RegionKey>>;

/// Adjudicate a set of orders
pub fn adjudicate<'a, O: IntoIterator<Item = MappedMainOrder>>
    (map: &'a Map,
     orders: O)
     -> HashMap<MappedMainOrder, OrderState> {
    let ctx = ResolverContext::new(map, orders.into_iter().collect());
    ctx.resolve()
}