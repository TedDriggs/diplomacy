//! Contains the logic needed to adjudicate a turn.

mod calc;
mod convoy;
mod outcome;
mod resolver;
mod rulebook;
mod state_type;
mod strength;
pub mod support;

use std::collections::HashMap;

pub use self::outcome::Outcome;
pub use self::state_type::{OccupationOutcome, OrderState};

pub use self::resolver::{Adjudicate, ResolverContext, ResolverState};
pub use self::rulebook::Rulebook;
use crate::geo::{Border, Map, RegionKey, Terrain};
use crate::order::{MainCommand, Order};
use crate::UnitType;

pub type MappedMainOrder = Order<RegionKey, MainCommand<RegionKey>>;

/// Adjudicate a set of orders
pub fn adjudicate<O: IntoIterator<Item = MappedMainOrder>>(
    map: &Map,
    orders: O,
) -> HashMap<MappedMainOrder, OrderState> {
    let ctx = ResolverContext::new(map, orders.into_iter().collect());
    ctx.resolve()
}

impl Border {
    fn is_passable_by(&self, unit_type: UnitType) -> bool {
        unit_type.can_occupy(self.terrain())
    }
}

impl UnitType {
    fn can_occupy(self, terrain: Terrain) -> bool {
        match terrain {
            Terrain::Coast => true,
            Terrain::Land => self == UnitType::Army,
            Terrain::Sea => self == UnitType::Fleet,
        }
    }
}
