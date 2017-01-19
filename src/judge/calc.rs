use order::Command;
use super::prelude::*;
use super::{support, convoy};
use geo::{RegionKey, ProvinceKey};

impl Border {
    fn is_passable_by(&self, unit_type: &UnitType) -> bool {
        unit_type.can_occupy(self.terrain())
    }
}

impl UnitType {
    fn can_occupy(&self, terrain: &Terrain) -> bool {
        match *terrain {
            Terrain::Coast => true,
            Terrain::Land => self == &UnitType::Army,
            Terrain::Sea => self == &UnitType::Fleet,
        }
    }
}

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

                let convoy_exists = convoy::route_exists(context, resolver, order);

                border_exists || convoy_exists
            } else {
                false
            }
        } else {
            false
        }
    } else {
        true
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

pub fn is_head_to_head<'a>(o1: &MappedMainOrder, o2: &MappedMainOrder) -> bool {
    match (&o1.command, &o2.command) {
        (&MainCommand::Move(ref d1), &MainCommand::Move(ref d2)) => {
            o1.region.province() == d2.province() && o2.region.province() == d1.province()
        }
        _ => false,
    }
}

pub fn indiscriminate_atk_strength<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                                      resolver: &mut ResolverState<'a, A>,
                                                      order: &MappedMainOrder)
                                                      -> usize {
    if order.command.move_dest().is_some() {
        if !path_exists(context, resolver, order) {
            0
        } else {
            let supports = support::find_successful_for(context, resolver, order);
            1 + supports.len()
        }
    } else {
        0
    }
}

fn prevent_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                     resolver: &mut ResolverState<'a, A>,
                                     order: &MappedMainOrder)
                                     -> Option<Prevent<'a>> {
    if order.command.move_dest().is_some() {
        Some(if !path_exists(context, resolver, order) {
            Prevent::NoPath
        } else {
            if let Some(h2h) = context.orders_ref().iter().find(|o| is_head_to_head(o, order)) {
                println!("PREVENT needs to know if {} beats {} in head-to-head",
                         h2h,
                         order);
                match resolver.resolve(context, h2h) {
                    OrderState::Succeeds => Prevent::LostHeadToHead,
                    OrderState::Fails => {
                        Prevent::Prevents(support::find_successful_for(context, resolver, order))
                    }
                }
            } else {
                Prevent::Prevents(support::find_successful_for(context, resolver, order))
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

/// Gets the hold outcome for a given province.
///
/// The HOLD STRENGTH decision of an area must be calculated as follows:
/// In case the area is empty, the HOLD STRENGTH is zero. In case
/// the area contains a unit without a move order, then the HOLD STRENGTH
/// is one plus the number of orders that support this unit and that have a SUPPORT
/// decision with status 'given'. In case the area contains a unit
/// with a move order, then the HOLD STRENGTH is zero when the MOVE decision has status
/// 'moves' and the one when the MOVE decisions has status 'failed'.
///
/// [DATC 5.B.5](http://web.inter.nl.net/users/L.B.Kruijswijk/#5.B.5)
pub fn hold_result<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                      resolver: &mut ResolverState<'a, A>,
                                      region: &RegionKey)
                                      -> ProvinceHold<'a> {

    match context.find_order_to_province(region.province()) {
        None => ProvinceHold::Empty,
        Some(occupier) => {
            if occupier.command.is_move() {
                println!("HOLD needs to know if {} successfully vacated {}",
                         occupier,
                         region);
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
                                            -> DestResistance<'a> {
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

    // in the case that this order leaves the province, there's no need to
    // search for a dislodging order.
    if order.command.move_dest().is_some() && resolver.resolve(context, order).into() {
        None
    } else {
        let order_ref = context.orders_ref();
        for order in order_ref.into_iter().find(|o| is_move_to_province(o, &order.region)) {
            if resolver.resolve(context, order).into() {
                return Some(order);
            }
        }

        None
    }
}