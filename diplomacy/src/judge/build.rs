//! Resolver for build phases.

use super::{MappedBuildOrder, OrderState};
use crate::geo::{Map, ProvinceKey, RegionKey, SupplyCenter};
use crate::order::BuildCommand;
use crate::{Nation, ShortName, UnitType};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

/// The outcome of a build-turn order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl From<OrderOutcome> for OrderState {
    fn from(outcome: OrderOutcome) -> Self {
        if outcome == OrderOutcome::Succeeds {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
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
pub struct ResolverContext<'a, W: WorldState> {
    world: &'a Map,
    home_scs: HashMap<&'a Nation, HashSet<ProvinceKey>>,
    ownerships: HashMap<&'a Nation, i16>,
    last_time: &'a HashMap<ProvinceKey, Nation>,
    this_time: &'a W,
    orders: Vec<&'a MappedBuildOrder>,
}

impl<'a, W: WorldState> ResolverContext<'a, W> {
    /// Create a new context for resolution.
    ///
    /// # First Winter
    /// The first build phase of the game should pass the initial supply center ownerships to
    /// `last_time` to ensure the resolver knows never-since-occupied home SCs belong to their
    /// home power.
    pub fn new(
        world: &'a Map,
        last_time: &'a HashMap<ProvinceKey, Nation>,
        this_time: &'a W,
        orders: Vec<&'a MappedBuildOrder>,
    ) -> Self {
        if last_time.is_empty() {
            panic!("At least one supply center must have been owned by at least one nation. Did you forget to pass the initial world state?");
        }

        let mut home_scs = HashMap::with_capacity(25);
        let mut ownerships = HashMap::new();

        // Figure out who owns what and where nations are allowed to build.
        for province in world.provinces().filter(|p| p.is_supply_center()) {
            if let SupplyCenter::Home(nat) = &province.supply_center {
                home_scs
                    .entry(nat)
                    .or_insert_with(HashSet::new)
                    .insert(province.into());
            }

            let key = ProvinceKey::from(province);
            if let Some(nation) = this_time.occupier(&key).or_else(|| last_time.get(&key)) {
                *ownerships.entry(nation).or_insert(0) += 1;
            }
        }

        Self {
            world,
            home_scs,
            ownerships,
            last_time,
            this_time,
            orders,
        }
    }

    pub fn current_owner(&'a self, province: &ProvinceKey) -> Option<&'a Nation> {
        self.this_time
            .occupier(province)
            .or_else(|| self.last_time.get(province))
    }

    pub fn resolve(&'a self) -> Outcome<'a> {
        Resolution::new(self).resolve(self)
    }
}

struct Resolution<'a> {
    deltas: HashMap<&'a Nation, (BuildCommand, i16)>,
    state: HashMap<&'a MappedBuildOrder, OrderOutcome>,
    civil_disorder: HashSet<(UnitType, RegionKey)>,
    final_units: HashMap<&'a Nation, HashSet<(UnitType, RegionKey)>>,
}

impl<'a> Resolution<'a> {
    pub fn new<W: WorldState>(context: &'a ResolverContext<W>) -> Self {
        let final_units = context
            .this_time
            .nations()
            .into_iter()
            .map(|nation| (nation, context.this_time.units(nation)))
            .collect();

        let deltas = context
            .ownerships
            .iter()
            .filter_map(|(&nation, ownerships)| {
                let adjustment = ownerships - context.this_time.unit_count(nation) as i16;
                match adjustment {
                    0 => None,
                    x if x > 0 => Some((nation, (BuildCommand::Build, x))),
                    x => Some((nation, (BuildCommand::Disband, -x))),
                }
            })
            .collect();

        Resolution {
            deltas,
            state: HashMap::with_capacity(context.orders.len()),
            civil_disorder: HashSet::new(),
            final_units,
        }
    }

    pub fn resolve(mut self, context: &'a ResolverContext<impl WorldState>) -> Outcome<'a> {
        for order in &context.orders {
            self.resolve_order(context, order);
        }

        for (nation, delta) in &mut self.deltas {
            if delta.0 == BuildCommand::Build || delta.1 == 0 {
                continue;
            }

            let usize_delta: usize = delta.1.try_into().unwrap();
            let units = self.final_units.remove(nation).unwrap();

            for unit in units.clone().into_iter().take(usize_delta) {
                self.civil_disorder.insert(unit);
            }

            self.final_units
                .insert(nation, units.into_iter().skip(usize_delta).collect());
        }

        Outcome {
            orders: self.state,
            final_units: self.final_units,
            civil_disorder: self.civil_disorder,
        }
    }

    fn resolve_order(
        &mut self,
        context: &'a ResolverContext<impl WorldState>,
        order: &'a MappedBuildOrder,
    ) -> OrderOutcome {
        use self::OrderOutcome::*;

        // We already know the answer to this one
        if let Some(outcome) = self.state.get(order) {
            return *outcome;
        }

        let Some(delta) = self.deltas.get_mut(&order.nation) else {
            return self.resolve_as(order, RedeploymentProhibited);
        };

        // A power is only allowed to build or disband in a given turn, not both
        if delta.0 != order.command {
            return self.resolve_as(order, RedeploymentProhibited);
        }

        let adjudication = adjudicate(context, order);

        if adjudication != OrderOutcome::Succeeds {
            return self.resolve_as(order, adjudication);
        }

        match order.command {
            BuildCommand::Build => {
                if delta.1 == 0 {
                    return self.resolve_as(order, AllBuildsUsed);
                }

                delta.1 -= 1;

                self.final_units
                    .entry(&order.nation)
                    .or_insert_with(HashSet::new)
                    .insert((order.unit_type, order.region.clone()));

                self.resolve_as(order, Succeeds)
            }
            BuildCommand::Disband => {
                if delta.1 == 0 {
                    return self.resolve_as(order, AllDisbandsUsed);
                }

                delta.1 -= 1;

                self.final_units
                    .entry(&order.nation)
                    .or_insert_with(HashSet::new)
                    .remove(&(order.unit_type, order.region.clone()));

                self.resolve_as(order, Succeeds)
            }
        }
    }

    fn resolve_as(
        &mut self,
        order: &'a MappedBuildOrder,
        resolution: OrderOutcome,
    ) -> OrderOutcome {
        self.state.insert(order, resolution);
        resolution
    }
}

#[derive(Debug, Clone)]
pub struct Outcome<'a> {
    pub orders: HashMap<&'a MappedBuildOrder, OrderOutcome>,
    pub civil_disorder: HashSet<(UnitType, RegionKey)>,
    pub final_units: HashMap<&'a Nation, HashSet<(UnitType, RegionKey)>>,
}

/// Rulebook function for build-phase adjudication. This function does not worry about order quantities,
/// and just focuses on whether or not a given build or disband command is otherwise valid.
fn adjudicate(
    context: &ResolverContext<impl WorldState>,
    order: &MappedBuildOrder,
) -> OrderOutcome {
    use self::OrderOutcome::*;
    let province = order.region.province();

    match order.command {
        BuildCommand::Build => {
            if !context
                .home_scs
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

            let region = if let Some(region) = context.world.find_region(&order.region.short_name())
            {
                region
            } else {
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
