use std::collections::HashMap;

use geo::RegionKey;
use Nation;
use order::{Order, MainCommand};
use UnitType;

#[derive(Clone)]
pub struct Turn {
    units: HashMap<RegionKey, (Nation, UnitType)>,
    main_phase_orders: Vec<Order<RegionKey, MainCommand<RegionKey>>>,
}

impl Turn {}