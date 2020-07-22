//! This module contains the types needed to build a map of the world in Diplomacy.
//! Terminology in this module comes from the [DATC](http://web.inter.nl.net/users/L.B.Kruijswijk/).

mod border;
mod location;
mod map;
mod province;
mod region;
mod standard;

pub mod builder;

pub use self::border::Border;
pub use self::location::Location;
pub use self::map::Map;
pub use self::province::{Province, ProvinceKey, SupplyCenter};
pub use self::region::{Coast, Region, RegionKey, Terrain};
pub use self::standard::standard_map;
