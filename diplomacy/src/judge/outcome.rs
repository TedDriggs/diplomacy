use super::{
    retreat, Adjudicate, AttackOutcome, Context, ConvoyOutcome, HoldOutcome, MappedMainOrder,
    OrderState, ResolverState, SupportOutcome,
};
use from_variants::FromVariants;
use std::collections::HashMap;
use std::fmt;

/// The outcome of a specific order. The variant of the outcome will match the issued order
/// type.
#[derive(FromVariants, PartialEq, Eq)]
pub enum OrderOutcome<'a> {
    Invalid(InvalidOrder),
    Hold(HoldOutcome<'a>),
    Move(AttackOutcome<'a>),
    Support(SupportOutcome<'a>),
    Convoy(ConvoyOutcome<'a>),
}

impl From<&'_ OrderOutcome<'_>> for OrderState {
    fn from(other: &OrderOutcome<'_>) -> Self {
        match other {
            OrderOutcome::Invalid(i) => i.into(),
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
            OrderOutcome::Invalid(oo) => oo.fmt(f),
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

/// Outcome for an order that was invalid and not considered during adjudication.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InvalidOrder {
    /// There is no unit in position to act on the order.
    NoUnit,
    /// There is a unit in the region to which the order is addressed, but it belongs to a nation
    /// other than the order issuer.
    ForeignUnit,
    /// The owning nation issued multiple orders to the same unit, and this order was discarded
    /// as a result.
    MultipleToSameUnit,
}

impl From<&'_ InvalidOrder> for OrderState {
    fn from(_: &InvalidOrder) -> Self {
        OrderState::Fails
    }
}

/// Contains information about the outcome of a turn, used for reporting back
/// to players and for setting up the next turn.
pub struct Outcome<'a, A> {
    pub(in crate::judge) context: Context<'a, A>,
    pub(in crate::judge) resolver: ResolverState<'a>,
    pub(in crate::judge) orders: HashMap<&'a MappedMainOrder, OrderOutcome<'a>>,
}

impl<'a, A: Adjudicate> Outcome<'a, A> {
    pub(in crate::judge) fn new(context: Context<'a, A>, resolver: ResolverState<'a>) -> Self {
        let mut state = resolver.clone();
        let orders = context
            .orders()
            .map(|ord| (ord, context.rules.explain(&context, &mut state, ord)))
            .chain(
                context
                    .invalid_orders
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
    /// include invalid orders.
    pub fn orders(&self) -> impl Iterator<Item = &MappedMainOrder> {
        self.context.orders()
    }

    /// The union of all orders known to the outcome. This will include any invalid orders and the hold
    /// orders generated to ensure all units had an order during adjudication.
    pub fn all_orders(&self) -> impl Iterator<Item = &MappedMainOrder> {
        self.orders.keys().copied()
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
            .map(|ord| {
                (
                    ord.clone(),
                    OrderState::from(other.get(ord).expect("Outcome should be complete")),
                )
            })
            .collect()
    }
}
