use std::collections::HashMap;

use super::{Border, Province, ProvinceKey, Region, RegionKey, Terrain};
use crate::geo::builder::BorderRegistry;

/// A collection of provinces, their constituent regions, and the interconnecting borders.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Map {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region>,
    borders: Vec<Border>,
}

impl Map {
    /// Create an instance by consuming a builder.
    pub fn from(builder: BorderRegistry) -> Self {
        let (prov, reg, bord) = builder.contents();
        Map {
            provinces: prov,
            regions: reg,
            borders: bord,
        }
    }

    /// Find a region by its canonical short name.
    pub fn find_region<'a>(&'a self, short_name: &str) -> Option<&'a Region> {
        self.regions.get(short_name)
    }

    /// Get all borders with a region.
    pub fn borders_containing<'a, L: PartialEq<RegionKey>>(&'a self, r: &L) -> Vec<&Border> {
        self.borders.iter().filter(|&b| b.contains(r)).collect()
    }

    /// Gets the set of regions which connect to the specified region. If `terrain`
    /// is provided, only borders matching that terrain will be provided.
    pub fn find_bordering<'a>(
        &'a self,
        region: &impl PartialEq<RegionKey>,
        terrain: impl Into<Option<Terrain>>,
    ) -> Vec<&RegionKey> {
        let ter = terrain.into();
        self.borders_containing(region)
            .iter()
            .filter(|b| ter.map(|t| t == b.terrain()).unwrap_or(true))
            .filter_map(|b| b.dest_from(region))
            .collect()
    }

    /// Get a border between two regions, if one exists.
    pub fn find_border_between(&self, r1: &RegionKey, r2: &RegionKey) -> Option<&Border> {
        self.borders.iter().find(|b| b.connects(r1, r2))
    }

    /// Finds all borders connecting a region to a given province.
    /// Used for support and convoy cases.
    pub fn find_borders_between<'a>(&'a self, r1: &RegionKey, p2: &ProvinceKey) -> Vec<&Border> {
        self.borders.iter().filter(|b| b.connects(r1, p2)).collect()
    }
}
