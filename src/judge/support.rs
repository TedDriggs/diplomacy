//! Contains helper functions for evaluating the success of support commands
//! during the main phase of a turn.

use super::prelude::*;
use geo;
use super::calc;

fn order_cuts<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                 resolver: &mut ResolverState<'a, A>,
                                 support_order: &MappedMainOrder,
                                 cutting_order: &MappedMainOrder)
                                 -> bool {
    match cutting_order.command {
        MainCommand::Move(ref dst) => {
            dst == &support_order.region && calc::path_exists(ctx, resolver, cutting_order) &&
            support_order.nation != cutting_order.nation
        }
        _ => false,
    }
}

/// Find all orders which cut a specified support order.
pub fn find_cutting_orders<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                              resolver: &mut ResolverState<'a, A>,
                                              support_order: &MappedMainOrder)
                                              -> Vec<&'a MappedMainOrder> {
    let mut cutting_orders = vec![];
    for order in ctx.orders_ref() {
        if order_cuts(ctx, resolver, support_order, order) {
            cutting_orders.push(order)
        }
    }

    cutting_orders
}

/// Returns whether the support is cut. This method short-circuits the search after
/// any hit has been found.
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