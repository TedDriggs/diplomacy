use super::{
    calc::dislodger_of, calc::prevent_results, convoy, Adjudicate, AttackOutcome, ConvoyOutcome,
    HoldOutcome, MappedMainOrder, OrderState, ResolverContext, ResolverState, SupportOutcome,
};
use crate::geo::{Border, RegionKey};
use crate::order::Command;
use from_variants::FromVariants;
use std::collections::{BTreeSet, HashMap};
use std::fmt;

/// The outcome of a specific order. The variant of the outcome will match the issued order
/// type.
#[derive(FromVariants)]
pub enum OrderOutcome<'a> {
    Hold(HoldOutcome<'a>),
    Move(AttackOutcome<'a>),
    Support(SupportOutcome<'a>),
    Convoy(ConvoyOutcome<'a>),
}

impl From<&'_ OrderOutcome<'_>> for OrderState {
    fn from(other: &OrderOutcome<'_>) -> Self {
        match other {
            OrderOutcome::Hold(h) => h.into(),
            OrderOutcome::Move(m) => m.into(),
            OrderOutcome::Support(s) => s.into(),
            OrderOutcome::Convoy(c) => c.into(),
        }
    }
}

impl From<OrderOutcome<'_>> for OrderState {
    fn from(other: OrderOutcome<'_>) -> Self {
        (&other).into()
    }
}

impl fmt::Debug for OrderOutcome<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderOutcome::Hold(oo) => oo.fmt(f),
            OrderOutcome::Move(oo) => oo.fmt(f),
            OrderOutcome::Support(oo) => oo.fmt(f),
            OrderOutcome::Convoy(oo) => oo.fmt(f),
        }
    }
}

impl PartialEq<OrderState> for OrderOutcome<'_> {
    fn eq(&self, other: &OrderState) -> bool {
        OrderState::from(self) == *other
    }
}

/// Contains information about the outcome of a turn, used for reporting back
/// to players and for setting up the next turn.
pub struct Outcome<'a, A: Adjudicate> {
    context: &'a ResolverContext<'a>,
    resolver: ResolverState<'a, A>,
    orders: HashMap<&'a MappedMainOrder, OrderOutcome<'a>>,
}

impl<'a, A: Adjudicate> Outcome<'a, A> {
    pub(in crate::judge) fn new(
        context: &'a ResolverContext,
        resolver: ResolverState<'a, A>,
    ) -> Self {
        let mut state = resolver.clone();
        let orders = context
            .orders()
            .iter()
            .map(|ord| {
                (
                    ord,
                    resolver.adjudicator().explain(context, &mut state, ord),
                )
            })
            .collect();

        Self {
            context,
            resolver,
            orders,
        }
    }

    pub fn moved(&self) -> Vec<&MappedMainOrder> {
        self.context
            .orders()
            .iter()
            .filter(|o| {
                o.is_move()
                    && self
                        .get(o)
                        .map(|outcome| outcome == &OrderState::Succeeds)
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Gets a map of orders whose recipients were dislodged to the order which dislodged them.
    pub fn dislodged(&self) -> HashMap<&MappedMainOrder, &MappedMainOrder> {
        let mut dislodged = HashMap::new();
        for order in self.context.orders() {
            if let Some(dl_ord) = dislodger_of(&self.context, &mut self.resolver.clone(), order) {
                dislodged.insert(order, dl_ord);
            }
        }

        dislodged
    }

    pub fn get(&'a self, order: &'a MappedMainOrder) -> Option<&'a OrderOutcome<'a>> {
        self.orders.get(order)
    }

    pub fn get_retreat_destinations(&self) -> HashMap<&MappedMainOrder, BTreeSet<&RegionKey>> {
        let world = self.context.world_map;
        let mut state = self.resolver.clone();
        self.dislodged()
            .iter()
            .map(|(dislodged, dislodger)| {
                (
                    *dislodged,
                    world
                        .borders_containing(&dislodged.region)
                        .into_iter()
                        .filter(|b| {
                            self.is_valid_retreat_route(&mut state, dislodged, dislodger, b)
                        })
                        .filter_map(|b| b.dest_from(&dislodged.region))
                        .collect::<BTreeSet<_>>(),
                )
            })
            .collect()
    }

    fn is_valid_retreat_route(
        &self,
        state: &mut ResolverState<'a, impl Adjudicate>,
        retreater: &MappedMainOrder,
        dislodger: &MappedMainOrder,
        border: &Border,
    ) -> bool {
        if !border.is_passable_by(retreater.unit_type) {
            return false;
        }

        let dest = if let Some(dst) = border.dest_from(&retreater.region) {
            dst
        } else {
            return false;
        };

        if dest.province() == dislodger.region.province()
            && !convoy::route_exists(&self.context, state, dislodger)
        {
            return false;
        }

        prevent_results(&self.context, state, dest.province()).is_empty()
    }
}

#[allow(clippy::implicit_hasher)]
impl<A: Adjudicate> From<Outcome<'_, A>> for HashMap<MappedMainOrder, OrderState> {
    fn from(other: Outcome<'_, A>) -> Self {
        other
            .context
            .orders()
            .iter()
            .map(|ord| {
                (
                    ord.clone(),
                    OrderState::from(other.get(ord).expect("Outcome should be complete")),
                )
            })
            .collect()
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
