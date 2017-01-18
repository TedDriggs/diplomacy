use std::collections::HashMap;

use geo::{Map, RegionKey};
use judge::OrderState;
use order::{Order, RetreatCommand};

pub type MappedRetreatOrder = Order<RegionKey, RetreatCommand<RegionKey>>;

pub trait RetreatRule: Clone {
    fn adjudicate<'a>(&self,
                      context: &'a ResolverContext<'a>,
                      resolver: &mut RetreatState<'a, Self>,
                      order: &'a MappedRetreatOrder)
                      -> OrderState;
}

/// The immutable inputs for a resolution equation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverContext<'a> {
    pub orders: Vec<MappedRetreatOrder>,
    pub world_map: &'a Map,
}

impl<'a> ResolverContext<'a> {
    /// Creates a new resolver context for a set of orders on a map.
    pub fn new(world_map: &'a Map, orders: Vec<MappedRetreatOrder>) -> Self {
        ResolverContext {
            world_map: world_map,
            orders: orders,
        }
    }

    /// Get a view of the orders.
    pub fn orders_ref(&'a self) -> Vec<&'a MappedRetreatOrder> {
        self.orders.iter().collect()
    }

    fn resolve_orders(&self) -> Self {
        unimplemented!()
    }

    /// Resolve the orders in the context, producing a map of orders to outcomes
    pub fn resolve(self) -> HashMap<MappedRetreatOrder, OrderState> {
        let rs = self.resolve_orders();
        let out_map = HashMap::with_capacity(self.orders.len());

        out_map
    }
}

pub struct RetreatState<'a, R: RetreatRule> {
    state: HashMap<&'a MappedRetreatOrder, ResolutionState>,
    rules: R,
}

impl<'a, R: RetreatRule> RetreatState<'a, R> {
    pub fn with_rulebook(rules: R) -> Self {
        RetreatState {
            state: HashMap::new(),
            rules: rules,
        }
    }
    
    pub fn get_state(&self, order: &MappedRetreatOrder) -> Option<OrderState> {
        self.state.get(order).map(|rs| rs.into())
    }

    pub fn resolve(&mut self,
                   context: &'a ResolverContext<'a>,
                   order: &'a MappedRetreatOrder)
                   -> OrderState {
        unimplemented!()
    }
}