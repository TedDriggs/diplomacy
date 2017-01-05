use super::{
    Province,
    Region,
    Border,
    Coast,
};

use std::collections::HashMap;
use std::error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegionLookupError<'a> {
    Malformed,
    ProvinceNotFound,
    CoastNotFound(&'a Province, Coast)
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

pub struct World<'a> {
    provinces: HashMap<String, Province>,
    regions: HashMap<String, Region<'a>>,
    borders: Vec<Border<'a>>,
}

impl<'a> World<'a> {
    
    /// Find a region by its canonical short name.
    pub fn find_region(&self, short_name: &str) -> Result<&'a Region<'a>, RegionLookupError<'a>> {
        if let Some(region) = self.regions.get(short_name) {
            Ok(region)
        } else {
            RegionLookupError::Malformed,
        }
    }
    
    /// Find a province by its canonical short name.
    pub fn find_province(&self, short_name: &str) -> Option<&'a Province> {
        self.provinces.get(short_name)
    }
    
    /// Get all borders with a region.
    pub fn borders_with(&self, r: &Region<'a>) -> Vec<Border<'a>> {
        self.borders.iter().filter(|b| b.contains(r))
    }
}