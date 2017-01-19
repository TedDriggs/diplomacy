//! Contains helper functions for evaluating the success of support commands
//! during the main phase of a turn.

use super::prelude::*;
use super::Outcome;
use order::{Command, SupportedOrder};
use geo;
use super::calc;

fn order_cuts<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                 resolver: &mut ResolverState<'a, A>,
                                 support_order: &MappedMainOrder,
                                 cutting_order: &MappedMainOrder)
                                 -> bool {
    if let Some(ref dst) = cutting_order.command.move_dest() {
        let supporting_attack_on_cutter = match support_order.command {
            MainCommand::Support(SupportedOrder::Move(_, ref supported_dst)) => dst.province() == supported_dst,
            _ => false
        };
        
        dst == &support_order.region.province()
        && !supporting_attack_on_cutter 
        && support_order.nation != cutting_order.nation
        && calc::path_exists(ctx, resolver, cutting_order) 
    } else {
        false
    }
}

/// Find all orders which cut a specified support order.
pub fn find_cutting_order<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                              resolver: &mut ResolverState<'a, A>,
                                              support_order: &MappedMainOrder)
                                              -> Option<&'a MappedMainOrder> {
    for order in ctx.orders_ref() {
        if order_cuts(ctx, resolver, support_order, order) {
            return Some(order);
        }
    }

    None
}

/// A SUPPORT decision of a unit ordered to support results in 'cut' when:
/// At least one of the units ordered to move to the area of the supporting unit 
/// has a minimum ATTACK STRENGTH of one or more. Again, if the support order is 
/// a move support, then the unit that is on the area where the move is directed, 
/// should not be taken into account. Finally, the SUPPORT decisions also results 
/// in 'cut' when the DISLODGE decision of the unit has status 'dislodged' (dislodge rule).
///
/// This method short-circuits the search after any hit has been found.
pub fn is_order_cut<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                       resolver: &mut ResolverState<'a, A>,
                                       support_order: &MappedMainOrder)
                                       -> bool {
    for order in &ctx.orders {
        if order_cuts(ctx, resolver, support_order, &order) {
            return true;
        }
    }

    false
}

pub fn is_supporting_self(support_order: &MappedMainOrder) -> bool {
    if let MainCommand::Support(SupportedOrder::Hold(ref loc)) = support_order.command {
        loc.province() == &support_order.region
    } else {
        false
    }
}

fn needed_at(supported: &MappedMainOrder) -> &geo::RegionKey {
    use order::MainCommand::*;
    match supported.command {
        Move(ref dest) => dest,
        Hold | Support(..) | Convoy(..) => &supported.region,
    }
}

fn can_reach<'a>(world_map: &'a geo::Map,
                 supported: &'a MappedMainOrder,
                 support_order: &'a MappedMainOrder)
                 -> bool {
    world_map.find_border_between(&support_order.region, needed_at(supported)).is_some()
}

/// Returns true if a given support order successfully supports the specified supported order.
pub fn is_successful<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                        resolver: &mut ResolverState<'a, A>,
                                        supported: &MappedMainOrder,
                                        support_order: &'a MappedMainOrder)
                                        -> bool {
    if let MainCommand::Support(ref beneficiary) = support_order.command {
        beneficiary == supported && can_reach(&ctx.world_map, supported, support_order) &&
        resolver.resolve(ctx, support_order).into()
    } else {
        false
    }
}

/// Finds all successful orders which support a given order.
pub fn find_successful_for<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                              resolver: &mut ResolverState<'a, A>,
                                              supported: &MappedMainOrder)
                                              -> Vec<&'a MappedMainOrder> {
    let mut supports = vec![];
    for order in ctx.orders_ref() {
        if is_successful(ctx, resolver, supported, order) {
            supports.push(order)
        }
    }

    supports
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportOutcome<'a> {
    NotDisrupted,
    SupportingSelf,
    CutBy(&'a MappedMainOrder),
}

impl<'a> SupportOutcome<'a> {
    pub fn is_successful(&self) -> bool {
        self == &SupportOutcome::NotDisrupted
    }
}

impl<'a> Outcome for SupportOutcome<'a> {}

#[cfg(test)]
mod test {
    use geo::{RegionKey, ProvinceKey, standard_map};
    use Nation;
    use UnitType;
    use order::{Order, MainCommand, SupportedOrder};
    use super::*;
    use super::super::{ResolverState, ResolverContext};

    fn reg(s: &str) -> RegionKey {
        RegionKey::new(ProvinceKey::new(s), None)
    }

    #[test]
    fn is_support_successful() {
        let ger = Nation("ger".into());
        let supp_com = SupportedOrder::Move(reg("nth"), reg("nwy"));
        let orders = vec![
            Order::new(ger.clone(), UnitType::Fleet, reg("ska"), MainCommand::Support(supp_com.clone())),
            Order::new(ger.clone(), UnitType::Fleet, reg("nth"), MainCommand::Move(reg("nwy"))),
        ];

        assert_eq!(supp_com, orders[1]);
        assert!(super::can_reach(standard_map(), &orders[1], &orders[0]));

        let resolver_ctx = ResolverContext::new(standard_map(), orders.clone());
        let mut res_state = ResolverState::with_adjudicator(super::super::rulebook::Rulebook);
        let supporters = find_successful_for(&resolver_ctx, &mut res_state, &orders[1]);
        assert!(!supporters.is_empty());
    }
}