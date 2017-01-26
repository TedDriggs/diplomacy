use std::collections::HashMap;
use std::convert::From;
use std::fmt;

use order::{Command};
use judge::resolver::{self, Adjudicate};
use super::{MappedMainOrder, ResolverState, ResolverContext, OrderState};
use judge::calc::dislodger_of;
use judge::Rulebook;

/// Contains information about the outcome of a turn, used for reporting back
/// to players and for setting up the next turn.
pub struct Outcome<'a, A: Adjudicate> {
    context: &'a ResolverContext<'a>,
    resolver: ResolverState<'a, A>,
}

impl<'a, A: Adjudicate> Outcome<'a, A> {
    pub fn moved(&self) -> Vec<&MappedMainOrder> {
        self.context
            .orders_ref()
            .into_iter()
            .filter(|o| o.is_move() && self.get(o) == Some(OrderState::Succeeds))
            .collect()
    }

    /// Gets a map of orders whose recipients were dislodged to the order which dislodged them.
    pub fn dislodged(&self) -> HashMap<&MappedMainOrder, &MappedMainOrder> {
        let mut dislodged = HashMap::new();
        for order in self.context.orders_ref() {
            if let Some(dl_ord) = dislodger_of(&self.context, &mut self.resolver.clone(), order) {
                dislodged.insert(order, dl_ord);
            }
        }

        dislodged
    }

    pub fn get(&self, order: &MappedMainOrder) -> Option<OrderState> {
        self.resolver.get_state(order)
    }
}

impl<'a> From<&'a ResolverContext<'a>> for Outcome<'a, Rulebook> {
    fn from(rc: &'a ResolverContext<'a>) -> Self {
        Outcome {
            resolver: rc.resolve_to_state(),
            context: rc,
        }
    }
}

impl<'a, A: Adjudicate> fmt::Display for Outcome<'a, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "MOVED")?;
        for ord in self.moved() {
            writeln!(f, "  {}", ord)?;
        }
        
        writeln!(f, "DISLODGED")?;
        for (dislodged, dislodger) in self.dislodged() {
            writeln!(f, "  {} | {}", dislodged, dislodger)?;
        }
        
        Ok(())
    }
}

// BLACK MAGIC.
impl<'a, A: Adjudicate> ResolverState<'a, A> {
    fn get_state(&self, order: &MappedMainOrder) -> Option<OrderState> {
        resolver::get_state(self, order)
    }
}