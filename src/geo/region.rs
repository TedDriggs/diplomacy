use ShortName;
use geo::{ProvinceKey, Location};

use std::convert::From;

/// Differentiates regions within a province.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Coast {
    North,
    East,
    South,
    West,
}

impl ShortName for Coast {
    fn short_name(&self) -> String {
        use self::Coast::*;
        String::from(match *self {
            North => "(nc)",
            East => "(ec)",
            South => "(sc)",
            West => "(wc)",
        })
    }
}

/// The type of environment (land, sea, coast). Armies cannot operate at sea, and
/// fleets cannot operate on land.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Terrain {
    Land,
    Coast,
    Sea,
}

/// A space to which a unit can move. Provinces are made up of 1 or more regions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region(ProvinceKey, Option<Coast>, Terrain);

impl Region {
    /// Creates a new region.
    pub fn new<IP: Into<ProvinceKey>, IC: Into<Option<Coast>>>(province: IP,
                                                               coast: IC,
                                                               terrain: Terrain)
                                                               -> Self {
        Region(province.into(), coast.into(), terrain)
    }

    /// Gets the parent province for the region.
    pub fn province(&self) -> &ProvinceKey {
        &self.0
    }

    /// Gets the coast of the region.
    pub fn coast(&self) -> &Option<Coast> {
        &self.1
    }

    /// Gets the region's terrain.
    pub fn terrain(&self) -> &Terrain {
        &self.2
    }
}

impl ShortName for Region {
    fn short_name(&self) -> String {
        format!("{}{}",
                self.province().short_name(),
                match self.coast() {
                    &Some(ref val) => val.short_name(),
                    &None => String::from(""),
                })
    }
}

impl Location for Region {}

/// An identifier that references a region.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionKey(ProvinceKey, Option<Coast>);

impl RegionKey {
    /// Creates a new region.
    pub fn new<IP: Into<ProvinceKey>, IC: Into<Option<Coast>>>(province: IP, coast: IC) -> Self {
        RegionKey(province.into(), coast.into())
    }

    /// Gets the parent province for the region.
    pub fn province(&self) -> &ProvinceKey {
        &self.0
    }

    /// Gets the coast of the region.
    pub fn coast(&self) -> &Option<Coast> {
        &self.1
    }
}

impl<'a> From<&'a Region> for RegionKey {
    fn from(r: &'a Region) -> Self {
        RegionKey(r.0.clone(), r.1.clone())
    }
}

impl<'a> From<&'a RegionKey> for &'a ProvinceKey {
    fn from(r: &'a RegionKey) -> Self {
        r.province()
    }
}

impl PartialEq<Region> for RegionKey {
    fn eq(&self, other: &Region) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl ShortName for RegionKey {
    fn short_name(&self) -> String {
        format!("{}{}",
                self.province().short_name(),
                match self.coast() {
                    &Some(ref val) => val.short_name(),
                    &None => String::from(""),
                })
    }
}

impl Location for RegionKey {}