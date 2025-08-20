use super::calc::{dislodger_of, is_head_to_head, max_prevent_result, path_exists};
use super::convoy::ConvoyOutcome;
use super::resolver::{Context, ResolverState};
use super::support::{self, SupportOutcome};
use super::{Adjudicate, MappedMainOrder, OrderOutcome, OrderState};
use crate::geo::Terrain;
use crate::judge::strength::Strength;
use crate::judge::WillUseConvoy;
use crate::order::{Command, MainCommand};
use crate::ShortName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConvoyUsePolicy {
    /// Any move order that can be convoyed will use a convoy.
    Any,
    /// A convoy will be used if the order explicitly mandates it, or if the convoy route
    /// includes an order from the same nation as the convoyed unit.
    IncludesSameCountry,
    /// A convoy will only be used if the order explicitly mandates it.
    MustBeExplicit,
}

impl WillUseConvoy for ConvoyUsePolicy {
    fn will_use_convoy(&self, order: &MappedMainOrder, route: &[&MappedMainOrder]) -> bool {
        let MainCommand::Move(cmd) = &order.command else {
            return false;
        };

        // If the order talks about convoys and does not mandate use of a convoy,
        // then never use one.
        if cmd.mentions_convoy() && !cmd.mandates_convoy() {
            return false;
        }

        match self {
            ConvoyUsePolicy::Any => true,
            ConvoyUsePolicy::IncludesSameCountry => {
                cmd.mandates_convoy() || route.iter().any(|o| o.nation == order.nation)
            }
            ConvoyUsePolicy::MustBeExplicit => cmd.mandates_convoy(),
        }
    }
}

/// The standard Diplomacy rules.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Rulebook {
    convoy_policy: ConvoyUsePolicy,
}

impl Rulebook {
    /// Returns the 1971 edition of the standard rules.
    pub fn edition_1971() -> Self {
        Self {
            convoy_policy: ConvoyUsePolicy::Any,
        }
    }

    /// Returns the 1982 edition of the standard rules.
    pub fn edition_1982() -> Self {
        Self {
            convoy_policy: ConvoyUsePolicy::IncludesSameCountry,
        }
    }

    /// Returns the 2023 edition of the standard rules.
    pub fn edition_2023() -> Self {
        Self {
            convoy_policy: ConvoyUsePolicy::IncludesSameCountry,
        }
    }

    pub fn dptg() -> Self {
        Self {
            convoy_policy: ConvoyUsePolicy::MustBeExplicit,
        }
    }
}

impl Rulebook {
    /// Apply rules to determine hold outcome.
    fn adjudicate_hold<'a>(
        ctx: &Context<'a, impl Adjudicate>,
        rslv: &mut ResolverState<'a>,
        ord: &'a MappedMainOrder,
    ) -> HoldOutcome<&'a MappedMainOrder> {
        dislodger_of(ctx, rslv, ord)
            .map(HoldOutcome::Dislodged)
            .unwrap_or(HoldOutcome::Succeeds)
    }

    /// Apply rules to determine move outcome.
    fn adjudicate_move<'a>(
        ctx: &Context<'a, impl Adjudicate + WillUseConvoy>,
        rslv: &mut ResolverState<'a>,
        ord: &'a MappedMainOrder,
    ) -> AttackOutcome<&'a MappedMainOrder> {
        if ord.command.move_dest() == Some(&ord.region) {
            AttackOutcome::MoveToSelf
        } else if !path_exists(ctx, rslv, ord) {
            AttackOutcome::NoPath
        } else if ord.command.is_move() {
            let mut atk_supports = support::find_for(ctx, rslv, ord);
            let mut atk_strength = 1 + atk_supports.len();
            let prevent = max_prevent_result(ctx, rslv, ord);

            // if the attack cannot overcome the prevent even in the best case,
            // there isn't any point in continuing the calculation and we should
            // immediately report the failure. This avoids breaking test case
            // 6.C.03 Three army circular movement bounces.
            if atk_strength <= prevent.strength() {
                AttackOutcome::Prevented(prevent.unwrap().unwrap_order())
            } else {
                if let Some(occupier) =
                    ctx.find_order_to_province(ord.command.move_dest().unwrap().into())
                {
                    // A head-to-head battle occurs when two units have mirrored move orders and
                    // no convoy is available to help one of the units move around the other.
                    let is_head_to_head = is_head_to_head(ctx, rslv, ord, occupier);

                    // Separately compute the resistance and the head-to-head strengths,
                    // to take into account nuances about which support orders participate
                    // in which stages.
                    let (resistance, h2h) = if !occupier.command.is_move() || is_head_to_head {
                        // DEFEND and HOLD strengths include supports that may seek to thwart
                        // other orders from the same nation.

                        // Example:
                        // France:
                        // A Belgium Supports A Burgundy - Ruhr
                        // A Holland Supports A Burgundy - Ruhr
                        // A Burgundy - Ruhr
                        // A Munich Supports A Ruhr - Burgundy
                        // A Marseilles - Burgundy

                        // Germany:
                        // A Ruhr - Burgundy
                        // In this example the French army in Munich supports the move of the German army
                        // in Ruhr instead of the French army in Burgundy. This makes that the ATTACK STRENGTH,
                        // the PREVENT STRENGTH and the DEFEND STRENGTH of the German army in Ruhr are all different.
                        // The ATTACK STRENGTH is one, because the French support should not be counted for the attack.
                        // The PREVENT STRENGTH is zero, because it is dislodged by the French army in Burgundy
                        // and therefore it can not prevent the army in Marseilles to go to Burgundy. However, the
                        // DEFEND STRENGTH contains all supports and is therefore two. Still this DEFEND STRENGTH
                        // is insufficient in the head to head battle, since the French army in Burgundy has an
                        // ATTACK STRENGTH of three.
                        let mut resisting_supports = support::find_for(ctx, rslv, occupier);

                        let resistance = 1 + resisting_supports.len();

                        if is_head_to_head {
                            // Make sure the head-to-head opponent is not getting head-to-head support that would result in
                            // `ord` losing from `ord`'s own nation.
                            resisting_supports.retain(|support| support.nation != ord.nation);
                            (resistance, 1 + resisting_supports.len())
                        } else {
                            (resistance, 0)
                        }
                    }
                    // failed exits resist with strength 1 (the unit trapped in the province)
                    else if rslv.resolve(ctx, occupier) == OrderState::Fails {
                        (1, 0)
                    // successful exits mount no resistance
                    } else {
                        (0, 0)
                    };

                    // A unit can not dislodge a unit of the same player.
                    // Head-to-head, failed exit, and hold cases all collapse in friendly fire.
                    if resistance > 0 && ord.nation == occupier.nation {
                        return AttackOutcome::FriendlyFire;
                    } else if resistance > 0 {
                        let self_defend_strength = atk_strength;

                        // Supports to a foreign unit can not be used to dislodge an own unit.
                        // Therefore, we remove any move supports from the nation whose unit
                        // is resisting the move.
                        atk_supports.retain(|sup| sup.nation != occupier.nation);
                        atk_strength = 1 + atk_supports.len();

                        // Re-check if the attack strength is sufficient to overcome prevent
                        // strength now that friendly-fire support is ignored; see 6.E.7
                        if atk_strength <= prevent.strength() {
                            return AttackOutcome::Prevented(prevent.unwrap().unwrap_order());
                        }

                        // Only lose a head-to-head if the head-to-head opponent's attack strength
                        // is higher than our defend strength.
                        if self_defend_strength < h2h {
                            return AttackOutcome::LostHeadToHead;
                        }

                        if atk_strength <= resistance {
                            return AttackOutcome::OccupierDefended;
                        }
                    }
                }

                AttackOutcome::Succeeds
            }
        } else {
            panic!("Don't try to adjudicate non-moves as moves");
        }
    }

    fn adjudicate_support<'a>(
        ctx: &Context<'a, impl Adjudicate + WillUseConvoy>,
        rslv: &mut ResolverState<'a>,
        ord: &'a MappedMainOrder,
    ) -> SupportOutcome<&'a MappedMainOrder> {
        if support::is_supporting_self(ord) {
            SupportOutcome::SupportingSelf
        } else if !support::can_reach(ctx.world_map, ord) {
            SupportOutcome::CantReach
        } else {
            match support::find_cutting_order(ctx, rslv, ord) {
                Some(cutter) => SupportOutcome::CutBy(cutter),
                None => SupportOutcome::NotDisrupted,
            }
        }
    }

    fn adjudicate_convoy<'a>(
        ctx: &Context<'a, impl Adjudicate + WillUseConvoy>,
        rslv: &mut ResolverState<'a>,
        ord: &'a MappedMainOrder,
    ) -> ConvoyOutcome<&'a MappedMainOrder> {
        // Test case 6.F.1: Fleets cannot convoy in coastal areas
        //
        // Note: We explicitly check that "coast" is none because explicit-coast
        // regions are marked as being 'sea' to prevent armies from occupying them,
        // but are not valid locations for convoys to operate.
        let is_at_sea = ord.region.coast().is_none()
            && ctx
                .world_map
                .find_region(&ord.region.short_name())
                .map(|r| r.terrain() == Terrain::Sea)
                .unwrap_or(false);

        if !is_at_sea {
            return ConvoyOutcome::NotAtSea;
        }

        if let Some(dislodger) = dislodger_of(ctx, rslv, ord) {
            return ConvoyOutcome::Dislodged(dislodger);
        }

        if rslv.order_in_paradox(ord) {
            ConvoyOutcome::Paradox
        } else {
            ConvoyOutcome::NotDisrupted
        }
    }

    fn explain<'a>(
        context: &Context<'a, impl Adjudicate + WillUseConvoy>,
        resolver: &mut ResolverState<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderOutcome<&'a MappedMainOrder> {
        use crate::order::MainCommand::*;

        if let Some(&reason) = resolver.illegal_orders.get(order) {
            return reason.into();
        }

        match order.command {
            // A move order succeeds when the unit successfully transitions to the target.
            Move(..) => Rulebook::adjudicate_move(context, resolver, order).into(),

            // A support order "succeeds" if the support is not cut. This doesn't
            // necessarily mean support got applied.
            Support(..) => Rulebook::adjudicate_support(context, resolver, order).into(),

            // Hold orders succeed when the unit is not dislodged.
            Hold => Rulebook::adjudicate_hold(context, resolver, order).into(),

            // Convoy orders succeed when the unit is not dislodged and the convoy doesn't create
            // a paradox.
            Convoy(..) => Rulebook::adjudicate_convoy(context, resolver, order).into(),
        }
    }
}

impl Adjudicate for Rulebook {
    fn explain<'a>(
        &self,
        context: &Context<'a, Self>,
        resolver: &mut ResolverState<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderOutcome<&'a MappedMainOrder> {
        Rulebook::explain(context, resolver, order)
    }
}

impl Adjudicate for &Rulebook {
    fn explain<'a>(
        &self,
        context: &Context<'a, Self>,
        resolver: &mut ResolverState<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderOutcome<&'a MappedMainOrder> {
        Rulebook::explain(context, resolver, order)
    }
}

impl WillUseConvoy for Rulebook {
    fn will_use_convoy(&self, order: &MappedMainOrder, route: &[&MappedMainOrder]) -> bool {
        self.convoy_policy.will_use_convoy(order, route)
    }
}

impl WillUseConvoy for &Rulebook {
    fn will_use_convoy(&self, order: &MappedMainOrder, route: &[&MappedMainOrder]) -> bool {
        self.convoy_policy.will_use_convoy(order, route)
    }
}

impl Default for Rulebook {
    fn default() -> Self {
        Self::edition_1971()
    }
}

/// Standard rulebook build-phase adjudication.
mod build {
    use std::{
        borrow::Cow,
        collections::{HashMap, HashSet},
    };

    use crate::{
        geo::{ProvinceKey, SupplyCenter},
        judge::{
            build::{adjudicate, Adjudicate, Context, OrderOutcome, ResolverState, WorldState},
            MappedBuildOrder,
        },
        order::BuildCommand,
        Nation, Unit, UnitPosition,
    };

    /// Scratch space for standard variant build-phase adjudication.
    pub struct RuleScratch<'a> {
        deltas: HashMap<&'a Nation, (BuildCommand, i16)>,
        home_scs: HashMap<&'a Nation, HashSet<ProvinceKey>>,
        ownerships: HashMap<&'a Nation, HashSet<ProvinceKey>>,
    }

    impl<'a> RuleScratch<'a> {
        fn new<W: WorldState, A>(context: &Context<'a, W, A>) -> Self {
            let mut home_scs = HashMap::<&Nation, HashSet<ProvinceKey>>::new();
            let mut ownerships = HashMap::<&Nation, HashSet<ProvinceKey>>::new();

            // Figure out who owns what and where nations are allowed to build.
            for province in context
                .world_map
                .provinces()
                .filter(|p| p.is_supply_center())
            {
                if let SupplyCenter::Home(nat) = &province.supply_center {
                    home_scs.entry(nat).or_default().insert(province.into());
                }

                let key = ProvinceKey::from(province);
                if let Some(nation) = context
                    .this_time
                    .occupier(&key)
                    .or_else(|| context.last_time.get(&key))
                {
                    ownerships.entry(nation).or_default().insert(key);
                }
            }

            Self {
                deltas: ownerships
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
                    .collect(),
                home_scs,
                ownerships,
            }
        }
    }

    impl<'a> Adjudicate<'a> for crate::judge::Rulebook {
        type Scratch = RuleScratch<'a>;

        fn initialize<W: WorldState>(&self, context: &Context<'a, W, Self>) -> Self::Scratch {
            Self::Scratch::new(context)
        }

        fn explain<W: WorldState>(
            &self,
            context: &Context<'a, W, Self>,
            resolver: &mut ResolverState<'a, Self::Scratch>,
            order: &'a MappedBuildOrder,
        ) -> OrderOutcome {
            use crate::judge::build::OrderOutcome::*;

            let Some(delta) = resolver.scratch.deltas.get_mut(&order.nation) else {
                return RedeploymentProhibited;
            };

            // A power is only allowed to build or disband in a given turn, not both
            if delta.0 != order.command {
                return RedeploymentProhibited;
            }

            let adjudication = adjudicate(context, &resolver.scratch.home_scs, order);

            if adjudication != OrderOutcome::Succeeds {
                return adjudication;
            }

            match order.command {
                BuildCommand::Build => {
                    if delta.1 == 0 {
                        return AllBuildsUsed;
                    }

                    delta.1 -= 1;

                    resolver
                        .final_units
                        .entry(&order.nation)
                        .or_default()
                        .insert((order.unit_type, order.region.clone()));

                    Succeeds
                }
                BuildCommand::Disband => {
                    if delta.1 == 0 {
                        return AllDisbandsUsed;
                    }

                    delta.1 -= 1;

                    resolver
                        .final_units
                        .entry(&order.nation)
                        .or_default()
                        .remove(&(order.unit_type, order.region.clone()));

                    Succeeds
                }
            }
        }

        fn civil_disorder<W: WorldState>(
            &self,
            context: &Context<'a, W, Self>,
            resolver: &mut ResolverState<'a, Self::Scratch>,
        ) {
            let world_graph = context.world_map.to_graph();

            for (nation, delta) in &mut resolver.scratch.deltas {
                if delta.0 == BuildCommand::Build || delta.1 == 0 {
                    continue;
                }

                let usize_delta: usize = delta.1.try_into().unwrap();
                let units = resolver.final_units.remove(*nation).unwrap();

                // Per 2023 rulebook, units disband based on distance from the nation's owned
                // supply centers (earlier editions had it based on distance from the home supply centers)
                let Some(owned_scs) = resolver.scratch.ownerships.get(*nation) else {
                    // If there are no owned supply centers, all units disband
                    resolver
                        .civil_disorder
                        .extend(units.into_iter().map(|unit| {
                            UnitPosition::new(Unit::new(Cow::Borrowed(*nation), unit.0), unit.1)
                        }));
                    continue;
                };

                // Get all regions in the owned supply centers. The rules require checking
                // distance to all coasts, so province precision is insufficient.
                let owned_sc_regions = context
                    .world_map
                    .regions()
                    .filter(|r| owned_scs.contains(r.province()))
                    .collect::<Vec<_>>();

                let mut units_by_disband_priority = units
                    .into_iter()
                    .map(|unit| {
                        let unit_region = context
                            .world_map
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
                resolver.civil_disorder.extend(
                    units_by_disband_priority.drain(0..usize_delta).map(|v| {
                        UnitPosition::new(Unit::new(Cow::Borrowed(*nation), v.0 .0), v.0 .1)
                    }),
                );

                // Add the remaining units to the map of units that survive the turn.
                resolver.final_units.insert(
                    nation,
                    units_by_disband_priority.into_iter().map(|v| v.0).collect(),
                );
            }
        }
    }
}

/// The outcome of a main-phase hold order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HoldOutcome<O> {
    /// The unit remains in its current region
    Succeeds,
    /// The unit is dislodged by the specified order
    Dislodged(O),
}

impl<O> HoldOutcome<O> {
    /// Apply a function to any orders referenced by `self`, returning a new outcome.
    pub fn map_order<U>(self, map_fn: impl Fn(O) -> U) -> HoldOutcome<U> {
        use HoldOutcome::*;

        match self {
            Succeeds => Succeeds,
            Dislodged(o) => Dislodged(map_fn(o)),
        }
    }
}

impl<O> From<&'_ HoldOutcome<O>> for OrderState {
    fn from(other: &HoldOutcome<O>) -> Self {
        if matches!(other, HoldOutcome::Succeeds) {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

impl<O> From<HoldOutcome<O>> for OrderState {
    fn from(other: HoldOutcome<O>) -> Self {
        (&other).into()
    }
}

/// The outcome of a main-phase move order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttackOutcome<O> {
    /// The order was a move to the unit's current location.
    MoveToSelf,
    /// There was no way for the unit to reach the specified destination.
    ///
    /// This usually indicates that a convoy was possible but either disrupted
    /// or not ordered, as routes where there is no possibility of a path are
    /// instead deemed illegal.
    NoPath,
    /// The unit tried to move into a province occupied by another unit of the
    /// same nation.
    FriendlyFire,
    /// The unit was prevented from entering the province by the specified order.
    Prevented(O),
    /// The intended victim of the attack instead dislodged the attacker and did not use a convoy.
    ///
    /// A unit that loses a head-to-head battle is dislodged, cannot retreat to the province from
    /// which it was attacked, and has no strength to prevent other units from occupying that
    /// province.
    LostHeadToHead,
    /// The intended victim of the attack fended off the attacker, possibly with support from
    /// other units.
    OccupierDefended,
    /// The unit successfully moved to its destination.
    Succeeds,
}

impl<O> AttackOutcome<O> {
    /// Apply a function to any orders referenced by `self`, returning a new outcome.
    pub fn map_order<U>(self, map_fn: impl Fn(O) -> U) -> AttackOutcome<U> {
        use AttackOutcome::*;
        match self {
            MoveToSelf => MoveToSelf,
            NoPath => NoPath,
            FriendlyFire => FriendlyFire,
            Prevented(p) => Prevented(map_fn(p)),
            LostHeadToHead => LostHeadToHead,
            OccupierDefended => OccupierDefended,
            Succeeds => Succeeds,
        }
    }
}

impl<O> From<&'_ AttackOutcome<O>> for OrderState {
    fn from(ao: &AttackOutcome<O>) -> Self {
        if matches!(ao, AttackOutcome::Succeeds) {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

impl<O> From<AttackOutcome<O>> for OrderState {
    fn from(ao: AttackOutcome<O>) -> Self {
        (&ao).into()
    }
}

impl<O> From<AttackOutcome<O>> for bool {
    fn from(ao: AttackOutcome<O>) -> Self {
        OrderState::from(ao).into()
    }
}
