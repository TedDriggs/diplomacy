use super::calc::{dislodger_of, is_head_to_head, max_prevent_result, path_exists};
use super::convoy::ConvoyOutcome;
use super::resolver::{Context, ResolverState};
use super::support::{self, SupportOutcome};
use super::{Adjudicate, MappedMainOrder, OrderOutcome, OrderState};
use crate::geo::Terrain;
use crate::judge::strength::Strength;
use crate::order::Command;
use crate::ShortName;

/// The standard Diplomacy rules.
#[derive(Debug, Clone, Default)]
pub struct Rulebook;

impl Rulebook {
    /// Apply rules to determine hold outcome.
    fn adjudicate_hold<'a>(
        ctx: &Context<'a, Self>,
        rslv: &mut ResolverState<'a>,
        ord: &'a MappedMainOrder,
    ) -> HoldOutcome<&'a MappedMainOrder> {
        dislodger_of(ctx, rslv, ord)
            .map(HoldOutcome::Dislodged)
            .unwrap_or(HoldOutcome::Succeeds)
    }

    /// Apply rules to determine move outcome.
    fn adjudicate_move<'a>(
        ctx: &Context<'a, Self>,
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
        ctx: &Context<'a, Self>,
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
        ctx: &Context<'a, Self>,
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
}

impl Adjudicate for Rulebook {
    fn adjudicate<'a>(
        &self,
        context: &Context<'a, Self>,
        resolver: &mut ResolverState<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderState {
        self.explain(context, resolver, order).into()
    }

    fn explain<'a>(
        &self,
        context: &Context<'a, Self>,
        resolver: &mut ResolverState<'a>,
        order: &'a MappedMainOrder,
    ) -> OrderOutcome<&'a MappedMainOrder> {
        use crate::order::MainCommand::*;

        if let Some(reason) = resolver.invalid_orders.get(order) {
            return OrderOutcome::Invalid(*reason);
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
