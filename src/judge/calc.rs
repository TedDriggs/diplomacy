use order::{Command, Order};
use super::prelude::*;
use super::{support, convoy};
use geo::ProvinceKey;

/// Returns true if `order` is a move AND between the source and dest, either:
///
/// 1. A border exists OR
/// 1. A non-disrupted convoy route exists.
pub fn path_exists<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                      resolver: &mut ResolverState<'a, A>,
                                      order: &MappedMainOrder)
                                      -> bool {
    if let Some(dst) = order.command.move_dest() {
        if let Some(reg) = context.world_map.find_region_with_key(dst).ok() {
            if order.unit_type.can_occupy(reg.terrain()) {
                let border_exists = context.world_map
                    .find_border_between(&order.region, dst)
                    .map(|b| b.is_passable_by(&order.unit_type))
                    .unwrap_or(false);

                // NOTE: This short-circuits convoy assessment when there is a border.
                return border_exists || convoy::route_exists(context, resolver, order);
            }
        }
    }

    // default to false
    false
}

/// Checks if an order is a move to the province identified by `d`.
fn is_move_to_province<'a, 'b, DST: Into<&'b ProvinceKey>>(o: &MappedMainOrder, d: DST) -> bool {
    o.command.move_dest().map(|md| md.province() == d.into()).unwrap_or(false)
}

fn prevent_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                     resolver: &mut ResolverState<'a, A>,
                                     order: &'a MappedMainOrder)
                                     -> Option<Prevent<'a>> {
    if order.command.is_move() {
        Some(if !path_exists(context, resolver, order) {
            Prevent::NoPath
        } else {
            if let Some(h2h) = context.orders_ref()
                .iter()
                .find(|o| Order::is_head_to_head(o, order)) {
                match resolver.resolve(context, h2h) {
                    OrderState::Succeeds => Prevent::LostHeadToHead,
                    OrderState::Fails => {
                        Prevent::Prevents(order, support::find_for(context, resolver, order))
                    }
                }
            } else {
                Prevent::Prevents(order, support::find_for(context, resolver, order))
            }
        })
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn prevent_results<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                          resolver: &mut ResolverState<'a, A>,
                                          province: &ProvinceKey)
                                          -> Vec<Prevent<'a>> {
    let mut prevents = vec![];
    for order in context.orders_ref().iter().filter(|ord| is_move_to_province(ord, province)) {
        if let Some(prev) = prevent_result(context, resolver, order) {
            prevents.push(prev);
        }
    }

    prevents
}

pub fn max_prevent_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                             resolver: &mut ResolverState<'a, A>,
                                             preventing: &MappedMainOrder)
                                             -> Option<Prevent<'a>> {
    if let &MainCommand::Move(ref dst) = &preventing.command {
        let mut best_prevent = None;
        let mut best_prevent_strength = 0;
        for order in context.orders_ref()
            .iter()
            .filter(|ord| ord != &&preventing && is_move_to_province(ord, dst)) {
            if let Some(prev) = prevent_result(context, resolver, order) {
                let nxt_str = prev.strength();
                if nxt_str >= best_prevent_strength {
                    best_prevent_strength = nxt_str;
                    best_prevent = Some(prev);
                }
            }
        }

        best_prevent
    } else {
        None
    }
}

/// Get the order that dislodges the provided order, if one exists.
///
/// A DISLODGE decision of a unit results in 'dislodged' when:
/// There is a unit with a move order to the area of the unit, for which the
/// MOVE decision has status 'moves' and in case the unit (of the DISLODGE
/// decision) was ordered to move has a MOVE decision with status 'fails'.
pub fn dislodger_of<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                       resolver: &mut ResolverState<'a, A>,
                                       order: &'a MappedMainOrder)
                                       -> Option<&'a MappedMainOrder> {
    let order_ref = context.orders_ref();
    for dislodger in order_ref.into_iter().find(|o| is_move_to_province(o, &order.region)) {
        
        // If we found someone trying to move into `order`'s old province, we
        // check to see if `order` vacated. If so, then it couldn't have been
        // dislodged.
        if order.command.is_move() && resolver.resolve(context, order).into() {
            return None;
        }
        
        if resolver.resolve(context, dislodger).into() {
            return Some(dislodger);
        }
    }
    
    // If we couldn't find any orders that attempted to move into the province 
    // `order` occupied, then there can't be any dislodgers.
    None
}