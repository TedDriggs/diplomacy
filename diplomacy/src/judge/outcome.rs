use super::{
    retreat, Adjudicate, AttackOutcome, Context, ConvoyOutcome, HoldOutcome, MappedMainOrder,
    OrderState, ResolverState, SupportOutcome,
};
use from_variants::FromVariants;
use std::collections::HashMap;
use std::fmt;

/// The outcome of a specific order. The variant of the outcome will match the issued order
/// type.
#[derive(FromVariants, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderOutcome<O> {
    Illegal(IllegalOrder),
    Hold(HoldOutcome<O>),
    Move(AttackOutcome<O>),
    Support(SupportOutcome<O>),
    Convoy(ConvoyOutcome<O>),
}

impl<O> OrderOutcome<O> {
    /// Apply a function to any orders referenced by `self`, returning a new outcome.
    pub fn map_order<U>(self, map_fn: impl Fn(O) -> U) -> OrderOutcome<U> {
        use OrderOutcome::*;
        match self {
            Illegal(oo) => Illegal(oo),
            Hold(oo) => Hold(oo.map_order(map_fn)),
            Move(oo) => Move(oo.map_order(map_fn)),
            Support(oo) => Support(oo.map_order(map_fn)),
            Convoy(oo) => Convoy(oo.map_order(map_fn)),
        }
    }
}

impl<O> From<&'_ OrderOutcome<O>> for OrderState {
    fn from(other: &OrderOutcome<O>) -> Self {
        match other {
            OrderOutcome::Illegal(i) => i.into(),
            OrderOutcome::Hold(h) => h.into(),
            OrderOutcome::Move(m) => m.into(),
            OrderOutcome::Support(s) => s.into(),
            OrderOutcome::Convoy(c) => c.into(),
        }
    }
}

impl<O> From<OrderOutcome<O>> for OrderState {
    fn from(other: OrderOutcome<O>) -> Self {
        (&other).into()
    }
}

impl<O: fmt::Debug> fmt::Debug for OrderOutcome<O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderOutcome::Illegal(oo) => oo.fmt(f),
            OrderOutcome::Hold(oo) => oo.fmt(f),
            OrderOutcome::Move(oo) => oo.fmt(f),
            OrderOutcome::Support(oo) => oo.fmt(f),
            OrderOutcome::Convoy(oo) => oo.fmt(f),
        }
    }
}

/// Outcome for an order that was illegal and not considered during adjudication.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IllegalOrder {
    /// There is no unit in position to act on the order.
    NoUnit,
    /// There is a unit in the region to which the order is addressed, but it belongs to a nation
    /// other than the order issuer.
    ForeignUnit,
    /// The owning nation issued multiple orders to the same unit, and this order was discarded
    /// as a result.
    MultipleToSameUnit,
    /// There is no path for the unit to follow, even assuming all existing fleets are ordered
    /// to convoy the unit from its current location to its destination.
    UnreachableDestination,
}

impl From<&'_ IllegalOrder> for OrderState {
    fn from(_: &IllegalOrder) -> Self {
        OrderState::Fails
    }
}

/// Contains information about the outcome of a turn, used for reporting back
/// to players and for setting up the next turn.
pub struct Outcome<'a, A> {
    pub(in crate::judge) context: Context<'a, A>,
    pub(in crate::judge) resolver: ResolverState<'a>,
    pub(in crate::judge) orders: HashMap<&'a MappedMainOrder, OrderOutcome<&'a MappedMainOrder>>,
}

impl<'a, A: Adjudicate> Outcome<'a, A> {
    pub(in crate::judge) fn new(context: Context<'a, A>, resolver: ResolverState<'a>) -> Self {
        let mut state = resolver.clone();
        let orders = context
            .orders()
            .map(|ord| (ord, context.rules.explain(&context, &mut state, ord)))
            .chain(
                context
                    .illegal_orders
                    .iter()
                    .map(|(&ord, &reason)| (ord, reason.into())),
            )
            .collect();

        Self {
            context,
            resolver,
            orders,
        }
    }

    /// The orders that participated in resolution, in the order they were provided. This does not
    /// include illegal orders.
    pub fn orders(&self) -> impl Iterator<Item = &MappedMainOrder> {
        self.context.orders()
    }

    /// The union of all orders known to the outcome. This will include any illegal orders and the hold
    /// orders generated to ensure all units had an order during adjudication.
    pub fn all_orders(&self) -> impl Iterator<Item = &MappedMainOrder> {
        self.orders.keys().copied()
    }

    pub fn all_orders_with_outcomes(
        &self,
    ) -> impl Iterator<Item = (&MappedMainOrder, &OrderOutcome<&MappedMainOrder>)> {
        self.orders.iter().map(|(ord, outcome)| (*ord, outcome))
    }

    pub fn get(
        &'a self,
        order: &'a MappedMainOrder,
    ) -> Option<&'a OrderOutcome<&'a MappedMainOrder>> {
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
            .all_orders()
            .map(|ord| {
                (
                    ord.clone(),
                    OrderState::from(other.get(ord).expect("Outcome should be complete")),
                )
            })
            .collect()
    }
}
