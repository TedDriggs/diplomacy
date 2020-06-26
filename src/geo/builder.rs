//! Contains structs needed to assemble a `geo::Map` instance.
//!
//! # Usage
//! 1. Create a `ProvinceRegistry` and add all provinces.
//! 1. Call `ProvinceRegistry::finish()` and then add all regions to that function's return.
//! 1. Call `RegionRegistry::finish()` and add all borders to that function's return.
//! 1. Call `BorderRegistry::finish()` and use the resulting map.

use super::{Border, Coast, Map, Province, ProvinceKey, Region, Terrain};
use crate::ShortName;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MapError {
    ProvinceNotFound,
    RegionNotFound,
    IncompatibleBorderTerrain,
}

/// A collection of provinces that validates on insertion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProvinceRegistry {
    provinces: HashMap<String, Province>,
}

impl ProvinceRegistry {
    /// Inserts a new province into the registry.
    pub fn register(&mut self, p: Province) -> Result<(), MapError> {
        self.provinces.insert(p.short_name().into_owned(), p);
        Ok(())
    }

    pub fn finish(self) -> RegionRegistry {
        RegionRegistry::new(self)
    }
}

/// A validating collection of regions and provinces. All regions must
/// belong to provinces registered with the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionRegistry {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region>,
}

impl RegionRegistry {
    pub fn new(provinces: ProvinceRegistry) -> Self {
        RegionRegistry {
            provinces: provinces.provinces,
            regions: HashMap::new(),
        }
    }

    /// Add a region to the map after validation.
    ///
    /// This function validates that:
    ///
    /// 1. `province_name` identifies a known province.
    pub fn register(
        &mut self,
        province_name: &str,
        coast: impl Into<Option<Coast>>,
        terrain: Terrain,
    ) -> Result<(), MapError> {
        let region = Region::new(self.find_province(province_name)?, coast, terrain);
        self.regions
            .insert(region.short_name().into_owned(), region);
        Ok(())
    }

    fn find_province(&self, k: &str) -> Result<ProvinceKey, MapError> {
        self.provinces
            .get(k)
            .map(ProvinceKey::from)
            .ok_or(MapError::ProvinceNotFound)
    }

    /// Stops accepting regions and starts accepting borders.
    pub fn finish(self) -> BorderRegistry {
        BorderRegistry::new(self)
    }
}

impl From<ProvinceRegistry> for RegionRegistry {
    fn from(pr: ProvinceRegistry) -> Self {
        pr.finish()
    }
}

/// A collection of provinces, regions, and borders that allows border insertion after validation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BorderRegistry {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region>,
    borders: Vec<Border>,
}

impl BorderRegistry {
    /// Creates a new instance from a `RegionRegistry`.
    pub fn new(builder: RegionRegistry) -> Self {
        BorderRegistry {
            provinces: builder.provinces,
            regions: builder.regions,
            borders: vec![],
        }
    }

    /// Register a border between two regions identified by key after validation.
    ///
    /// This function validates that:
    ///
    /// 1. `r1` and `r2` are keys to known regions.
    /// 1. Border terrain is valid compared to the two regions.
    pub fn register(&mut self, r1: &str, r2: &str, terrain: Terrain) -> Result<(), MapError> {
        {
            let rk1 = self.find_region(r1)?;
            let rk2 = self.find_region(r2)?;

            BorderRegistry::validate_terrain(rk1.terrain(), rk2.terrain(), terrain)?;
        }

        self.borders.push(Border::new(
            r1.parse().unwrap(),
            r2.parse().unwrap(),
            terrain,
        ));
        Ok(())
    }

    /// Convert the builder to an immutable Map instance.
    pub fn finish(self) -> Map {
        Map::from(self)
    }

    /// Get a view of the contents in a format that `Map` can use.
    pub fn contents(
        self,
    ) -> (
        HashMap<String, Province>,
        HashMap<String, Region>,
        Vec<Border>,
    ) {
        (self.provinces, self.regions, self.borders)
    }

    /// Find a region by its canonical short name.
    fn find_region<'a>(&'a self, short_name: &str) -> Result<&'a Region, MapError> {
        if let Some(region) = self.regions.get(short_name) {
            Ok(region)
        } else {
            Err(MapError::RegionNotFound)
        }
    }

    fn validate_terrain(rt1: Terrain, rt2: Terrain, bt: Terrain) -> Result<(), MapError> {
        use crate::geo::Terrain::*;

        if ((rt1 == Sea || rt2 == Sea) && bt != Sea)
            || ((rt1 == Land || rt2 == Land) && bt != Land)
            || ((rt1 == Sea && rt2 == Land) || (rt1 == Land && rt2 == Sea))
        {
            Err(MapError::IncompatibleBorderTerrain)
        } else {
            Ok(())
        }
    }
}

impl From<RegionRegistry> for BorderRegistry {
    fn from(rr: RegionRegistry) -> Self {
        rr.finish()
    }
}
