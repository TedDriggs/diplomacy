//! Resolver for build phases.

use super::{MappedBuildOrder, OrderState};
use crate::geo::{Map, ProvinceKey, RegionKey, SupplyCenter};
use crate::order::BuildCommand;
use crate::{Nation, ShortName, UnitType};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

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
pub struct Context<'a, W: WorldState> {
    world: &'a Map,
    home_scs: HashMap<&'a Nation, HashSet<ProvinceKey>>,
    ownerships: HashMap<&'a Nation, HashSet<ProvinceKey>>,
    last_time: &'a HashMap<ProvinceKey, Nation>,
    this_time: &'a W,
    orders: Vec<MappedBuildOrder>,
}

impl<'a, W: WorldState> Context<'a, W> {
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
        orders: impl IntoIterator<Item = MappedBuildOrder>,
    ) -> Self {
        if last_time.is_empty() {
            panic!("At least one supply center must have been owned by at least one nation. Did you forget to pass the initial world state?");
        }

        let mut home_scs = HashMap::with_capacity(25);
        let mut ownerships = HashMap::<&Nation, HashSet<ProvinceKey>>::new();

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
                ownerships.entry(nation).or_default().insert(key);
            }
        }

        Self {
            world,
            home_scs,
            ownerships,
            last_time,
            this_time,
            orders: orders.into_iter().collect(),
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
    pub fn new<W: WorldState>(context: &'a Context<W>) -> Self {
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
                let adjustment = i16::from(ownerships.len() as u8)
                    - i16::from(context.this_time.unit_count(nation));
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

    pub fn resolve(mut self, context: &'a Context<impl WorldState>) -> Outcome<'a> {
        for order in &context.orders {
            self.resolve_order(context, order);
        }

        self.compute_mandatory_disbands(context);

        Outcome {
            orders: self.state,
            final_units: self.final_units,
            civil_disorder: self.civil_disorder,
        }
    }

    fn resolve_order(
        &mut self,
        context: &'a Context<impl WorldState>,
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
                    .or_default()
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
                    .or_default()
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

    /// Balance unit populations with national supply centers by forcibly disbanding
    /// excess units. This will only have an effect if a nation did not issue enough
    /// disband orders to cover their supply center losses.
    fn compute_mandatory_disbands(&mut self, context: &'a Context<impl WorldState>) {
        let world_graph = context.world.to_graph();

        for (nation, delta) in &mut self.deltas {
            if delta.0 == BuildCommand::Build || delta.1 == 0 {
                continue;
            }

            let usize_delta: usize = delta.1.try_into().unwrap();
            let units = self.final_units.remove(nation).unwrap();

            // Per 2023 rulebook, units disband based on distance from the nation's owned
            // supply centers (earlier editions had it based on distance from the home supply centers)
            let Some(owned_scs) = context.ownerships.get(*nation) else {
                // If there are no owned supply centers, all units disband
                self.civil_disorder.extend(units);
                continue;
            };

            // Get all regions in the owned supply centers. The rules require checking
            // distance to all coasts, so province precision is insufficient.
            let owned_sc_regions = context
                .world
                .regions()
                .filter(|r| owned_scs.contains(r.province()))
                .collect::<Vec<_>>();

            let mut units_by_disband_priority = units
                .into_iter()
                .map(|unit| {
                    let unit_region = context
                        .world
                        .find_region(&unit.1.to_string())
                        .unwrap_or_else(|| {
                            panic!("Unit location {} should exist in world", unit.1)
                        });

                    if owned_sc_regions.contains(&unit_region) {
                        return (unit, 0);
                    }

                    let min_distance = owned_sc_regions
                        .iter()
                        .filter_map(|sc_region| {
                            // Using dijkstra because there isn't an obvious way to estimate
                            // distance for A*, and the graph size is so small that the efficiency
                            // difference shouldn't matter.
                            petgraph::algo::dijkstra(
                                &world_graph,
                                unit_region,
                                Some(sc_region),
                                // Per DATC test 6.J.6, terrain is ingored in this
                                // calculation. This is a deviation from older versions
                                // of the DATC, which stated that sea units could only
                                // consider sea distances
                                |_| 1,
                            )
                            .get(sc_region)
                            .copied()
                        })
                        .min()
                        .unwrap_or(i32::MAX);

                    (unit, min_distance)
                })
                .collect::<Vec<_>>();

            // Per the DATC, units are sorted by distance from an owned SC. Equidistant fleets
            // are disbanded before armies, and sorting of units within the same type is done
            // alphabetically.
            units_by_disband_priority.sort_by(|a, b| {
                // Distance from nearest owned supply center, descending
                b.1.cmp(&a.1)
                    // when equidistant, disband fleets before armies
                    .then(b.0 .0.cmp(&a.0 .0))
                    // when units are same type and equidistant, disband in alphabetical order
                    .then_with(|| a.0 .1.cmp(&b.0 .1))
            });

            // Add units from the disband queue to the civil disorder output
            self.civil_disorder
                .extend(units_by_disband_priority.drain(0..usize_delta).map(|v| v.0));

            // Add the remaining units to the map of units that survive the turn.
            self.final_units.insert(
                nation,
                units_by_disband_priority.into_iter().map(|v| v.0).collect(),
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct Outcome<'a> {
    pub orders: HashMap<&'a MappedBuildOrder, OrderOutcome>,
    pub civil_disorder: HashSet<(UnitType, RegionKey)>,
    pub final_units: HashMap<&'a Nation, HashSet<(UnitType, RegionKey)>>,
}

impl<'a> Outcome<'a> {
    pub fn get(&self, order: &MappedBuildOrder) -> Option<&OrderOutcome> {
        self.orders.get(order)
    }

    pub fn order_outcomes(&self) -> impl Iterator<Item = (&MappedBuildOrder, &OrderOutcome)> {
        self.orders.iter().map(|(k, v)| (*k, v))
    }
}

/// Rulebook function for build-phase adjudication. This function does not worry about order quantities,
/// and just focuses on whether or not a given build or disband command is otherwise valid.
fn adjudicate(context: &Context<impl WorldState>, order: &MappedBuildOrder) -> OrderOutcome {
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
