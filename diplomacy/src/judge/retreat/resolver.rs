use super::{DestStatus, Start};
use crate::judge::MappedRetreatOrder;
use crate::order::{Command, RetreatCommand};
use crate::{geo::ProvinceKey, geo::RegionKey, Unit, UnitPosition, UnitPositions};
use std::collections::HashMap;

/// The immutable parts of retreat phase adjudication.
pub struct Context<'a> {
    start: &'a Start<'a>,
    orders: Vec<MappedRetreatOrder>,
}

impl<'a> Context<'a> {
    pub fn new(start: &'a Start<'a>, orders: impl IntoIterator<Item = MappedRetreatOrder>) -> Self {
        Self {
            start,
            orders: orders.into_iter().collect(),
        }
    }

    /// Adjudicate a retreat phase and determine which units move or are disbanded.
    pub fn resolve(&self) -> Outcome {
        let mut outcomes = HashMap::new();
        let mut destinations = HashMap::new();

        for order in &self.orders {
            let dests = if let Some(dests) = self
                .start
                .retreat_destinations()
                .get(&order.unit_position())
            {
                dests
            } else {
                outcomes.insert(order, OrderOutcome::InvalidRecipient);
                continue;
            };

            match &order.command {
                RetreatCommand::Hold => {
                    outcomes.insert(order, OrderOutcome::DisbandsAsOrdered);
                }
                RetreatCommand::Move(dest) => match dests.get(dest) {
                    DestStatus::Available => {
                        if let Some(conflicted) = destinations.insert(dest.province(), order) {
                            outcomes.insert(conflicted, OrderOutcome::Prevented(order));
                            outcomes.insert(order, OrderOutcome::Prevented(conflicted));
                        } else {
                            outcomes.insert(order, OrderOutcome::Moves);
                        }
                    }
                    status => {
                        outcomes.insert(order, OrderOutcome::InvalidDestination(status));
                    }
                },
            }
        }

        Outcome::new(outcomes, self.start.unit_positions.clone())
    }
}

/// The result of a retreat phase adjudication, and unit positions after the retreat phase
/// and its preceding main phase.
pub struct Outcome<'a> {
    by_order: HashMap<&'a MappedRetreatOrder, OrderOutcome<&'a MappedRetreatOrder>>,
    unit_positions: HashMap<&'a ProvinceKey, UnitPosition<'a>>,
}

impl<'a> Outcome<'a> {
    fn new(
        by_order: HashMap<&'a MappedRetreatOrder, OrderOutcome<&'a MappedRetreatOrder>>,
        retreat_start_positions: HashMap<&'a ProvinceKey, UnitPosition<'a>>,
    ) -> Self {
        let mut unit_positions = retreat_start_positions;
        for (order, outcome) in &by_order {
            if let Some(dest) = order.move_dest() {
                if let OrderOutcome::Moves = outcome {
                    unit_positions
                        .insert(dest.province(), UnitPosition::new((*order).into(), dest));
                }
            }
        }

        Self {
            by_order,
            unit_positions,
        }
    }

    pub fn get(
        &'a self,
        order: &MappedRetreatOrder,
    ) -> Option<&'a OrderOutcome<&'a MappedRetreatOrder>> {
        self.by_order.get(order)
    }

    /// Iterate over the outcomes for each retreat order.
    pub fn order_outcomes(
        &self,
    ) -> impl Iterator<Item = (&MappedRetreatOrder, &OrderOutcome<&'a MappedRetreatOrder>)> {
        self.by_order.iter().map(|(k, v)| (*k, v))
    }
}

impl UnitPositions<RegionKey> for Outcome<'_> {
    fn unit_positions(&self) -> Vec<UnitPosition> {
        self.unit_positions.unit_positions()
    }

    fn find_province_occupier(&self, province: &ProvinceKey) -> Option<UnitPosition> {
        self.unit_positions.find_province_occupier(province)
    }

    fn find_region_occupier(&self, region: &RegionKey) -> Option<Unit> {
        self.unit_positions.find_region_occupier(region)
    }
}

/// The outcome of a specific retreat phase order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderOutcome<O> {
    /// The order was prevented by one or more other retreat orders.
    Prevented(O),
    /// The order destination was invalid. The `DestStatus` provides information on why
    /// the destination was invalid.
    InvalidDestination(DestStatus),
    /// The order was issued to a region that does not contain a retreating unit.
    ///
    /// The region may be vacant, or may contain a unit that was not dislodged.
    InvalidRecipient,
    /// The unit successfully retreats to a new region
    Moves,
    /// The unit was ordered to disband and did so.
    DisbandsAsOrdered,
}

impl<O> OrderOutcome<O> {
    /// Apply a function to any orders referenced by `self`, returning a new outcome.
    pub fn map_order<U>(self, map_fn: impl Fn(O) -> U) -> OrderOutcome<U> {
        use OrderOutcome::*;
        match self {
            Prevented(o) => Prevented(map_fn(o)),
            InvalidDestination(status) => InvalidDestination(status),
            InvalidRecipient => InvalidRecipient,
            Moves => Moves,
            DisbandsAsOrdered => DisbandsAsOrdered,
        }
    }

    /// Check if the ordered unit disbanded at the conclusion of the retreat phase.
    pub fn did_disband(&self) -> bool {
        match self {
            Self::Moves | Self::InvalidRecipient => false,
            Self::Prevented(_) | Self::InvalidDestination(_) | Self::DisbandsAsOrdered => true,
        }
    }
}
