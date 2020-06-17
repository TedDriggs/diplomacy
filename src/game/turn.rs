use std::collections::HashMap;

use crate::geo::RegionKey;
use crate::Nation;
use crate::order::{Order, MainCommand};
use crate::UnitType;

#[derive(Clone)]
pub struct Turn {
    units: HashMap<RegionKey, (Nation, UnitType)>,
    main_phase_orders: Vec<Order<RegionKey, MainCommand<RegionKey>>>,
}
