use super::strength::{Prevent, Strength};
use super::{convoy, support};
use super::{Adjudicate, Context, MappedMainOrder, ResolverState};
use crate::order::{Command, MainCommand};
use crate::{geo::ProvinceKey, ShortName};

/// Returns true if `order` is a move AND between the source and dest, either:
///
/// 1. A border exists and the order permits direct travel OR
/// 1. A non-disrupted convoy route exists.
pub fn path_exists<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    order: &MappedMainOrder,
) -> bool {
    if let MainCommand::Move(cmd) = &order.command {
        let dst = cmd.dest();
        if let Some(reg) = context.world_map.find_region(&dst.short_name()) {
            if order.unit_type.can_occupy(reg.terrain()) {
                // If the move order allows direct travel, look for a border that would support
                // direct movement.
                let can_travel_directly = !cmd.mandates_convoy()
                    && context
                        .world_map
                        .find_border_between(&order.region, dst)
                        .map(|b| b.is_passable_by(order.unit_type))
                        .unwrap_or(false);

                // NOTE: As-written, this short-circuits convoy assessment when
                // there is an acceptable direct route. Don't change that behavior, as
                // it may impact how resolution works.
                return can_travel_directly || convoy::route_exists(context, resolver, order);
            }
        }
    }

    // default to false
    false
}

/// Two orders form a head-to-head battle when they are mirrored moves and no convoy exists to
/// ferry one of the armies around the other one.
pub fn is_head_to_head<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    order1: &MappedMainOrder,
    order2: &MappedMainOrder,
) -> bool {
    // First check if the two orders are mirrored moves
    (order1.move_dest() != Some(&order1.region)
        && order1.move_dest().map(|d| d.province()) == Some(order2.region.province())
        && order2.move_dest().map(|d| d.province()) == Some(order1.region.province()))
    // Then check to see if a convoy route enables the two to avoid head-to-head battle
        && !convoy::route_exists(context, resolver, order1)
        && !convoy::route_exists(context, resolver, order2)
}

fn prevent_result<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    order: &'a MappedMainOrder,
) -> Option<Prevent<'a>> {
    if order.is_move() {
        if !path_exists(context, resolver, order) {
            Some(Prevent::NoPath)
        } else {
            // A unit that lost a head-to-head cannot prevent.
            if let Some(h2h) = context
                .orders()
                .find(|o| is_head_to_head(context, resolver, o, order))
            {
                if resolver.resolve(context, h2h).into() {
                    return Some(Prevent::LostHeadToHead);
                }
            }

            Some(Prevent::Prevents(
                order,
                support::find_for(context, resolver, order),
            ))
        }
    } else {
        None
    }
}

/// Get all prevents for a province, with their supporters.
pub fn prevent_results<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    province: &ProvinceKey,
) -> Vec<Prevent<'a>> {
    context
        .orders()
        .filter(|ord| ord.is_move_to_province(province))
        .filter_map(|ord| prevent_result(context, resolver, ord))
        .collect()
}

pub fn max_prevent_result<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    preventing: &MappedMainOrder,
) -> Option<Prevent<'a>> {
    preventing.move_dest().and_then(|dst| {
        let mut best_prevent = None;
        let mut best_prevent_strength = 0;
        for order in context
            .orders()
            .filter(|ord| ord != &preventing && ord.is_move_to_province(dst.into()))
        {
            if is_head_to_head(context, resolver, order, preventing)
                && resolver.resolve(context, order).into()
            {
                if best_prevent == None {
                    best_prevent = Some(Prevent::LostHeadToHead);
                }
                continue;
            } else if let Some(prev) = prevent_result(context, resolver, order) {
                let nxt_str = prev.strength();
                if nxt_str >= best_prevent_strength {
                    best_prevent_strength = nxt_str;
                    best_prevent = Some(prev);
                }
            }
        }

        best_prevent
    })
}

/// Get the order that dislodges the provided order, if one exists.
///
/// A DISLODGE decision of a unit results in 'dislodged' when:
/// There is a unit with a move order to the area of the unit, for which the
/// MOVE decision has status 'moves' and in case the unit (of the DISLODGE
/// decision) was ordered to move has a MOVE decision with status 'fails'.
pub fn dislodger_of<'a>(
    context: &Context<'a, impl Adjudicate>,
    resolver: &mut ResolverState<'a>,
    order: &'a MappedMainOrder,
) -> Option<&'a MappedMainOrder> {
    for would_be_dislodger in context
        .orders()
        .filter(|o| o.is_move_to_province(order.region.province()))
    {
        // If we found someone trying to move into `order`'s old province, we
        // check to see if `order` vacated. If so, then it couldn't have been
        // dislodged.
        //
        // We defer this check to avoid triggering unnecessary resolutions
        if order.is_move() && resolver.resolve(context, order).into() {
            return None;
        }

        if resolver.resolve(context, would_be_dislodger).into() {
            return Some(would_be_dislodger);
        }
    }

    // If we couldn't find any orders that attempted to move into the province
    // `order` occupied, then there can't be any dislodgers.
    None
}

#[cfg(test)]
mod tests {
    use super::{max_prevent_result, Prevent};
    use crate::judge::{Context, MappedMainOrder, ResolverState, Rulebook};

    #[test]
    fn t6e01_prevent_strengths() {
        let orders = vec![
            "GER: A ber -> pru",
            "GER: F kie -> ber",
            "GER: A sil supports A ber -> pru",
            "RUS: A pru -> ber",
        ]
        .into_iter()
        .map(|ord| ord.parse::<MappedMainOrder>().unwrap())
        .collect::<Vec<_>>();

        let context = Context::new(crate::geo::standard_map(), Rulebook, &orders);
        let mut state = ResolverState::new();

        assert_eq!(
            max_prevent_result(&context, &mut state, &orders[1]),
            Some(Prevent::LostHeadToHead)
        );
    }

    #[test]
    fn t6g16_prevent_strengths() {
        let orders = vec![
            "ENG: A nwy -> swe",                // 0
            "ENG: A den Supports A nwy -> swe", // 1
            "ENG: F bal Supports A nwy -> swe", // 2
            "ENG: F nth -> nwy",                // 3
            "RUS: A swe -> nwy via Convoy",     // 4
            "RUS: F ska convoys swe -> nwy",    // 5
            "RUS: F nwg Supports A swe -> nwy", // 6
        ]
        .into_iter()
        .map(|ord| ord.parse::<MappedMainOrder>().unwrap())
        .collect::<Vec<_>>();

        let context = Context::new(crate::geo::standard_map(), Rulebook, &orders);
        let mut state = ResolverState::new();
        let nth_prevent = max_prevent_result(&context, &mut state, &orders[3]);
        let swe_prevent = max_prevent_result(&context, &mut state, &orders[4]);

        assert_eq!(
            nth_prevent,
            Some(Prevent::Prevents(&orders[4], vec![&orders[6]]))
        );
        assert_eq!(swe_prevent, Some(Prevent::Prevents(&orders[3], vec![])));
    }
}
