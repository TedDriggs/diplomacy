//! Contains helper functions for evaluating the success of support commands
//! during the main phase of a turn.

use super::{calc, Adjudicate, Context, MappedMainOrder, OrderState, ResolverState};
use crate::geo::Map;
use crate::order::{Command, MainCommand, SupportedOrder};

fn order_cuts<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    support_order: &MappedMainOrder,
    cutting_order: &'a MappedMainOrder,
) -> bool {
    // Only moves can cut support
    if let Some(dst) = cutting_order.command.move_dest() {
        // If the cutting order is attacking somebody else, it can't cut this support
        if dst != support_order.region.province() {
            return false;
        }

        // Units cannot cut support provided by their countrymen
        if support_order.nation == cutting_order.nation {
            return false;
        }

        // If the supported order is attacking the cutting order's province, then
        // support is only cut if the cutting order dislodges the supporter
        let is_supporter_immune = match support_order.command {
            MainCommand::Support(SupportedOrder::Move(_, _, ref supported_dst))
                if cutting_order.region.province() == supported_dst.province() =>
            {
                // Per http://uk.diplom.org/pouch/Zine/S2009M/Kruijswijk/DipMath_Chp5.htm
                // we only resolve the cutting order in this precise case to minimize cycle
                // risks.
                !bool::from(resolver.resolve(ctx, cutting_order))
            }
            _ => false,
        };

        if is_supporter_immune {
            return false;
        }

        // We use the Szykman rule to deal with paradoxes created by convoying support cuts.
        // Therefore, we don't worry about units being convoyed that cut support on attacks against
        // their convoys; those situations will be handled by the cycle resolver.
        calc::path_exists(ctx, resolver, cutting_order)
    } else {
        false
    }
}

/// Find all orders which cut a specified support order.
pub fn find_cutting_order<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    support_order: &MappedMainOrder,
) -> Option<&'a MappedMainOrder> {
    ctx.orders()
        .find(|order| order_cuts(ctx, resolver, support_order, order))
}

/// A SUPPORT decision of a unit ordered to support results in 'cut' when:
/// At least one of the units ordered to move to the area of the supporting unit
/// has a minimum ATTACK STRENGTH of one or more. Again, if the support order is
/// a move support, then the unit that is on the area where the move is directed,
/// should not be taken into account. Finally, the SUPPORT decisions also results
/// in 'cut' when the DISLODGE decision of the unit has status 'dislodged' (dislodge rule).
///
/// This method short-circuits the search after any hit has been found.
pub fn is_order_cut<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    support_order: &MappedMainOrder,
) -> bool {
    ctx.orders()
        .any(|order| order_cuts(ctx, resolver, support_order, order))
}

pub fn is_supporting_self(support_order: &MappedMainOrder) -> bool {
    if let MainCommand::Support(SupportedOrder::Hold(_, ref loc)) = support_order.command {
        loc.province() == &support_order.region
    } else {
        false
    }
}

/// Determines if a support order can reach the province where it is needed.
/// This requires a border from the unit's current region to the province
/// where support is needed.
pub fn can_reach(world_map: &Map, support_order: &MappedMainOrder) -> bool {
    if let MainCommand::Support(supported) = &support_order.command {
        // The province which the supporter must be able to reach to help.
        // For move orders, this is the **destination** province. For all other orders,
        // it is the **currently occupied** province.
        let needed_at = match supported {
            SupportedOrder::Move(_, _, dest) => dest.province(),
            SupportedOrder::Hold(_, target) => target.province(),
        };

        world_map
            .find_borders_between(&support_order.region, needed_at)
            .iter()
            .any(|b| b.is_passable_by(support_order.unit_type))
    } else {
        false
    }
}

/// Returns true if an order is a legal support order.
fn is_legal(support_order: &MappedMainOrder) -> bool {
    use crate::order::MainCommand::*;

    match support_order.command {
        Support(SupportedOrder::Hold(_, ref tgt)) => {
            tgt.province() != support_order.region.province()
        }

        // test case 6.d.34; support targeting own area not allowed.
        Support(SupportedOrder::Move(_, _, ref dst)) => {
            dst.province() != support_order.region.province()
        }
        Hold | Move(..) | Convoy(..) => false,
    }
}

/// Returns true if a given support order successfully supports the specified supported order.
pub fn is_successful<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    supported: &MappedMainOrder,
    support_order: &'a MappedMainOrder,
) -> bool {
    if let MainCommand::Support(ref beneficiary) = support_order.command {
        is_legal(support_order)
            && beneficiary.is_legal()
            && beneficiary == supported
            && can_reach(ctx.world_map, support_order)
            && resolver.resolve(ctx, support_order).into()
    } else {
        false
    }
}

/// Finds all successful orders which support a given order.
pub fn find_for<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    supported: &MappedMainOrder,
) -> Vec<&'a MappedMainOrder> {
    ctx.orders()
        .filter(|order| is_successful(ctx, resolver, supported, order))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupportOutcome<O> {
    NotDisrupted,
    SupportingSelf,
    /// The support order can't reach the province where assistance is required.
    ///
    /// Support cannot be convoyed, so reachability is a simple border check.
    CantReach,
    CutBy(O),
}

impl<O> SupportOutcome<O> {
    pub fn is_successful(&self) -> bool {
        matches!(self, SupportOutcome::NotDisrupted)
    }

    /// Apply a function to any orders referenced by `self`, returning a new outcome.
    pub fn map_order<U>(self, map_fn: impl Fn(O) -> U) -> SupportOutcome<U> {
        use SupportOutcome::*;
        match self {
            NotDisrupted => NotDisrupted,
            SupportingSelf => SupportingSelf,
            CantReach => CantReach,
            CutBy(atk) => CutBy(map_fn(atk)),
        }
    }
}

impl<O> From<&'_ SupportOutcome<O>> for OrderState {
    fn from(so: &SupportOutcome<O>) -> Self {
        if matches!(so, SupportOutcome::NotDisrupted) {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

impl<O> From<SupportOutcome<O>> for OrderState {
    fn from(so: SupportOutcome<O>) -> Self {
        (&so).into()
    }
}

#[cfg(test)]
mod test {
    use super::super::{Context, ResolverState};
    use super::*;
    use crate::geo::{standard_map, RegionKey};
    use crate::order::{MainCommand, MoveCommand, Order, SupportedOrder};
    use crate::Nation;
    use crate::UnitType;
    use std::str::FromStr;

    fn reg(s: &str) -> RegionKey {
        RegionKey::from_str(s).unwrap()
    }

    #[test]
    fn is_support_successful() {
        let ger = Nation::from("ger");
        let supp_com = SupportedOrder::Move(UnitType::Fleet, reg("nth"), reg("nwy"));
        let orders = vec![
            Order::new(
                ger.clone(),
                UnitType::Fleet,
                reg("ska"),
                MainCommand::Support(supp_com.clone()),
            ),
            Order::new(
                ger,
                UnitType::Fleet,
                reg("nth"),
                MoveCommand::new(reg("nwy")).into(),
            ),
        ];

        assert_eq!(supp_com, orders[1]);
        assert!(super::can_reach(standard_map(), &orders[0]));

        let resolver_ctx = Context::new(standard_map(), crate::judge::Rulebook::default(), &orders);
        let mut res_state = ResolverState::new();
        let supporters = find_for(&resolver_ctx, &mut res_state, &orders[1]);
        assert!(!supporters.is_empty());
    }

    #[test]
    fn support_t6b04_support_to_unreachable_coast_allowed() {
        let fra = Nation::from("fra");
        let spa_nc = RegionKey::from_str("spa(nc)").unwrap();
        let supp_com = SupportedOrder::Move(UnitType::Fleet, reg("gas"), spa_nc.clone());
        let orders = vec![
            Order::new(
                fra.clone(),
                UnitType::Fleet,
                reg("gas"),
                MoveCommand::new(spa_nc).into(),
            ),
            Order::new(fra, UnitType::Fleet, reg("mar"), supp_com.clone().into()),
            Order::new(
                "ita".into(),
                UnitType::Fleet,
                reg("wes"),
                MoveCommand::new(reg("spa(sc)")).into(),
            ),
        ];

        assert_eq!(supp_com, orders[0]);
        assert!(super::can_reach(standard_map(), &orders[1]));
    }
}
