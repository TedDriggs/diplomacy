use super::{Adjudicate, Context, MappedMainOrder, OrderState, ResolverState};
use crate::geo::{Map, ProvinceKey};
use crate::order::{Command, MainCommand};
use crate::UnitType;

/// Failure cases for convoy route lookup.
pub enum ConvoyRouteError {
    /// Only armies can be convoyed.
    CanOnlyConvoyArmy,

    /// Hold, support, and convoy orders cannot be convoyed.
    CanOnlyConvoyMove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvoyOutcome<'a> {
    /// The convoy order is invalid because the convoying unit is not at sea.
    NotAtSea,
    /// The convoying unit was dislodged by another move
    Dislodged(&'a MappedMainOrder),
    /// The convoy was failed to resolve a paradox
    Paradox,
    /// The convoy was not disrupted. This doesn't mean the move necessarily succeeded.
    NotDisrupted,
}

impl From<&'_ ConvoyOutcome<'_>> for OrderState {
    fn from(other: &ConvoyOutcome<'_>) -> Self {
        if other == &ConvoyOutcome::NotDisrupted {
            OrderState::Succeeds
        } else {
            OrderState::Fails
        }
    }
}

impl From<ConvoyOutcome<'_>> for OrderState {
    fn from(other: ConvoyOutcome<'_>) -> Self {
        (&other).into()
    }
}

/// Checks whether `convoy` is a valid convoy that will carry `mv_ord` from
/// its current location to the destination.
fn is_convoy_for(convoy: &MappedMainOrder, mv_ord: &MappedMainOrder) -> bool {
    match &convoy.command {
        MainCommand::Convoy(ref cm) => cm == mv_ord,
        _ => false,
    }
}

/// Find all routes from `origin` to `dest` given a set of valid convoys.
fn route_steps<'a>(
    map: &Map,
    convoys: &[&'a MappedMainOrder],
    origin: &ProvinceKey,
    dest: &ProvinceKey,
    working_path: Vec<&'a MappedMainOrder>,
) -> Vec<Vec<&'a MappedMainOrder>> {
    let adjacent_regions = map.find_bordering(origin);
    // if we've got a convoy going and there is one hop to the destination,
    // we've found a valid solution.
    if !working_path.is_empty() && adjacent_regions.iter().any(|&r| r == dest) {
        vec![working_path]
    } else {
        let mut paths = vec![];
        for convoy in convoys {
            // move to adjacent, and don't allow backtracking/cycles
            if !working_path.contains(&convoy) && adjacent_regions.contains(&&convoy.region) {
                let mut next_path = working_path.clone();
                next_path.push(&convoy);
                let mut steps =
                    route_steps(map, convoys, convoy.region.province(), dest, next_path);
                if !steps.is_empty() {
                    paths.append(&mut steps);
                }
            }
        }

        paths
    }
}

/// Finds all valid convoy routes for a given move order.
pub fn routes<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    state: &mut ResolverState<'a>,
    mv_ord: &MappedMainOrder,
) -> Result<Vec<Vec<&'a MappedMainOrder>>, ConvoyRouteError> {
    if mv_ord.unit_type == UnitType::Fleet {
        Err(ConvoyRouteError::CanOnlyConvoyArmy)
    } else if let Some(dst) = mv_ord.move_dest() {
        // Get the convoy orders that can ferry the provided move order and are
        // successful. Per http://uk.diplom.org/pouch/Zine/S2009M/Kruijswijk/DipMath_Chp6.htm
        // we resolve all convoy orders eagerly to avoid wild recursion during the depth-first
        // search.
        let mut convoy_steps = vec![];
        for order in ctx.orders() {
            if is_convoy_for(order, mv_ord) && state.resolve(ctx, order).into() {
                convoy_steps.push(order);
            }
        }

        Ok(route_steps(
            ctx.world_map,
            &convoy_steps,
            mv_ord.region.province(),
            dst.province(),
            vec![],
        ))
    } else {
        Err(ConvoyRouteError::CanOnlyConvoyMove)
    }
}

/// Determines if any valid convoy route exists for the given move order.
pub fn route_exists<'a>(
    ctx: &Context<'a, impl Adjudicate>,
    state: &mut ResolverState<'a>,
    mv_ord: &MappedMainOrder,
) -> bool {
    routes(ctx, state, mv_ord)
        .map(|r| !r.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod test {
    use crate::geo::{self, ProvinceKey, RegionKey};
    use crate::judge::MappedMainOrder;
    use crate::order::{ConvoyedMove, Order};
    use crate::UnitType;

    fn convoy(l: &str, f: &str, t: &str) -> MappedMainOrder {
        Order::new(
            "eng".into(),
            UnitType::Fleet,
            RegionKey::new(String::from(l), None),
            ConvoyedMove::new(
                RegionKey::new(String::from(f), None),
                RegionKey::new(String::from(t), None),
            )
            .into(),
        )
    }

    #[test]
    fn pathfinder() {
        let convoys = vec![
            convoy("ska", "lon", "swe"),
            convoy("eng", "lon", "swe"),
            convoy("nth", "lon", "swe"),
            convoy("nwg", "lon", "swe"),
        ];

        let routes = super::route_steps(
            geo::standard_map(),
            &convoys.iter().collect::<Vec<_>>(),
            &ProvinceKey::new("lon"),
            &ProvinceKey::new("swe"),
            vec![],
        );
        for r in &routes {
            println!("CHAIN");
            for o in r.iter() {
                println!("  {}", o);
            }
        }

        assert_eq!(2, routes.len());
    }
}
