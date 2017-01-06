use geo::Location;
use order::{Order, Command};
use ShortName;

use std::str::FromStr;

/// An order that has not yet been resolved against a world map.
pub type UnmappedOrder<C : Command<String>> = Order<String, C>;

impl ShortName for String {
    fn short_name(&self) -> String {
        self.clone()
    }
}

impl Location for String {
    
}

impl<C : Command<String>> FromStr for UnmappedOrder<C> {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}