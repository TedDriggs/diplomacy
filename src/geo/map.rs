use super::{Province, Region, RegionKey, Border, Coast, Terrain, ProvinceKey};

use ShortName;

use std::collections::HashMap;
use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegionLookupError {
    Malformed,
    ProvinceNotFound,
    CoastNotFound(ProvinceKey, Coast),
}

impl error::Error for RegionLookupError {
    fn description(&self) -> &str {
        use self::RegionLookupError::*;
        match *self {
            Malformed => "Malformed region short name",
            ProvinceNotFound => "Province not found",
            CoastNotFound(..) => "Coast not found",
        }
    }
}

impl fmt::Display for RegionLookupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RegionLookupError::*;
        match *self {
            Malformed => write!(f, "Malformed region short name"),
            ProvinceNotFound => write!(f, "Province not found"),
            CoastNotFound(ref p, ref c) => write!(f, "{:?} does not have a {:?} coast", p, c),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapBuilder {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region>,
    borders: Vec<Border>,
}

impl MapBuilder {
    /// Add a province to the map, returning a pointer that is valid for the life of the builder.
    pub fn register_province(&mut self, p: Province) {
        self.provinces.insert(p.short_name(), p);
    }

    /// Add a region to the map, returning a pointer that is valid for the life of the builder.
    pub fn register_region<IC: Into<Option<Coast>>>(&mut self,
                                                    province_name: &str,
                                                    coast: IC,
                                                    terrain: Terrain)
                                                    -> Result<(), ()> {
        let region = Region::new(ProvinceKey::new(province_name), coast, terrain);
        self.regions.insert(region.short_name(), region);
        Ok(())
    }

    pub fn register_border(&mut self, b: Border) {
        self.borders.push(b);
        &self.borders[self.borders.len() - 1];
    }

    /// Find a region by its canonical short name.
    pub fn find_region<'a>(&'a self, short_name: &str) -> Result<&'a Region, RegionLookupError> {
        if let Some(region) = self.regions.get(short_name) {
            Ok(region)
        } else {
            Err(RegionLookupError::Malformed)
        }
    }

    /// Find a province by its canonical short name.
    pub fn find_province<'a>(&'a self, short_name: &str) -> Option<&'a Province> {
        self.provinces.get(short_name)
    }

    /// Convert the builder to an immutable Map instance.
    pub fn finish(self) -> Map {
        Map::new(self)
    }
}

/// A collection of provinces, their constituent regions, and the interconnecting borders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Map {
    builder: MapBuilder,
}

impl Map {
    pub fn new(builder: MapBuilder) -> Self {
        Map { builder: builder }
    }

    /// Find a region by its canonical short name.
    pub fn find_region<'a>(&'a self, short_name: &str) -> Result<&'a Region, RegionLookupError> {
        if let Some(region) = self.builder.regions.get(short_name) {
            Ok(region)
        } else {
            Err(RegionLookupError::Malformed)
        }
    }

    /// Find a province by its canonical short name.
    pub fn find_province<'a>(&'a self, short_name: &str) -> Option<&'a Province> {
        self.builder.provinces.get(short_name)
    }

    /// Get all borders with a region.
    pub fn borders_containing<'a, L: PartialEq<RegionKey>>(&'a self, r: &L) -> Vec<&Border> {
        self.builder.borders.iter().filter(|&b| b.contains(r)).collect()
    }

    /// Gets the set of regions which connect to the specified region. If `terrain`
    /// is provided, only borders matching that terrain will be provided.
    pub fn find_bordering<'a, RK: PartialEq<RegionKey>, IT: Into<Option<Terrain>>>(&'a self,
                                                         region: &RK,
                                                         terrain: IT)
                                                         -> Vec<&RegionKey> {
        let ter = terrain.into();
        self.borders_containing(region)
            .iter()
            .filter(|b| ter.as_ref().map(|t| t == b.terrain()).unwrap_or(true))
            .filter_map(|b| b.dest_from(region))
            .collect()
    }

    /// Get a border between two regions, if one exists.
    pub fn find_border_between<'a, 'b, IR1: Into<&'b RegionKey>, IR2: Into<&'b RegionKey>>
        (&'a self,
         r1: IR1,
         r2: IR2)
         -> Option<&Border> {
        let rk1 = r1.into();
        let rk2 = r2.into();
        self.builder.borders.iter().find(|b| b.connects(rk1, rk2))
    }
}