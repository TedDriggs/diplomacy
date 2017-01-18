use super::prelude::*;
use geo::{Map, RegionKey, ProvinceKey};

/// Failure cases for convoy route lookup.
pub enum ConvoyRouteError {
    /// Only armies can be convoyed.
    CanOnlyConvoyArmy,

    /// Hold, support, and convoy orders cannot be convoyed.
    CanOnlyConvoyMove,
}

/// Checks whether `convoy` is a valid convoy that will carry `move_order` from
/// its current location to the destination.
fn is_convoy_for(convoy: &MappedMainOrder, move_order: &MappedMainOrder) -> bool {
    match &convoy.command {
        &MainCommand::Convoy(ref cm) => cm == move_order,
        _ => false,
    }
}

pub fn route_steps<'a>(map: &Map,
                       convoys: Vec<&'a MappedMainOrder>,
                       origin: &ProvinceKey,
                       dest: &ProvinceKey,
                       working_path: Vec<&'a MappedMainOrder>)
                       -> Vec<Vec<&'a MappedMainOrder>> {

    let adjacent_regions = map.find_bordering(origin, None);
    if adjacent_regions.iter().find(|&&r| r == dest).is_some() {
        vec![working_path]
    } else {
        let mut paths = vec![];
        for convoy in &convoys {
            if !working_path.contains(&convoy) && adjacent_regions.contains(&&convoy.region) {
                let mut next_path = working_path.clone();
                next_path.push(&convoy);
                let mut steps = route_steps(map,
                                            convoys.clone(),
                                            (&convoy.region).into(),
                                            dest,
                                            next_path);
                if !steps.is_empty() {
                    paths.append(&mut steps);
                }
            }
        }

        paths
    }
}

pub fn find_routes<'a, A: Adjudicate>
    (ctx: &'a ResolverContext<'a>,
     state: &mut ResolverState<'a, A>,
     move_order: &MappedMainOrder)
     -> Result<Vec<Vec<&'a MappedMainOrder>>, ConvoyRouteError> {
    if move_order.unit_type == UnitType::Fleet {
        Err(ConvoyRouteError::CanOnlyConvoyArmy)
    } else {
        use order::MainCommand::*;
        match move_order.command {
            Hold | Support(..) | Convoy(..) => Err(ConvoyRouteError::CanOnlyConvoyMove),
            Move(ref dst) => {
                let mut convoy_steps = vec![];
                for order in ctx.orders_ref() {
                    if is_convoy_for(order, move_order) && state.resolve(ctx, order).into() {
                        convoy_steps.push(order);
                    }
                }

                Ok(route_steps(ctx.world_map,
                               convoy_steps,
                               (&move_order.region).into(),
                               dst.into(),
                               vec![]))
            }
        }
    }
}

/// Determines if any valid convoy route exists for the given move order.
pub fn route_exists<'a, A: Adjudicate>(ctx: &'a ResolverContext<'a>,
                                       state: &mut ResolverState<'a, A>,
                                       move_order: &MappedMainOrder)
                                       -> bool {
    find_routes(ctx, state, move_order).map(|r| !r.is_empty()).unwrap_or(false)
}

#[cfg(test)]
mod test {
    use super::super::prelude::*;
    use super::*;
    use order::Order;
    use order::ConvoyedMove;
    use geo::{self, RegionKey, ProvinceKey};
    use Nation;

    fn convoy(l: &str, f: &str, t: &str) -> MappedMainOrder {
        Order::new(Nation("eng".into()),
                   UnitType::Fleet,
                   RegionKey::no_coast(String::from(l)),
                   ConvoyedMove::new(RegionKey::no_coast(String::from(f)),
                                     RegionKey::no_coast(String::from(t)))
                       .into())
    }

    #[test]
    fn pathfinder() {
        let convoys = vec![
            convoy("ska", "lon", "swe"),
            convoy("eng", "lon", "swe"),
            convoy("nth", "lon", "swe"),
            convoy("nwg", "lon", "swe"),
        ];

        let routes = route_steps(geo::standard_map(),
                                 convoys.iter().collect(),
                                 &ProvinceKey::new("lon"),
                                 &ProvinceKey::new("swe"),
                                 vec![]);
        for r in &routes {
            println!("CHAIN");
            for o in r.iter() {
                println!("  {}", o);
            }
        }

        assert_eq!(2, routes.len());
    }
}