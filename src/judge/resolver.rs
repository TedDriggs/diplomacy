use super::{MappedMainOrder, OrderState, Rulebook};
use crate::geo::{Map, ProvinceKey, RegionKey};
use crate::order::{Command, MainCommand};
use std::collections::{HashMap, HashSet};

/// A clonable container for a rulebook which can be used to adjudicate a turn.
pub trait Adjudicate: Clone {
    /// Determine the success of an order.
    fn adjudicate<'a>(
        &self,
        context: &'a ResolverContext<'a>,
        resolver: &mut ResolverState<'a, Self>,
        order: &'a MappedMainOrder,
    ) -> OrderState;
}

/// The immutable inputs for a resolution equation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverContext<'a> {
    /// Set of orders which were issued during this turn.
    orders: Vec<MappedMainOrder>,

    /// The map against which orders were issued.
    pub world_map: &'a Map,
}

impl<'a> ResolverContext<'a> {
    /// Creates a new resolver context for a set of orders on a map.
    pub fn new(world_map: &'a Map, orders: Vec<MappedMainOrder>) -> Self {
        ResolverContext { world_map, orders }
    }

    /// Get a view of the orders.
    pub fn orders(&self) -> &[MappedMainOrder] {
        &self.orders
    }

    pub fn resolve_to_state(&self) -> ResolverState<super::rulebook::Rulebook> {
        let mut rs = ResolverState::with_adjudicator(super::rulebook::Rulebook);

        // XXX Paradoxes can trigger infinite loops if we're not careful, so we
        // try to resolve convoys first so that cycle detection will make progress
        // towards a global resolution.
        let mut orders = self.orders().iter().collect::<Vec<_>>();
        orders.sort_unstable_by_key(|r| {
            if let MainCommand::Convoy(..) = r.command {
                1
            } else {
                2
            }
        });

        for order in orders {
            rs.resolve(&self, order);
        }

        rs
    }

    pub fn explain(
        &'a self,
        state: &'a mut ResolverState<'a, Rulebook>,
        order: &'a MappedMainOrder,
    ) {
        match order.command {
            MainCommand::Move(..) => println!(
                "{}: {:?}",
                order,
                Rulebook::adjudicate_move(self, state, order)
            ),
            MainCommand::Support(..) => println!(
                "{}: {:?}",
                order,
                Rulebook::adjudicate_support(self, state, order)
            ),
            MainCommand::Hold => println!(
                "{}: {:?}",
                order,
                Rulebook::adjudicate_hold(self, state, order)
            ),
            MainCommand::Convoy(..) => println!(
                "{}: {:?}",
                order,
                Rulebook::adjudicate_convoy(self, state, order)
            ),
        }
    }

    /// Resolve the orders in the context, producing a map of orders to outcomes
    pub fn resolve(self) -> HashMap<MappedMainOrder, OrderState> {
        let rs = self.resolve_to_state();
        rs.into()
    }

    pub fn find_order_to_province(&'a self, p: &ProvinceKey) -> Option<&'a MappedMainOrder> {
        self.orders().iter().find(|o| &o.region == p)
    }

    pub fn find_order_to_region(&'a self, r: &RegionKey) -> Option<&'a MappedMainOrder> {
        self.orders().iter().find(|o| &o.region == r)
    }
}

#[allow(clippy::implicit_hasher)]
impl<'a> From<ResolverContext<'a>> for HashMap<MappedMainOrder, OrderState> {
    fn from(rc: ResolverContext<'a>) -> Self {
        rc.resolve()
    }
}

impl<'a> From<&'a ResolutionState> for OrderState {
    fn from(rs: &'a ResolutionState) -> Self {
        match *rs {
            ResolutionState::Guessing(os) | ResolutionState::Known(os) => os,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ResolutionState {
    Guessing(OrderState),
    Known(OrderState),
}

impl std::fmt::Debug for ResolutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({:?})",
            match self {
                ResolutionState::Guessing(..) => "Guessing",
                ResolutionState::Known(..) => "Known",
            },
            OrderState::from(self)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverState<'a, A: Adjudicate> {
    state: HashMap<&'a MappedMainOrder, ResolutionState>,
    /// Orders which form part of a paradox. These should only be convoy orders, and will
    /// be treated as hold orders to advance resolution.
    paradoxical_orders: HashSet<&'a MappedMainOrder>,
    dependency_chain: Vec<&'a MappedMainOrder>,
    adjudicator: A,
}

impl<'a, A: Adjudicate> ResolverState<'a, A> {
    /// Create a new resolver for a given rulebook.
    pub fn with_adjudicator(adjudicator: A) -> Self {
        ResolverState {
            state: HashMap::new(),
            dependency_chain: vec![],
            paradoxical_orders: HashSet::new(),
            adjudicator,
        }
    }

    fn clear_state(&mut self, order: &MappedMainOrder) {
        self.state.remove(order);
    }

    fn set_state(&mut self, order: &'a MappedMainOrder, resolution: ResolutionState) {
        self.state.insert(order, resolution);
    }

    /// Get the current projected outcome of an order.
    fn get(&self, order: &MappedMainOrder) -> Option<OrderState> {
        self.state.get(order).map(|rs| rs.into())
    }

    fn knows_outcome_of(&self, order: &MappedMainOrder) -> bool {
        if let Some(ResolutionState::Known(_)) = self.state.get(order) {
            true
        } else {
            false
        }
    }

    pub(crate) fn order_in_paradox(&self, order: &'a MappedMainOrder) -> bool {
        self.paradoxical_orders.contains(order)
    }

    /// Create a clone of the resolver state, add a guess at the success or failure
    /// of the given order, then adjudicate the order with the amended resolver.
    ///
    /// This returns the entire guesser because in some cases the calling resolver needs
    /// the dependency chain and the entire state generated during the post-guess adjudication.
    fn with_guess(
        &self,
        context: &'a ResolverContext<'a>,
        order: &'a MappedMainOrder,
        guess: OrderState,
    ) -> (Self, OrderState) {
        let mut guesser = self.clone();
        guesser.set_state(order, ResolutionState::Guessing(guess));
        let result = self.adjudicator.adjudicate(context, &mut guesser, order);
        (guesser, result)
    }

    /// When a dependency cycle is detected, attempt to resolve all orders in the cycle.
    fn resolve_dependency_cycle(&mut self, cycle: &[&'a MappedMainOrder]) {
        use self::ResolutionState::*;
        use super::OrderState::*;

        // if every order in the cycle is a move, then this is a circular move
        if cycle.iter().all(|o| o.is_move()) {
            for o in cycle {
                self.set_state(o, Known(Succeeds));
            }
        } else {
            for o in cycle {
                self.dependency_chain.pop();
                if self.knows_outcome_of(o) {
                    continue;
                }

                if let MainCommand::Convoy(_) = o.command {
                    self.paradoxical_orders.insert(o);
                    self.set_state(o, ResolutionState::Known(OrderState::Fails));
                } else {
                    self.clear_state(o);
                }
            }
        }
    }

    /// Resolve whether an order succeeds or fails, possibly updating
    /// the resolver's state in the process.
    pub fn resolve(
        &mut self,
        context: &'a ResolverContext<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderState {
        use self::ResolutionState::*;
        use super::OrderState::*;

        // The Rust debugger doesn't use fmt::Debug when showing values, so we create
        // this string to give us a better way to see which order we're resolving.
        let _order = format!("{}", order);

        // dbg!(order);
        // dbg!(&self.state);
        // dbg!(&self.dependency_chain);

        match self.state.get(order) {
            Some(&Known(order_state)) => order_state,
            Some(&Guessing(order_state)) => {
                // In recursive cases, we accumulate dependencies
                if !self.dependency_chain.contains(&order) {
                    self.dependency_chain.push(order)
                }

                order_state
            }
            None => {
                // checkpoint the resolver and tell it to assume the order fails.
                // get the order state based on that assumption.
                let (resolver_if_fails, if_fails) = self.with_guess(context, order, Fails);

                // If we found no new dependencies then this is a valid resolution!
                // We now snap to the resolver state from the assumption so that we can
                // reuse it in future calculations.
                if resolver_if_fails.dependency_chain.len() == self.dependency_chain.len() {
                    self.state = resolver_if_fails.state;
                    self.set_state(order, Known(if_fails));
                    if_fails
                } else {
                    let next_dep = resolver_if_fails.dependency_chain[self.dependency_chain.len()];

                    // if we depend on some new guess but we haven't hit a cycle,
                    // then we cautiously proceed. We update state to match what we've learned
                    // from the hypothetical and proceed with our guesses.
                    if next_dep != order {
                        self.state = resolver_if_fails.state;
                        self.set_state(order, Guessing(if_fails));
                        self.dependency_chain.push(next_dep);
                        if_fails
                    }
                    // if the next dependency is the one we're already depending on, we're stuck.
                    else {
                        let (_resolver_if_succeeds, if_succeeds) =
                            self.with_guess(context, order, Succeeds);

                        // If there's a paradox but the outcome doesn't depend on this order,
                        // then all we've learned is the state of this one order.
                        if if_fails == if_succeeds {
                            self.set_state(order, Known(if_fails));
                            if_fails
                        } else {
                            let tail_start = self.dependency_chain.len();
                            let tail = &resolver_if_fails.dependency_chain[tail_start..];

                            self.resolve_dependency_cycle(tail);
                            self.resolve(context, order)
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::implicit_hasher)]
impl<'a, A: Adjudicate> From<ResolverState<'a, A>> for HashMap<MappedMainOrder, OrderState> {
    fn from(state: ResolverState<'a, A>) -> Self {
        let mut out_map = HashMap::with_capacity(state.state.len());

        for (order, order_state) in state.state {
            let order_state = match order_state {
                ResolutionState::Known(os) | ResolutionState::Guessing(os) => os,
            };

            out_map.insert(order.clone(), order_state);
        }

        out_map
    }
}

pub fn get_state<'a, A: Adjudicate>(
    r: &ResolverState<'a, A>,
    order: &MappedMainOrder,
) -> Option<OrderState> {
    r.get(order)
}
