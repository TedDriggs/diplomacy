use super::{Adjudicate, InvalidOrder, MappedMainOrder, OrderState, Outcome, Rulebook};
use crate::geo::{Map, ProvinceKey, RegionKey};
use crate::order::{Command, MainCommand, Order};
use crate::{Unit, UnitPosition, UnitPositions};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "dependency-graph")]
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

pub struct Submission {
    submitted_orders: Vec<MappedMainOrder>,
    civil_disorder_orders: Vec<MappedMainOrder>,
    starting_state: Option<Box<dyn UnitPositions<RegionKey>>>,
}

impl Submission {
    pub fn new(
        starting_state: impl 'static + UnitPositions<RegionKey>,
        orders: Vec<MappedMainOrder>,
    ) -> Self {
        Submission {
            submitted_orders: orders,
            civil_disorder_orders: vec![],
            starting_state: Some(Box::new(starting_state)),
        }
    }

    pub fn with_inferred_state(
        orders: Vec<MappedMainOrder>
    ) -> Self {
        Submission {
            submitted_orders: orders,
            civil_disorder_orders: vec![],
            starting_state: None,
        }
    }

    pub fn start_adjudication<'a>(&'a mut self, world: &'a Map) -> ResolverContext<'a> {
        let mut invalid_orders = HashMap::new();
        let mut inserted_orders = vec![];

        {
            let positions = self.unit_positions().into_iter().collect::<HashSet<_>>();
            let mut ordered_units = HashSet::new();

            // Reject any invalid orders to prevent them being considered for the rest of
            // the resolution process.
            for order in &self.submitted_orders {
                if !positions.contains(&order.unit_position()) {
                    if self.find_region_occupier(&order.region).is_some() {
                        invalid_orders.insert(order, InvalidOrder::ForeignUnit);
                    } else {
                        invalid_orders.insert(order, InvalidOrder::NoUnit);
                    }
                } else if !ordered_units.insert(order) {
                    invalid_orders.insert(order, InvalidOrder::MultipleToSameUnit);
                }
            }

            // Having rejected invalid orders, we figure out which positions still
            let positions_with_valid_orders = self
                .submitted_orders()
                .filter(|o| !invalid_orders.contains_key(o))
                .map(|o| o.unit_position())
                .collect::<HashSet<_>>();

            // Issue hold orders to any units that don't have orders.
            for position in positions.difference(&positions_with_valid_orders) {
                if !positions_with_valid_orders.contains(&position) {
                    inserted_orders.push(Order::new(
                        position.nation().clone(),
                        position.unit.unit_type(),
                        position.region.clone(),
                        MainCommand::Hold,
                    ))
                }
            }
        }

        self.civil_disorder_orders = inserted_orders;

        let mut context = ResolverContext::new(
            world,
            self.submitted_orders
                .iter()
                .filter(|o| !invalid_orders.contains_key(o))
                .chain(&self.civil_disorder_orders),
        );

        context.invalid_orders = invalid_orders;

        context
    }

    pub fn submitted_orders(&self) -> impl Iterator<Item = &MappedMainOrder> {
        self.submitted_orders.iter()
    }
}

impl UnitPositions<RegionKey> for Submission {
    fn unit_positions(&self) -> Vec<UnitPosition<'_>> {
        if let Some(start) = self.starting_state.as_ref() {
            start.unit_positions()
        } else {
            self.submitted_orders.unit_positions()
        }
    }

    fn find_province_occupier(&self, province: &ProvinceKey) -> Option<UnitPosition<'_>> {
        if let Some(start) = self.starting_state.as_ref() {
            start.find_province_occupier(province)
        } else {
            self.submitted_orders.find_province_occupier(province)
        }
    }

    fn find_region_occupier(&self, region: &RegionKey) -> Option<Unit<'_>> {
        if let Some(start) = self.starting_state.as_ref() {
            start.find_region_occupier(region)
        } else {
            self.submitted_orders.find_region_occupier(region)
        }
    }
}

/// The immutable inputs for a resolution equation.
pub struct ResolverContext<'a> {
    /// Set of orders which were issued during this turn.
    orders: Vec<&'a MappedMainOrder>,

    /// The map against which orders were issued.
    pub world_map: &'a Map,

    invalid_orders: HashMap<&'a MappedMainOrder, InvalidOrder>,
}

impl<'a> ResolverContext<'a> {
    /// Creates a new resolver context for a set of orders on a map.
    pub fn new(world_map: &'a Map, orders: impl IntoIterator<Item = &'a MappedMainOrder>) -> Self {
        ResolverContext {
            world_map,
            orders: orders.into_iter().collect(),
            invalid_orders: HashMap::new(),
        }
    }

    /// Get a view of the orders in the order they were submitted.
    pub fn orders<'b>(&'b self) -> impl 'b + Iterator<Item = &'a MappedMainOrder> where 'a: 'b {
        self.orders.iter().copied()
    }

    /// Resolve the context using the provided adjudicator.
    ///
    /// The adjudicator is responsible for rule questions, while the resolver is responsible for
    /// tracking whether orders are successful. The two are interdependent, calling back and forth
    /// as they work towards a solution.
    pub fn resolve_using<A: Adjudicate>(&'a self, rules: A) -> Outcome<'a, A> {
        let mut rs = ResolverState::with_adjudicator(rules);

        for (order, reason) in &self.invalid_orders {
            rs.invalid_orders.insert(order, *reason);
        }

        for order in self.orders() {
            rs.resolve(&self, order);
        }

        Outcome::new(self, rs)
    }

    /// Resolve the orders in the context using the standard rulebook
    pub fn resolve(&'a self) -> Outcome<'a, Rulebook> {
        self.resolve_using(Rulebook)
    }

    pub fn find_order_to_province(&self, p: &ProvinceKey) -> Option<&'a MappedMainOrder> {
        self.orders().find(|o| &o.region == p)
    }
}

#[allow(clippy::implicit_hasher)]
impl<'a> From<ResolverContext<'a>> for HashMap<MappedMainOrder, OrderState> {
    fn from(rc: ResolverContext<'a>) -> Self {
        rc.resolve().into()
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
pub struct ResolverState<'a, A> {
    state: HashMap<&'a MappedMainOrder, ResolutionState>,
    /// Orders which form part of a paradox. These should only be convoy orders, and will
    /// be treated as hold orders to advance resolution.
    paradoxical_orders: HashSet<&'a MappedMainOrder>,

    /// A dependency chain which adds every order as soon as a guess is made. This is used
    /// to facilitate tracing dependencies rather than for cycle detection.
    #[cfg(feature = "dependency-graph")]
    greedy_chain: Vec<&'a MappedMainOrder>,
    /// A set containing directed edges in a graph of order dependencies.
    #[cfg(feature = "dependency-graph")]
    deps: Rc<RefCell<BTreeSet<(MappedMainOrder, MappedMainOrder)>>>,
    /// The conservative dependency chain used to trigger cycle detection. This contains
    /// guesses that have been visited twice, indicating that a cycle has been found.
    dependency_chain: Vec<&'a MappedMainOrder>,
    adjudicator: A,

    pub(in crate::judge) invalid_orders: HashMap<&'a MappedMainOrder, InvalidOrder>,
}

impl<'a, A> ResolverState<'a, A> {
    pub fn adjudicator(&self) -> &A {
        &self.adjudicator
    }

    fn clear_state(&mut self, order: &MappedMainOrder) {
        self.state.remove(order);
    }

    fn set_state(&mut self, order: &'a MappedMainOrder, resolution: ResolutionState) {
        self.state.insert(order, resolution);
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
}

impl<'a, A: Adjudicate> ResolverState<'a, A> {
    /// Create a new resolver for a given rulebook.
    pub fn with_adjudicator(adjudicator: A) -> Self {
        #[cfg(feature = "dependency-graph")]
        {
            ResolverState {
                state: HashMap::new(),
                deps: Rc::new(RefCell::new(BTreeSet::default())),
                greedy_chain: vec![],
                dependency_chain: vec![],
                paradoxical_orders: HashSet::new(),
                adjudicator,
                invalid_orders: HashMap::new(),
            }
        }

        #[cfg(not(feature = "dependency-graph"))]
        {
            ResolverState {
                state: HashMap::new(),
                dependency_chain: vec![],
                paradoxical_orders: HashSet::new(),
                adjudicator,
                invalid_orders: HashMap::new(),
            }
        }
    }

    /// Create a clone of the resolver state, add a guess at the success or failure
    /// of the given order, then adjudicate the order with the amended resolver.
    ///
    /// This returns the entire guesser because in some cases the calling resolver needs
    /// the dependency chain and the entire state generated during the post-guess adjudication.
    fn with_guess<'b>(
        &self,
        context: &'b ResolverContext<'a>,
        order: &'a MappedMainOrder,
        guess: OrderState,
    ) -> (ResolverState<'a, A>, OrderState) {
        let mut guesser = self.clone();

        #[cfg(feature = "dependency-graph")]
        {
            guesser.greedy_chain.push(order);
        }

        guesser.set_state(order, ResolutionState::Guessing(guess));
        let result = self.adjudicator.adjudicate(context, &mut guesser, order);
        (guesser, result)
    }

    /// Take durable state from another `ResolverState`, leaving behind intermediate calculations.
    ///
    /// In the original C implementation, this wasn't necessary because hypotheticals directly modified
    /// the speculating resolver state. That requires the C implementation to unwind guesses that don't
    /// work out, but allows it to do nothing to leverage successful resolutions.
    ///
    /// The Rust implementation does the opposite, so it needs to "snap to" a resolution state that it
    /// wants to keep.
    fn snap_to(&mut self, other: Self) {
        self.state = other.state;
        self.paradoxical_orders = other.paradoxical_orders;
        self.dependency_chain = other.dependency_chain;
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
        context: &ResolverContext<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderState {
        use self::ResolutionState::*;
        use super::OrderState::*;

        // The Rust debugger doesn't use fmt::Debug when showing values, so we create
        // this string to give us a better way to see which order we're resolving.
        let _order = format!("{}", order);

        #[cfg(feature = "dependency-graph")]
        {
            if !self.greedy_chain.is_empty() {
                self.deps.borrow_mut().insert((
                    self.greedy_chain[self.greedy_chain.len() - 1].clone(),
                    order.clone(),
                ));
            }
        }

        // dbg!(order);
        // dbg!(&self.state);
        // dbg!(&self.dependency_chain);
        // eprintln!("");

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
                let (first_resolver, first_result) = self.with_guess(context, order, Fails);

                // If we found no new dependencies then this is a valid resolution!
                // We now snap to the resolver state from the assumption so that we can
                // reuse it in future calculations.
                if first_resolver.dependency_chain.len() == self.dependency_chain.len() {
                    self.snap_to(first_resolver);
                    self.set_state(order, Known(first_result));
                    first_result
                } else {
                    let next_dep = first_resolver.dependency_chain[self.dependency_chain.len()];

                    // if we depend on some new guess but we haven't hit a cycle,
                    // then we cautiously proceed. We update state to match what we've learned
                    // from the hypothetical and proceed with our guesses.
                    if next_dep != order {
                        self.snap_to(first_resolver);
                        self.set_state(order, Guessing(first_result));
                        self.dependency_chain.push(order);
                        first_result
                    }
                    // if the next dependency is the one we're already depending on, we're stuck.
                    else {
                        let (_second_resolver, second_result) =
                            self.with_guess(context, order, Succeeds);

                        // If there's a paradox but the outcome doesn't depend on this order,
                        // then all we've learned is the state of this one order.
                        if first_result == second_result {
                            self.set_state(order, Known(first_result));
                            first_result
                        } else {
                            let tail_start = self.dependency_chain.len();
                            let tail = &first_resolver.dependency_chain[tail_start..];

                            self.resolve_dependency_cycle(tail);
                            self.resolve(context, order)
                        }
                    }
                }
            }
        }
    }

    /// Get the set of inter-order dependencies encountered while resolving this
    #[cfg(feature = "dependency-graph")]
    pub(crate) fn dependencies(&self) -> BTreeSet<(MappedMainOrder, MappedMainOrder)> {
        self.deps.borrow().clone()
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
