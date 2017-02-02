//! Contains the logic needed to adjudicate a turn.

mod resolver;
mod strength;
mod state_type;
mod calc;
mod convoy;
pub mod support;
mod rulebook;
mod outcome;

use std::collections::HashMap;

pub use self::outcome::Outcome;
pub use self::state_type::{OrderState, OccupationOutcome};

use order::{Order, MainCommand};
use geo::{Border, RegionKey, Map, Terrain};
use UnitType;
pub use self::resolver::{Adjudicate, ResolverContext, ResolverState};
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

impl Border {
    fn is_passable_by(&self, unit_type: &UnitType) -> bool {
        unit_type.can_occupy(self.terrain())
    }
}

impl UnitType {
    fn can_occupy(&self, terrain: &Terrain) -> bool {
        match *terrain {
            Terrain::Coast => true,
            Terrain::Land => self == &UnitType::Army,
            Terrain::Sea => self == &UnitType::Fleet,
        }
    }
}