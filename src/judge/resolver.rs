use geo::{Map, Region, Border, Province};
use order::{Order, MainCommand};
use super::{MappedMainOrder, OrderState, ResolutionState};

use std::collections::HashMap;
use std::cmp::Ordering;

pub trait Adjudicate : Clone {
    fn adjudicate();
}

fn adjudicate<'a, A : Adjudicate>(
    r : ResolverState<'a, A>,
    context: ResolverContext<'a>, 
    order: &'a MappedMainOrder<'a>,
) {
    use order::MainCommand::*;
    
    match order.command {
        Hold => unimplemented!(),
        Move(ref dest) => unimplemented!(),
        _ => unimplemented!(),
    }
}

/// The immutable inputs for a resolution equation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverContext<'a> {
    pub orders: Vec<MappedMainOrder<'a>>,
    pub world_map: Map<'a>,
}

impl<'a> ResolverContext<'a> {
    pub fn orders_ref(&'a self) -> Vec<&'a MappedMainOrder<'a>> {
        self.orders.iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverState<'a, A : Adjudicate> {
    state: HashMap<&'a MappedMainOrder<'a>, ResolutionState>,
    dependency_chain: Vec<&'a MappedMainOrder<'a>>,
    adjudicator : A,
}

impl<'a, A : Adjudicate> ResolverState<'a, A> {
    
    fn with_state(&mut self, order: &'a MappedMainOrder<'a>, resolution: ResolutionState) -> &Self {
        self.state.insert(order, resolution);
        self
    }
    
    /// When a dependency cycle is detected, attempt to resolve all orders in the cycle.
    fn resolve_dependency_cycle(&mut self, cycle: &[&'a MappedMainOrder<'a>]) {
        use super::OrderState::*;
        use super::ResolutionState::*;
        
        // if every order in the cycle is a move, then this is a circular move
        if cycle.iter().all(|o| o.command.is_move()) {
            for order in cycle {
                self.with_state(order, Known(Succeeds));
            }
        } else {
            // can't resolve convoy paradoxes yet
            unimplemented!()
        }
    }
    
    pub fn resolve(&mut self, context: &'a ResolverContext<'a>, order: &'a MappedMainOrder<'a>) -> OrderState {
        use super::OrderState::*;
        use super::ResolutionState::*;
        
        match self.state.get(order) {
            Some(&Known(order_state)) => order_state,
            Some(&Guessing(order_state)) => {
                
                // In recursive cases, we accumulate dependencies
                if !self.dependency_chain.contains(&order) {
                    self.dependency_chain.push(order)
                }
                
                order_state
            },
            None => {
                // checkpoint the resolver and tell it to assume the order fails.
                let mut resolver_if_fails = self.clone();
                resolver_if_fails.with_state(order, Guessing(Fails));
                
                // get the order state based on that assumption.
                let if_fails = resolver_if_fails.resolve(context, order);
                
                // If we found no new dependencies then this is a valid resolution!
                // We now snap to the resolver state from the assumption so that we can
                // reuse it in future calculations.
                if resolver_if_fails.dependency_chain.len() == self.dependency_chain.len() {
                    self.clone_from(&resolver_if_fails);
                    if_fails
                } else {
                    let next_dep = resolver_if_fails.dependency_chain[self.dependency_chain.len()];
                    if next_dep != order {
                        resolver_if_fails.with_state(order, Guessing(if_fails));
                        self.state = resolver_if_fails.state;
                        self.dependency_chain.push(next_dep);
                        if_fails
                    } else {
                        let mut resolver_if_succeeds = self.clone();
                        resolver_if_succeeds.with_state(order, Guessing(Succeeds));
                        let if_succeeds = resolver_if_succeeds.resolve(context, order);
                        
                        // If there's a paradox but the outcome doesn't depend on this order,
                        // we can gloss over the problem and keep moving.
                        if if_fails == if_succeeds {
                            self.state = resolver_if_fails.state;
                            if_fails
                        } else {
                            let tail_start = self.dependency_chain.len();
                            self.resolve_dependency_cycle(&resolver_if_fails.dependency_chain[tail_start..]);
                            self.resolve(context, order)
                        }
                    }
                }
            }
        }
    }
}