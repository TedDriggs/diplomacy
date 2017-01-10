use super::prelude::*;
use super::support;

impl<'a> Border<'a> {
    fn supports(&self, unit_type: &UnitType) -> bool {
        match self.terrain() {
            &Terrain::Land => unit_type == &UnitType::Army,
            &Terrain::Sea => unit_type == &UnitType::Fleet,
            &Terrain::Coast => true,
        }
    }
}

fn path_exists<'a, A: Adjudicate>(context: &ResolverContext<'a>,
                                  resolver: &ResolverState<'a, A>,
                                  order: &MappedMainOrder<'a>)
                                  -> bool {
    match order.command {
        MainCommand::Move(ref dst) => {
            context.world_map
                .find_border_between(&order.region, dst)
                .map(|b| b.supports(&order.unit_type))
                .unwrap_or(false)
        }
        _ => true,
    }
}

fn is_move_and_dest_is_not<'a, 'b, D: Into<&'b Province>>(o: &MappedMainOrder<'a>, d: D) -> bool {
    match o.command {
        MainCommand::Move(ref dst) => dst.province() != d.into(),
        _ => false,
    }
}

fn move_outcome<'a, A: Adjudicate>(context: &'a ResolverContext<'a>,
                                   resolver: &'a mut ResolverState<'a, A>,
                                   order: &'a MappedMainOrder<'a>)
                                   -> Option<Attack<'a>> {
    use order::MainCommand::*;
    match order.command {
        Move(..) if !path_exists(context, resolver, order) => Some(Attack::NoPath),
        Move(ref dst) => {
            Some({

                let dst_occupant =
                    context.orders.iter().find(|occ| occ.region.province() == dst.province());
                let supports = support::find_successful_for(context, resolver, order);
                match dst_occupant {
                    None => Attack::AgainstVacant(supports),
                    Some(ref occ) => {
                        // In the case that the occupier is leaving, this is a follow-in
                        if is_move_and_dest_is_not(occ, order.region) &&
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
        Hold | Support(..) | Convoy(..) => None,
    }
}