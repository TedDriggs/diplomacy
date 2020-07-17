use super::{DestStatus, Start};
use crate::judge::MappedRetreatOrder;
use crate::order::RetreatCommand;
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
    pub fn resolve(&'a self) -> Outcome<'a> {
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

        Outcome { by_order: outcomes }
    }
}

/// The result of a retreat phase adjudication.
pub struct Outcome<'a> {
    by_order: HashMap<&'a MappedRetreatOrder, OrderOutcome<'a>>,
}

impl<'a> Outcome<'a> {
    pub fn get(&'a self, order: &MappedRetreatOrder) -> Option<&'a OrderOutcome<'a>> {
        self.by_order.get(order)
    }
}

/// The outcome of a specific retreat phase order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderOutcome<'a> {
    /// The order was prevented by one or more other retreat orders.
    Prevented(&'a MappedRetreatOrder),
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

impl OrderOutcome<'_> {
    /// Check if the ordered unit disbanded at the conclusion of the retreat phase.
    pub fn did_disband(&self) -> bool {
        *self != OrderOutcome::Moves && *self != OrderOutcome::InvalidRecipient
    }
}
