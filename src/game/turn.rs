use std::collections::HashMap;

use geo::{Map, RegionKey};
use Nation;
use order::{Order, Command, MainCommand};
use UnitType;

pub struct Turn<'a> {
    map: &'a Map,
    units: HashMap<RegionKey, (Nation, UnitType)>,
    main_phase_orders: Vec<Order<RegionKey, MainCommand<RegionKey>>>,
}

impl<'a> Turn<'a> {
    
}