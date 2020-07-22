use super::{
    retreat, Adjudicate, AttackOutcome, ConvoyOutcome, HoldOutcome, MappedMainOrder, OrderState,
    ResolverContext, ResolverState, SupportOutcome,
};
use crate::order::Command;
use from_variants::FromVariants;
use std::collections::HashMap;
use std::fmt;

/// The outcome of a specific order. The variant of the outcome will match the issued order
/// type.
#[derive(FromVariants, PartialEq, Eq)]
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
pub struct Outcome<'a, A> {
    pub(in crate::judge) context: &'a ResolverContext<'a>,
    pub(in crate::judge) resolver: ResolverState<'a, A>,
    pub(in crate::judge) orders: HashMap<&'a MappedMainOrder, OrderOutcome<'a>>,
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

    /// The orders that participated in resolution, in the order they were provided.
    pub fn orders(&self) -> &[MappedMainOrder] {
        self.context.orders()
    }

    /// Get successful move orders from the phase.
    pub fn moved(&self) -> Vec<&MappedMainOrder> {
        self.orders
            .iter()
            .filter(|(o, outcome)| {
                o.is_move() && **outcome == OrderOutcome::Move(AttackOutcome::Succeeds)
            })
            .map(|(order, _)| *order)
            .collect()
    }

    pub fn get(&'a self, order: &'a MappedMainOrder) -> Option<&'a OrderOutcome<'a>> {
        self.orders.get(order)
    }

    /// Calculate retreat phase starting data based on this main-phase outcome.
    pub fn to_retreat_start(&'a self) -> retreat::Start<'a> {
        retreat::Start::new(self)
    }

    #[cfg(feature = "dependency-graph")]
    pub fn dependencies(&self) -> impl fmt::Display {
        struct Dependencies(std::collections::BTreeSet<(MappedMainOrder, MappedMainOrder)>);

        impl fmt::Display for Dependencies {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                writeln!(f, "digraph G {{")?;
                for (src, dest) in &self.0 {
                    writeln!(f, r#"  "{}" -> "{}""#, src, dest)?;
                }

                writeln!(f, "}}")
            }
        }

        Dependencies(self.resolver.dependencies())
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

        Ok(())
    }
}
