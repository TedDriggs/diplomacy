use super::{
    Province,
    Region,
    Border,
    Coast,
};

use std::collections::HashMap;
use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegionLookupError<'a> {
    Malformed,
    ProvinceNotFound,
    CoastNotFound(&'a Province, Coast),
}

impl<'a> error::Error for RegionLookupError<'a> {
    fn description(&self) -> &str {
        use self::RegionLookupError::*;
        match *self {
            Malformed => "Malformed region short name",
            ProvinceNotFound => "Province not found",
            CoastNotFound(..) => "Coast not found",
        }
    }
}

impl<'a> fmt::Display for RegionLookupError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::RegionLookupError::*;
        match *self {
            Malformed => write!(f, "Malformed region short name"),
            ProvinceNotFound => write!(f, "Province not found"),
            CoastNotFound(ref p, ref c) => write!(f, "{:?} does not have a {:?} coast", p, c)
        }
    }
}

/// A collection of provinces, their constituent regions, and the interconnecting borders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map<'a> {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region<'a>>,
    borders: Vec<Border<'a>>,
}

impl<'a> Map<'a> {
    
    /// Find a region by its canonical short name.
    pub fn find_region(&'a self, short_name: &str) -> Result<&'a Region<'a>, RegionLookupError<'a>> {
        if let Some(region) = self.regions.get(short_name) {
            Ok(region)
        } else {
            Err(RegionLookupError::Malformed)
        }
    }
    
    /// Find a province by its canonical short name.
    pub fn find_province(&'a self, short_name: &str) -> Option<&'a Province> {
        self.provinces.get(short_name)
    }
    
    /// Get all borders with a region.
    pub fn borders_with(&'a self, r: &Region<'a>) -> Vec<&Border<'a>> {
        self.borders.iter().filter(|b| b.contains(r)).collect()
    }
    
    /// Get a border between two regions, if one exists.
    pub fn find_border_between(&'a self, r1: &Region<'a>, r2: &'a Region<'a>) -> Option<&Border<'a>> {
        self.borders.iter().find(|b| b.connects(r1, r2))
    }
}