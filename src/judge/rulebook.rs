use super::calc::{dislodger_of, is_head_to_head, max_prevent_result, path_exists};
use super::convoy::ConvoyOutcome;
use super::resolver::{Adjudicate, ResolverContext, ResolverState};
use super::support::{self, SupportOutcome};
use super::{MappedMainOrder, OrderState};
use crate::geo::Terrain;
use crate::judge::strength::{Prevent, Strength};
use crate::order::Command;
use crate::ShortName;

/// The standard Diplomacy rules.
#[derive(Debug, Clone, Default)]
pub struct Rulebook;

impl Rulebook {
    /// Apply rules to determine hold outcome.
    pub fn adjudicate_hold<'a>(
        ctx: &'a ResolverContext<'a>,
        rslv: &mut ResolverState<'a, Self>,
        ord: &'a MappedMainOrder,
    ) -> HoldOutcome<'a> {
        dislodger_of(ctx, rslv, ord)
            .map(HoldOutcome::Dislodged)
            .unwrap_or(HoldOutcome::Succeeds)
    }

    /// Apply rules to determine move outcome.
    pub fn adjudicate_move<'a>(
        ctx: &'a ResolverContext<'a>,
        rslv: &mut ResolverState<'a, Self>,
        ord: &'a MappedMainOrder,
    ) -> AttackOutcome<'a> {
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
                AttackOutcome::Prevented(prevent.unwrap())
            } else {
                if let Some(occupier) =
                    ctx.find_order_to_province(ord.command.move_dest().unwrap().into())
                {
                    // A head-to-head battle occurs when two units have mirrored move orders and
                    // no convoy is available to help one of the units move around the other.
                    let is_head_to_head = is_head_to_head(ctx, rslv, ord, occupier);

                    let resistance = if !occupier.command.is_move() || is_head_to_head {
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
                        1 + support::find_for(ctx, rslv, occupier).len()
                    }
                    // failed exits resist with strength 1 (the unit trapped in the province)
                    else if rslv.resolve(ctx, occupier) == OrderState::Fails {
                        1
                    // successful exits mount no resistance
                    } else {
                        0
                    };

                    // A unit can not dislodge a unit of the same player.
                    // Head-to-head, failed exit, and hold cases all collapse in friendly fire.
                    if resistance > 0 && ord.nation == occupier.nation {
                        return AttackOutcome::FriendlyFire;
                    } else if resistance > 0 {
                        // Supports to a foreign unit can not be used to dislodge an own unit.
                        // Therefore, we remove any move supports from the nation whose unit
                        // is resisting the move.
                        atk_supports = atk_supports
                            .into_iter()
                            .filter(|sup| sup.nation != occupier.nation)
                            .collect();
                        atk_strength = 1 + atk_supports.len();

                        if atk_strength <= resistance {
                            if is_head_to_head {
                                return AttackOutcome::LostHeadToHead;
                            } else {
                                return AttackOutcome::Defended;
                            }
                        }
                    }
                }

                AttackOutcome::Succeeds
            }
        } else {
            panic!("Don't try to adjudicate non-moves as moves");
        }
    }

    pub fn adjudicate_support<'a>(
        ctx: &'a ResolverContext<'a>,
        rslv: &mut ResolverState<'a, Self>,
        ord: &'a MappedMainOrder,
    ) -> SupportOutcome<'a> {
        if support::is_supporting_self(ord) {
            SupportOutcome::SupportingSelf
        } else {
            match support::find_cutting_order(ctx, rslv, ord) {
                Some(cutter) => SupportOutcome::CutBy(cutter),
                None => SupportOutcome::NotDisrupted,
            }
        }
    }

    pub fn adjudicate_convoy<'a>(
        ctx: &'a ResolverContext<'a>,
        rslv: &mut ResolverState<'a, Self>,
        ord: &'a MappedMainOrder,
    ) -> ConvoyOutcome<'a> {
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
        context: &'a ResolverContext<'a>,
        resolver: &mut ResolverState<'a, Self>,
        order: &'a MappedMainOrder,
    ) -> OrderState {
        use crate::order::MainCommand::*;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldOutcome<'a> {
    /// The unit remains in its current region
    Succeeds,
    /// The unit is dislodged by the specified order
    Dislodged(&'a MappedMainOrder),
}

impl From<HoldOutcome<'_>> for OrderState {
    fn from(other: HoldOutcome<'_>) -> Self {
        if other == HoldOutcome::Succeeds {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackOutcome<'a> {
    MoveToSelf,
    NoPath,
    FriendlyFire,
    Prevented(Prevent<'a>),
    /// The intended victim of the attack instead dislodged the attacker and did not use a convoy.
    ///
    /// A unit that loses a head-to-head battle is dislodged, cannot retreat to the province from
    /// which it was attacked, and has no strength to prevent other units from occupying that
    /// province.
    LostHeadToHead,
    /// The intended victim of the attack fended off the attacker, possibly with support from
    /// other units.
    Defended,
    Succeeds,
}

impl<'a> From<AttackOutcome<'a>> for OrderState {
    fn from(ao: AttackOutcome) -> Self {
        use self::AttackOutcome::*;
        match ao {
            Succeeds => OrderState::Succeeds,
            NoPath | MoveToSelf | FriendlyFire | Prevented(..) | LostHeadToHead | Defended => {
                OrderState::Fails
            }
        }
    }
}

impl<'a> From<AttackOutcome<'a>> for bool {
    fn from(ao: AttackOutcome) -> Self {
        OrderState::from(ao).into()
    }
}
