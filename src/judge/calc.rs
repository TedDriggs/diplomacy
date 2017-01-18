use super::prelude::*;
use super::{support, convoy};
use geo::{RegionKey, ProvinceKey};

impl Border {
    fn is_passable_by(&self, unit_type: &UnitType) -> bool {
        match self.terrain() {
            &Terrain::Land => unit_type == &UnitType::Army,
            &Terrain::Sea => unit_type == &UnitType::Fleet,
            &Terrain::Coast => true,
        }
    }
}

pub fn path_exists<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                      resolver: &mut ResolverState<'a, A>,
                                      order: &MappedMainOrder)
                                      -> bool {
    match order.command {
        MainCommand::Move(ref dst) => {
            let border_exists = context.world_map
                .find_border_between(&order.region, dst)
                .map(|b| b.is_passable_by(&order.unit_type))
                .unwrap_or(false);

            let convoy_exists = convoy::route_exists(context, resolver, order);

            border_exists || convoy_exists
        }
        _ => true,
    }
}

/// Checks if an order is a move to the province identified by `d`.
fn is_move_to_province<'a, 'b, DST: Into<&'b ProvinceKey>>(o: &MappedMainOrder, d: DST) -> bool {
    if let MainCommand::Move(ref dst) = o.command {
        dst.province() == d.into()
    } else {
        false
    }
}

fn is_move_and_dest_is_not<'a, 'b, D: Into<&'b ProvinceKey>>(o: &MappedMainOrder, d: D) -> bool {
    match o.command {
        MainCommand::Move(ref dst) => dst.province() != d.into(),
        _ => false,
    }
}

fn is_head_to_head<'a>(o1: &MappedMainOrder, o2: &MappedMainOrder) -> bool {
    match (&o1.command, &o2.command) {
        (&MainCommand::Move(ref d1), &MainCommand::Move(ref d2)) => {
            o1.region.province() == d2.province() && o2.region.province() == d1.province()
        }
        _ => false,
    }
}

pub fn atk_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                     resolver: &mut ResolverState<'a, A>,
                                     order: &MappedMainOrder)
                                     -> Option<Attack<'a>> {
    use order::MainCommand::*;
    match order.command {
        Move(ref dst) => {
            if !path_exists(context, resolver, order) {
                Some(Attack::NoPath)
            } else {
                Some({

                    let dst_occupant =
                        context.orders.iter().find(|occ| occ.region.province() == dst.province());
                    let supports = support::find_successful_for(context, resolver, order);
                    match dst_occupant {
                        None => Attack::AgainstVacant(supports),
                        Some(ref occ) => {
                            // In the case that the occupier is leaving, this is a follow-in
                            if is_move_and_dest_is_not(occ, &order.region) &&
                               resolver.resolve(context, occ) == OrderState::Succeeds {
                                Attack::FollowingIn(supports)
                            } else if occ.nation == order.nation {
                                Attack::FriendlyFire
                            } else {
                                Attack::AgainstOccupied(supports)
                            }
                        }
                    }
                })
            }
        }
        // non-move commands don't generate a move outcome
        Hold | Support(..) | Convoy(..) => None,
    }
}

fn prevent_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                     resolver: &mut ResolverState<'a, A>,
                                     order: &MappedMainOrder)
                                     -> Option<Prevent<'a>> {
    use order::MainCommand::*;
    match order.command {
        Move(..) => {
            if !path_exists(context, resolver, order) {
                Some(Prevent::NoPath)
            } else {
                Some({
                    if let Some(h2h) = context.orders_ref()
                        .iter()
                        .find(|o| is_head_to_head(o, order)) {
                        match resolver.resolve(context, h2h) {
                            OrderState::Succeeds => Prevent::LostHeadToHead,
                            OrderState::Fails => {
                                Prevent::Prevents(support::find_successful_for(context,
                                                                               resolver,
                                                                               order))
                            }
                        }
                    } else {
                        Prevent::Prevents(support::find_successful_for(context, resolver, order))
                    }
                })
            }
        }
        Hold | Support(..) | Convoy(..) => None,
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
            .filter(|ord| ord != &&preventing && is_move_to_province(ord, dst.province())) {
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

/// Gets the hold outcome for a given province.
fn hold_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                  resolver: &mut ResolverState<'a, A>,
                                  region: &RegionKey)
                                  -> ProvinceHold<'a> {

    match context.orders_ref().iter().find(|o| &o.region == region) {
        None => ProvinceHold::Empty,
        Some(occupier) => {
            if occupier.command.is_move() {
                if resolver.resolve(context, occupier).into() {
                    ProvinceHold::SuccessfulExit
                } else {
                    ProvinceHold::FailedExit
                }
            } else {
                ProvinceHold::UnitHolds(support::find_successful_for(context, resolver, occupier))
            }
        }
    }
}

fn defend_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                    resolver: &mut ResolverState<'a, A>,
                                    order: &MappedMainOrder)
                                    -> Option<Defend<'a>> {
    match order.command {
        MainCommand::Move(..) => {
            Some(Defend(support::find_successful_for(context, resolver, order)))
        }
        _ => None,
    }
}

pub fn resistance_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                            resolver: &mut ResolverState<'a, A>,
                                            order: &MappedMainOrder)
                                            -> Resistance<'a> {
    if let Some(h2h) = context.orders_ref().iter().find(|o| is_head_to_head(order, o)) {
        defend_result(context, resolver, h2h).expect("Already verified h2h is a move").into()
    } else {
        use order::MainCommand::*;
        match order.command {
            Move(ref dest) => hold_result(context, resolver, dest).into(),
            Hold | Convoy(..) | Support(..) => unreachable!(),
        }
    }
}

/// Get the order that dislodges the provided order. This function does not check
/// whether the unit moved out of the province prior to being dislodged.
fn dislodger_no_exit<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                        resolver: &mut ResolverState<'a, A>,
                                        order: &'a MappedMainOrder)
                                        -> Option<&'a MappedMainOrder> {

    let order_ref = context.orders_ref();
    let mut dislodger = None;
    for order in order_ref.into_iter().find(|o| is_move_to_province(o, order.region.province())) {
        if resolver.resolve(context, order).into() {
            dislodger = Some(order);
            break;
        }
    }

    dislodger
}

/// Get the order that dislodges the provided order, if one exists.
pub fn dislodger_of<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                       resolver: &mut ResolverState<'a, A>,
                                       order: &'a MappedMainOrder)
                                       -> Option<&'a MappedMainOrder> {
    match order.command {
        MainCommand::Move(..) => {
            if resolver.resolve(context, order).into() {
                None
            } else {
                dislodger_no_exit(context, resolver, order)
            }
        }
        _ => dislodger_no_exit(context, resolver, order),
    }
}