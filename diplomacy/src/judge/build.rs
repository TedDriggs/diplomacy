//! Resolver for build phases.

use super::{MappedBuildOrder, OrderState};
use crate::geo::{Map, ProvinceKey, RegionKey, SupplyCenter};
use crate::order::BuildCommand;
use crate::{Nation, ShortName, Unit, UnitPosition, UnitType};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// The outcome of a build-turn order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderOutcome {
    /// The build or disband order was successful, resulting in a change in units
    /// in the world.
    Succeeds,
    /// A nation cannot issue "build" and "disband" commands in the same turn,
    /// as this would constitute an illegal teleportation of power from the
    /// disbanding region to the building region.
    RedeploymentProhibited,
    /// The build command was to a province where the issuing nation cannot build.
    InvalidProvince,
    /// The build command was to a home SC for the issuing power, but another
    /// power currently controls it.
    ForeignControlled,
    /// Build failed because the target province already has a friendly unit in it.
    OccupiedProvince,
    /// The build command is to a region that is qualified for
    InvalidTerrain,
    /// Disband failed because no unit exists at that location.
    DisbandingNonexistentUnit,
    /// Disband failed because the issuing power does not control the unit at that location.
    DisbandingForeignUnit,
    /// The issuing nation has already had as many successful builds as they are allowed.
    AllBuildsUsed,
    /// The issuing nation has already had as many successful disbands as they are allowed.
    AllDisbandsUsed,
}

impl From<&'_ OrderOutcome> for OrderState {
    fn from(value: &'_ OrderOutcome) -> Self {
        if value == &OrderOutcome::Succeeds {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

impl From<OrderOutcome> for OrderState {
    fn from(outcome: OrderOutcome) -> Self {
        (&outcome).into()
    }
}

/// Provider for the resolver to get state about the game world that it needs to successfully
/// judge a build phase.
pub trait WorldState {
    /// Get the set of nations in the game. This must include nations that issued no
    /// orders this turn, and may include nations that have no units if those units
    /// are entitled to build.
    fn nations(&self) -> HashSet<&Nation>;
    /// Get the nation with a unit _currently in_ the specified province. This should
    /// return `None` if the province is vacant, even if it's controlled by a nation.
    fn occupier(&self, province: &ProvinceKey) -> Option<&Nation>;
    /// Get the number of units owned by the specified nation
    fn unit_count(&self, nation: &Nation) -> u8;
    /// Get the units owned by the specified nation
    fn units(&self, nation: &Nation) -> HashSet<(UnitType, RegionKey)>;
}

/// The immutable pieces of a build-phase order resolution
pub struct Submission<'a, W: WorldState> {
    world_map: &'a Map,
    this_time: &'a W,
    last_time: &'a HashMap<ProvinceKey, Nation>,
    orders: Vec<MappedBuildOrder>,
}

impl<'a, W: WorldState> Submission<'a, W> {
    /// Returns a new submission for build-phase resolution.
    ///
    /// # First Winter
    /// The first build phase of the game should pass the initial supply center ownerships to
    /// `last_time` to ensure the resolver knows never-since-occupied home SCs belong to their
    /// home power.
    pub fn new(
        world: &'a Map,
        last_time: &'a HashMap<ProvinceKey, Nation>,
        this_time: &'a W,
        orders: impl IntoIterator<Item = MappedBuildOrder>,
    ) -> Self {
        if last_time.is_empty() {
            panic!("At least one supply center must have been owned by at least one nation. Did you forget to pass the initial world state?");
        }

        Self {
            world_map: world,
            last_time,
            this_time,
            orders: orders.into_iter().collect(),
        }
    }

    pub fn adjudicate<A: Adjudicate<'a>>(&'a self, rules: A) -> Outcome<'a> {
        Context::new(
            self.world_map,
            self.last_time,
            self.this_time,
            rules,
            self.orders.iter().collect(),
        )
        .resolve()
    }
}

/// The immutable parts of build-phase adjudication.
///
/// Note that several parts of build-phase adjudication are immutable, but
/// adjudicator-specific, and therefore are exposed via [`Adjudicate::Scratch`]
/// rather than being accessed from the context.
pub struct Context<'a, W, A> {
    pub world_map: &'a Map,
    /// The adjudicator being used.
    pub rules: A,
    /// The current locations of units at the start of the build phase.
    pub this_time: &'a W,
    /// Map of province ownerships to nations at the end of the last build phase.
    ///
    /// Unoccupied provinces stay under the control of their previous owner, so
    /// this is used when determining how many units a nation can sustain, and
    /// where a nation can build.
    pub last_time: &'a HashMap<ProvinceKey, Nation>,
    orders: Vec<&'a MappedBuildOrder>,
}

impl<'a, W: WorldState, A> Context<'a, W, A> {
    /// Returns the owner of the province at the end of this build phase.
    pub fn current_owner(&'a self, province: &ProvinceKey) -> Option<&'a Nation> {
        self.this_time
            .occupier(province)
            .or_else(|| self.last_time.get(province))
    }
}

impl<'a, W: WorldState, A: Adjudicate<'a>> Context<'a, W, A> {
    fn new(
        world_map: &'a Map,
        last_time: &'a HashMap<ProvinceKey, Nation>,
        this_time: &'a W,
        rules: A,
        orders: Vec<&'a MappedBuildOrder>,
    ) -> Self {
        Context {
            world_map,
            this_time,
            last_time,
            rules,
            orders,
        }
    }

    fn resolve(self) -> Outcome<'a> {
        ResolverState::new(&self).resolve(self)
    }
}

/// Mutable state for adjudicating a build phase.
///
/// This is created when calling [`Submission::adjudicate`] and passed to the [adjudicator](`Adjudicate`).
pub struct ResolverState<'a, Scratch> {
    /// Scratch state for the adjudicator. See [`Adjudicate::Scratch`] for more information.
    pub scratch: Scratch,
    state: HashMap<&'a MappedBuildOrder, OrderOutcome>,
    /// The set of units that were disbanded due to civil disorder.
    pub civil_disorder: HashSet<UnitPosition<'a, RegionKey>>,
    /// The final units after adjudication, grouped by nation.
    pub final_units: HashMap<&'a Nation, HashSet<(UnitType, RegionKey)>>,
}

impl<'a, S> ResolverState<'a, S> {
    fn new<W: WorldState, A: Adjudicate<'a, Scratch = S>>(context: &Context<'a, W, A>) -> Self {
        Self {
            scratch: context.rules.initialize(context),
            state: HashMap::with_capacity(context.orders.len()),
            civil_disorder: HashSet::new(),
            final_units: context
                .this_time
                .nations()
                .into_iter()
                .map(|nation| (nation, context.this_time.units(nation)))
                .collect(),
        }
    }

    fn resolve<W: WorldState, A: Adjudicate<'a, Scratch = S>>(
        mut self,
        context: Context<'a, W, A>,
    ) -> Outcome<'a> {
        for order in &context.orders {
            self.resolve_order(&context, order);
        }

        context.rules.civil_disorder(&context, &mut self);

        Outcome {
            civil_disorder: self.civil_disorder,
            final_units: self.final_units,
            orders: self.state,
        }
    }

    fn resolve_order<W: WorldState, A: Adjudicate<'a, Scratch = S>>(
        &mut self,
        context: &Context<'a, W, A>,
        order: &'a MappedBuildOrder,
    ) -> OrderOutcome {
        // We already know the answer to this one
        if let Some(outcome) = self.state.get(order) {
            return *outcome;
        }

        let adjudication = context.rules.explain(context, self, order);
        self.state.insert(order, adjudication);
        adjudication
    }
}

/// The outcome of a build-phase adjudication.
#[derive(Debug, Clone)]
pub struct Outcome<'a> {
    orders: HashMap<&'a MappedBuildOrder, OrderOutcome>,
    civil_disorder: HashSet<UnitPosition<'a, RegionKey>>,
    final_units: HashMap<&'a Nation, HashSet<(UnitType, RegionKey)>>,
}

impl<'a> Outcome<'a> {
    /// Returns the outcome of an order.
    pub fn get(&self, order: &MappedBuildOrder) -> Option<&OrderOutcome> {
        self.orders.get(order)
    }

    pub fn order_outcomes(&self) -> impl Iterator<Item = (&MappedBuildOrder, &OrderOutcome)> {
        self.orders.iter().map(|(k, v)| (*k, v))
    }

    /// Returns an iterator over the final units grouped by nation.
    pub fn final_units_by_nation<'s>(
        &'s self,
    ) -> impl Iterator<Item = (&'a Nation, &'s HashSet<(UnitType, RegionKey)>)> {
        self.final_units.iter().map(|(k, v)| (*k, v))
    }

    /// Returns an iterator over the units that exist after resolution.
    pub fn to_final_unit_positions<'s>(
        &'s self,
    ) -> impl 's + Iterator<Item = UnitPosition<'static, RegionKey>> {
        self.final_units.iter().flat_map(|(nation, units)| {
            units.iter().map(|(unit_type, region)| {
                UnitPosition::new(
                    Unit::new(Cow::Owned((*nation).clone()), *unit_type),
                    region.clone(),
                )
            })
        })
    }

    /// Returns the set of units that were disbanded due to civil disorder.
    pub fn to_civil_disorder(&self) -> HashSet<UnitPosition<'static, RegionKey>> {
        self.civil_disorder
            .iter()
            .map(|pos| {
                UnitPosition::new(
                    Unit::new(Cow::Owned(pos.unit.nation().clone()), pos.unit.unit_type()),
                    pos.region.clone(),
                )
            })
            .collect()
    }
}

/// Adjudicator for a build-phase.
///
/// For implementation efficiency, this trait allows for a "scratch" state that will be stored
/// in the resolver.
pub trait Adjudicate<'a>: Sized {
    /// Working storage for this adjudicator. This is used to store initial calculations
    /// and mutable state that is needed to adjudicate orders that are not independent.
    type Scratch;

    /// Initialize the scratch state for this adjudicator. This is called once per resolution.
    fn initialize<W: WorldState>(&self, ctx: &Context<'a, W, Self>) -> Self::Scratch;

    /// Determines and returns the success of an order.
    ///
    /// Calling this MUST leave the `resolver` in the same state as if [`Adjudicate::explain`] was called.
    ///
    /// Adjudicators MAY customize this method to be more memory-efficient by avoiding allocation of explanatory data.
    fn adjudicate<W: WorldState>(
        &self,
        context: &Context<'a, W, Self>,
        resolver: &mut ResolverState<'a, Self::Scratch>,
        order: &'a MappedBuildOrder,
    ) -> OrderState {
        self.explain(context, resolver, order).into()
    }

    /// Determines and returns the outcome of an order.
    ///
    /// The outcome contains enough information to determine both _whether_ the order succeeds or fails and _why_ the order succeeds or fails.
    fn explain<W: WorldState>(
        &self,
        context: &Context<'a, W, Self>,
        resolver: &mut ResolverState<'a, Self::Scratch>,
        order: &'a MappedBuildOrder,
    ) -> OrderOutcome;

    /// Modifies the resolver after all orders have been adjudicated to enforce civil disorder rules.
    fn civil_disorder<W: WorldState>(
        &self,
        context: &Context<'a, W, Self>,
        resolver: &mut ResolverState<'a, Self::Scratch>,
    );
}

/// Rulebook function for build-phase adjudication. This function does not worry about order quantities,
/// and just focuses on whether or not a given build or disband command is otherwise valid.
pub(crate) fn adjudicate<A>(
    context: &Context<impl WorldState, A>,
    home_scs: &HashMap<&Nation, HashSet<ProvinceKey>>,
    order: &MappedBuildOrder,
) -> OrderOutcome {
    use self::OrderOutcome::*;
    let province = order.region.province();

    match order.command {
        BuildCommand::Build => {
            if !home_scs
                .get(&order.nation)
                .expect("Every nation should have home SCs")
                .contains(province)
            {
                return InvalidProvince;
            }

            if Some(&order.nation) != context.current_owner(province) {
                return ForeignControlled;
            }

            if context.this_time.occupier(province).is_some() {
                return OccupiedProvince;
            }

            let Some(region) = context.world_map.find_region(&order.region.short_name()) else {
                return InvalidProvince;
            };

            if !order.unit_type.can_occupy(region.terrain()) {
                return InvalidTerrain;
            }

            Succeeds
        }
        BuildCommand::Disband => match context.this_time.occupier(province) {
            None => DisbandingNonexistentUnit,
            Some(nation) if &order.nation != nation => DisbandingForeignUnit,
            _ => Succeeds,
        },
    }
}

/// Convert a map into an initial ownership state where each nation owns their home
/// supply centers and all other supply centers are unowned.
pub fn to_initial_ownerships(map: &Map) -> HashMap<ProvinceKey, Nation> {
    map.provinces()
        .filter_map(|province| {
            if let SupplyCenter::Home(nat) = &province.supply_center {
                Some((province.into(), nat.clone()))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::to_initial_ownerships;
    use crate::geo::{standard_map, ProvinceKey};
    use crate::Nation;

    #[test]
    fn to_initial_ownerships_for_standard_map() {
        let ownerships = to_initial_ownerships(standard_map());

        assert_eq!(
            Some(&Nation::from("AUS")),
            ownerships.get(&ProvinceKey::from("bud"))
        );

        assert_eq!(None, ownerships.get(&ProvinceKey::from("bel")));
    }
}
