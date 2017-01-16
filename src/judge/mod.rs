mod resolver;
mod strength;
mod state_type;
mod calc;
mod prelude;
mod support;
mod rulebook;

use std::collections::HashMap;

pub use self::state_type::{OrderState, ResolutionState};

use order::{Order, MainCommand};
use geo::{Region, Map};
use self::resolver::ResolverContext;

pub type MappedMainOrder<'a> = Order<&'a Region<'a>, MainCommand<&'a Region<'a>>>;

/// Adjudicate a set of orders
pub fn adjudicate<'a, O: IntoIterator<Item = MappedMainOrder<'a>>>
    (map: &'a Map<'a>,
     orders: O)
     -> HashMap<MappedMainOrder<'a>, OrderState> {
    let ctx = ResolverContext::new(map, orders.into_iter().collect());
    ctx.resolve()
}