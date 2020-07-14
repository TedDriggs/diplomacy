use crate::parser::{Error, ErrorKind};
use crate::{geo::ProvinceKey, geo::RegionKey, Command, Nation, Order, ShortName};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;

/// The type of a military unit. Armies are convoyable land-based units; fleets
/// are sea-going units which are able to convoy armies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    /// A convoyable land-based unit which can traverse inland and coastal terrain.
    #[serde(rename = "A")]
    Army,

    /// A sea-based unit which can traverse sea and coastal terrain.
    #[serde(rename = "F")]
    Fleet,
}

impl FromStr for UnitType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "a" | "army" => Ok(UnitType::Army),
            "f" | "fleet" => Ok(UnitType::Fleet),
            _ => Err(Error::new(ErrorKind::InvalidUnitType, s)),
        }
    }
}

impl ShortName for UnitType {
    fn short_name(&self) -> std::borrow::Cow<'_, str> {
        Cow::Borrowed(match *self {
            UnitType::Army => "A",
            UnitType::Fleet => "F",
        })
    }
}

/// A specific unit that belongs to a nation.
///
/// Diplomacy doesn't invest much in unit identity across turns, so there's no difference
/// between one Austrian fleet and another.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit<'a> {
    nation: Cow<'a, Nation>,
    unit_type: UnitType,
}

impl<'a> Unit<'a> {
    pub fn new(nation: impl Into<Cow<'a, Nation>>, unit_type: UnitType) -> Self {
        Self {
            nation: nation.into(),
            unit_type,
        }
    }

    pub fn nation(&self) -> &Nation {
        self.nation.as_ref()
    }

    pub fn unit_type(&self) -> UnitType {
        self.unit_type
    }
}

/// A unit's instantaneous position in a region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitPosition<'a, L = &'a RegionKey> {
    pub unit: Unit<'a>,
    /// The unit's current location.
    pub region: L,
}

impl<'a, L> UnitPosition<'a, L> {
    /// Create a new unit at a given position.
    pub fn new(unit: Unit<'a>, region: L) -> Self {
        Self { unit, region }
    }

    pub fn nation(&self) -> &Nation {
        self.unit.nation()
    }
}

/// Knowledge of unit positions at a point in time.
pub trait UnitPositions {
    /// The current unit positions. The order is unspecified, but every unit should be
    /// returned exactly once.
    fn unit_positions(&self) -> Vec<UnitPosition<'_>>;

    /// Get the unit currently occupying a province.
    ///
    /// This function returns the region of the occupier as well.
    fn find_province_occupier(&self, province: &ProvinceKey) -> Option<UnitPosition<'_>>;

    /// Get the unit currently occupying a specific region.
    fn find_region_occupier(&self, region: &RegionKey) -> Option<Unit<'_>>;
}

/// Infer unit positions from a collection of orders. This assumes orders are trustworthy
/// and complete:
///
/// 1. There is an order for every unit.
/// 2. Orders are only issued to units that exist.
/// 3. There is at most one order per province.
impl<C: Command<RegionKey>> UnitPositions for Vec<Order<RegionKey, C>> {
    fn unit_positions(&self) -> Vec<UnitPosition<'_>> {
        self.iter().map(UnitPosition::from).collect()
    }

    fn find_province_occupier(&self, province: &ProvinceKey) -> Option<UnitPosition<'_>> {
        self.iter()
            .find(|ord| ord.region.province() == province)
            .map(UnitPosition::from)
    }

    fn find_region_occupier(&self, region: &RegionKey) -> Option<Unit<'_>> {
        self.iter()
            .find(|ord| ord.region == *region)
            .map(Unit::from)
    }
}

#[cfg(test)]
mod test {
    use super::UnitType;

    #[test]
    fn parse_unit_type() {
        assert_eq!(Ok(UnitType::Army), "Army".parse());
        assert_eq!(Ok(UnitType::Fleet), "Fleet".parse());
        assert_eq!(Ok(UnitType::Army), "A".parse());
        assert_eq!(Ok(UnitType::Fleet), "F".parse());
        assert_eq!(Ok(UnitType::Army), "a".parse());
        assert_eq!(Ok(UnitType::Fleet), "f".parse());
    }
}
