use ShortName;
use geo::Province;

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

/// The area which a unit can move to. Provinces are made up of 1 or more regions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region<'a>(&'a Province, Option<Coast>, Terrain);

impl<'a> Region<'a> {
    /// Creates a new region.
    pub fn new<IC : Into<Option<Coast>>>(province: &'a Province, coast: IC, terrain: Terrain) -> Self {
        Region(province, coast.into(), terrain)
    }
    
    /// Gets the parent province for the region.
    pub fn province(&self) -> &'a Province {
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

impl<'a> ShortName for Region<'a> {
    fn short_name(&self) -> String {
        format!("{}{}", self.province().short_name(), match self.coast() {
            &Some(ref val) => val.short_name(),
            &None => String::from("")
        })
    }
}

impl<'a> From<&'a Region<'a>> for &'a Province {
    fn from(r: &'a Region<'a>) -> Self {
        r.province()
    }
}