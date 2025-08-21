use crate::geo::{Border, ProvinceKey, RegionKey};
use crate::judge::WillUseConvoy;
use crate::judge::{
    Adjudicate, Context, MappedMainOrder, OrderState, Outcome, Prevent, ResolverState,
    calc::dislodger_of, calc::prevent_results, convoy,
};
use crate::{Unit, UnitPosition, UnitPositions, order::Command};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::iter;

/// Data needed to adjudicate the retreat phase and to present players with useful UI for submitting
/// retreat orders.
pub struct Start<'a> {
    /// A map of dislodged orders to the orders that dislodged them.
    dislodged: HashMap<&'a MappedMainOrder, &'a MappedMainOrder>,
    retreat_destinations: HashMap<UnitPosition<'a>, Destinations<'a>>,
    /// The positions of non-dislodged units at the start of the retreat phase
    pub(in crate::judge::retreat) unit_positions: HashMap<&'a ProvinceKey, UnitPosition<'a>>,
}

impl<'a> Start<'a> {
    /// Initialize a retreat phase, determining which units are dislodged and where they are
    /// able to go based on the outcome of a main phase adjudication.
    pub fn new(outcome: &'a Outcome<'a, impl Adjudicate + WillUseConvoy>) -> Self {
        let mut state = outcome.resolver.clone();
        let dislodged = {
            let mut dislodged = HashMap::new();
            for order in outcome.context.orders() {
                if let Some(dl_ord) = dislodger_of(&outcome.context, &mut state, order) {
                    dislodged.insert(order, dl_ord);
                }
            }

            dislodged
        };

        let interim_positions = non_dislodged_positions(outcome, &dislodged);
        let retreat_destinations = dislodged
            .iter()
            .map(|(dislodged_order, dislodger)| {
                (
                    dislodged_order.unit_position(),
                    outcome
                        .context
                        .world_map
                        .borders_containing(&dislodged_order.region)
                        .into_iter()
                        .filter_map(|border| {
                            Some((
                                border.dest_from(&dislodged_order.region)?,
                                is_valid_retreat_route(
                                    &outcome.context,
                                    &mut state,
                                    &interim_positions,
                                    &dislodged,
                                    dislodged_order,
                                    dislodger,
                                    border,
                                ),
                            ))
                        })
                        .collect(),
                )
            })
            .collect();

        Start {
            dislodged,
            retreat_destinations,
            unit_positions: interim_positions,
        }
    }

    /// Map of dislodged units to the units that dislodged them
    pub fn dislodged(&self) -> &HashMap<&MappedMainOrder, &MappedMainOrder> {
        &self.dislodged
    }

    /// For each dislodged unit, the set of adjacent regions and their suitability status for the
    /// current phase.
    pub fn retreat_destinations(&self) -> &HashMap<UnitPosition<'a>, Destinations<'a>> {
        &self.retreat_destinations
    }

    /// Checks if there are any dislodged units and if any of those units have valid retreat destinations.
    pub fn needs_player_input(&self) -> bool {
        self.retreat_destinations()
            .values()
            .any(|dests| !dests.is_any_available())
    }
}

fn is_valid_retreat_route<'a>(
    main_phase: &'a Context<'a, impl Adjudicate + WillUseConvoy>,
    state: &mut ResolverState<'a>,
    non_dislodged_positions: &impl UnitPositions<RegionKey>,
    dislodged: &HashMap<&MappedMainOrder, &MappedMainOrder>,
    retreater: &MappedMainOrder,
    dislodger: &MappedMainOrder,
    border: &Border,
) -> DestStatus {
    if !border.is_passable_by(retreater.unit_type) {
        return DestStatus::Unreachable;
    }

    let dest = if let Some(dst) = border.dest_from(&retreater.region) {
        dst
    } else {
        return DestStatus::Unreachable;
    };

    // A unit cannot retreat to its dislodger's point of origin unless the dislodger was
    // convoyed to the destination.
    if dest.province() == dislodger.region.province()
        && !convoy::uses_convoy(main_phase, state, dislodger)
    {
        return DestStatus::BlockedByDislodger;
    }

    // A unit cannot retreat to a position that is occupied at the end of the main phase
    if non_dislodged_positions
        .find_province_occupier(dest.province())
        .is_some()
    {
        return DestStatus::Occupied;
    }

    // Dislodged units' do not contest areas during the retreat phase
    let applicable_prevents = prevent_results(main_phase, state, dest.province())
        .into_iter()
        .any(|prevent| match prevent {
            Prevent::Prevents(ord, _) => !dislodged.contains_key(ord),
            _ => false,
        });

    // A unit cannot retreat to a position that was contested in the main phase, even
    // if the province is vacant due to a stalemate
    if applicable_prevents {
        DestStatus::Contested
    } else {
        DestStatus::Available
    }
}

/// Possible destinations a unit could move to during the retreat phase, along with the
/// status of each destination.
pub struct Destinations<'a> {
    regions: BTreeMap<&'a RegionKey, DestStatus>,
}

impl Destinations<'_> {
    /// Get the destination status of a particular region.
    pub fn get(&self, region: &RegionKey) -> DestStatus {
        self.regions
            .get(region)
            .copied()
            .unwrap_or(DestStatus::Unreachable)
    }

    /// Check if any region is available to the unit as a move destination. If not, the unit
    /// will have no choice but to disband.
    fn is_any_available(&self) -> bool {
        self.regions
            .values()
            .any(|&status| status == DestStatus::Available)
    }

    /// Get the regions that are viable retreat destinations.
    pub fn available(&self) -> BTreeSet<&RegionKey> {
        self.regions
            .iter()
            .filter_map(|(&region, &status)| {
                if status == DestStatus::Available {
                    Some(region)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<'a> iter::FromIterator<(&'a RegionKey, DestStatus)> for Destinations<'a> {
    fn from_iter<I: iter::IntoIterator<Item = (&'a RegionKey, DestStatus)>>(iterator: I) -> Self {
        Self {
            regions: iterator.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DestStatus {
    /// The region is a viable retreat destination.
    Available,
    /// The retreating unit cannot reach the destination, due to the lack of a passable border.
    ///
    /// This status covers three cases:
    ///
    /// 1. There is a border, but the unit cannot cross it due to terrain incompatibility.
    /// 2. There is no border with the specified destination; it is not adjacent to the retreating unit.
    /// 3. The specified destination does not exist on the map.
    Unreachable,
    /// The unit that dislodged the retreating unit launched a direct assault from this region's
    /// parent province.
    BlockedByDislodger,
    /// There is a unit in the destination province.
    Occupied,
    /// The region is vacant, but during the main phase the province was the site of a stalemate.
    /// Units cannot retreat into stalemate territory.
    Contested,
}

/// The state of the world between the main phase and retreat phases of a season, ignoring
/// dislodged units.
///
/// To calculate valid retreat destinations, it's necessary to know which provinces are
/// occupied when the retreats take place. It's therefore useful to compute positions
/// based on order outcomes, so that moved units have vacated their old provinces and
/// fill their new ones.
///
/// This approach creates a problem, however: Where are dislodged units? They haven't retreated
/// yet, so logically it seems that they're in their old positions. However, reporting them
/// there would mean multiple units are concurrently in the same province, which might
/// create some unforeseen weirdness. To mitigate this, we ignore those units.
fn non_dislodged_positions<'a, A>(
    outcome: &Outcome<'a, A>,
    dislodged: &HashMap<&MappedMainOrder, &MappedMainOrder>,
) -> HashMap<&'a ProvinceKey, UnitPosition<'a>> {
    let mut positions = HashMap::new();
    for (order, result) in &outcome.orders {
        if dislodged.contains_key(order) {
            continue;
        }

        if order.is_move() && OrderState::from(result) == OrderState::Succeeds {
            let new_position = order.move_dest().unwrap();
            positions.insert(
                new_position.province(),
                UnitPosition::new(Unit::from(*order), new_position),
            );
        } else {
            positions.insert(order.region.province(), UnitPosition::from(*order));
        }
    }

    positions
}
