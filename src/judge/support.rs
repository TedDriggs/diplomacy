//! Contains helper functions for evaluating the success of support commands
//! during the main phase of a turn.

use super::prelude::*;
use geo;

fn order_cuts<'a, A: Adjudicate>(ctx: &ResolverContext<'a>,
                                 resolver: &mut ResolverState<'a, A>,
                                 support_order: &MappedMainOrder<'a>,
                                 cutting_order: &MappedMainOrder<'a>)
                                 -> bool {
    support_order.nation != cutting_order.nation
}

/// Find all orders which cut a specified support order.
pub fn find_cutting_orders<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                              resolver: &mut ResolverState<'a, A>,
                                              support_order: &MappedMainOrder<'a>)
                                              -> Vec<&'a MappedMainOrder<'a>> {
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
pub fn is_order_cut<'a, A: Adjudicate>(ctx: &ResolverContext<'a>,
                                       resolver: &mut ResolverState<'a, A>,
                                       support_order: &MappedMainOrder<'a>)
                                       -> bool {
    for order in &ctx.orders {
        if order_cuts(ctx, resolver, support_order, &order) {
            return true;
        }
    }

    false
}

fn needed_at<'a>(supported: &'a MappedMainOrder<'a>) -> &'a geo::Region<'a> {
    use order::MainCommand::*;
    match supported.command {
        Move(ref dest) => dest,
        Hold | Support(..) | Convoy(..) => supported.region,
    }
}

fn can_reach<'a>(world_map: &'a geo::Map<'a>,
                 supported: &'a MappedMainOrder<'a>,
                 support_order: &'a MappedMainOrder<'a>)
                 -> bool {
    world_map.find_border_between(support_order.region, needed_at(supported)).is_some()
}

/// Returns true if a given support order successfully supports the specified supported order.
fn is_successful<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                    resolver: &mut ResolverState<'a, A>,
                                    supported: &MappedMainOrder<'a>,
                                    support_order: &'a MappedMainOrder<'a>)
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
                                              supported: &MappedMainOrder<'a>)
                                              -> Vec<&'a MappedMainOrder<'a>> {
    let mut supports = vec![];
    for order in ctx.orders_ref() {
        if is_successful(ctx, resolver, supported, order) {
            supports.push(order)
        }
    }

    supports
}