use super::{MappedMainOrder, OrderState, Rulebook};
use crate::geo::{Map, ProvinceKey, RegionKey};
use crate::order::{Command, MainCommand};
use std::collections::HashMap;

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
        ResolverContext {
            world_map,
            orders,
        }
    }

    /// Get a view of the orders.
    pub fn orders(&self) -> &[MappedMainOrder] {
        &self.orders
    }

    pub fn resolve_to_state(&self) -> ResolverState<super::rulebook::Rulebook> {
        let mut rs = ResolverState::with_adjudicator(super::rulebook::Rulebook);
        for order in self.orders() {
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
            _ => print!(""),
        }
    }

    /// Resolve the orders in the context, producing a map of orders to outcomes
    pub fn resolve(self) -> HashMap<MappedMainOrder, OrderState> {
        let rs = self.resolve_to_state();
        let mut out_map = HashMap::with_capacity(self.orders.len());

        for order in &self.orders {
            let order_state = rs.get(&order).expect("All orders should be resolved");
            out_map.insert(order.clone(), order_state);
        }

        out_map
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ResolutionState {
    Guessing(OrderState),
    Known(OrderState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverState<'a, A: Adjudicate> {
    state: HashMap<&'a MappedMainOrder, ResolutionState>,
    dependency_chain: Vec<&'a MappedMainOrder>,
    adjudicator: A,
}

impl<'a, A: Adjudicate> ResolverState<'a, A> {
    /// Create a new resolver for a given rulebook.
    pub fn with_adjudicator(adjudicator: A) -> Self {
        ResolverState {
            state: HashMap::new(),
            dependency_chain: vec![],
            adjudicator,
        }
    }

    fn set_state(&mut self, order: &'a MappedMainOrder, resolution: ResolutionState) {
        self.state.insert(order, resolution);
        // self.report_with_label("Updated");
    }

    /// Get the current projected outcome of an order.
    fn get(&self, order: &MappedMainOrder) -> Option<OrderState> {
        self.state.get(order).map(|rs| rs.into())
    }

    /// Dump the current resolver state to the console. Used in debugging.
    pub fn report_with_label(&self, label: &str) {
        println!("=== STATE @ {} ===", label);

        self.report_dependencies();

        for (order, state) in &self.state {
            println!("  {} {:?}", order, state);
        }
    }

    fn report_dependencies(&self) {
        println!(
            "Deps: [{}]",
            self.dependency_chain
                .iter()
                .map(|o| format!("{}", o))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }

    /// When a dependency cycle is detected, attempt to resolve all orders in the cycle.
    fn resolve_dependency_cycle(&mut self, cycle: &[&'a MappedMainOrder]) {
        use self::ResolutionState::*;
        use super::OrderState::*;

        // if every order in the cycle is a move, then this is a circular move
        if cycle.iter().all(|o| o.command.is_move()) {
            for o in cycle {
                self.set_state(o, Known(Succeeds));
            }
        } else {
            // can't resolve convoy paradoxes yet
            unimplemented!()
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
                let mut resolver_if_fails = self.clone();
                resolver_if_fails.set_state(order, Guessing(Fails));

                // get the order state based on that assumption.
                let if_fails = self
                    .adjudicator
                    .adjudicate(context, &mut resolver_if_fails, order);

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
                        resolver_if_fails.set_state(order, Guessing(if_fails));
                        self.state = resolver_if_fails.state;
                        self.dependency_chain.push(next_dep);
                        if_fails
                    }
                    // if the next dependency is the one we're already depending on, we're stuck.
                    else {
                        let mut resolver_if_succeeds = self.clone();
                        resolver_if_succeeds.set_state(order, Guessing(Succeeds));
                        let if_succeeds = resolver_if_succeeds.resolve(context, order);

                        resolver_if_fails.report_with_label(&format!("If {} fails", order));
                        resolver_if_succeeds.report_with_label(&format!("If {} succeeds", order));

                        // If there's a paradox but the outcome doesn't depend on this order,
                        // then all we've learned is the state of this one order.
                        if if_fails == if_succeeds {
                            self.set_state(order, Known(if_fails));
                            if_fails
                        } else {
                            let tail_start = self.dependency_chain.len();
                            self.resolve_dependency_cycle(
                                &resolver_if_fails.dependency_chain[tail_start..],
                            );
                            self.resolve(context, order)
                        }
                    }
                }
            }
        }
    }
}

pub fn get_state<'a, A: Adjudicate>(
    r: &ResolverState<'a, A>,
    order: &MappedMainOrder,
) -> Option<OrderState> {
    r.get(order)
}
