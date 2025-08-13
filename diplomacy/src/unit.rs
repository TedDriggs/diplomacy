use crate::parser::{Error, ErrorKind};
use crate::{geo::Location, geo::RegionKey, Command, Nation, Order, ShortName};
use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::str::FromStr;

/// The type of a military unit. Armies are convoyable land-based units; fleets
/// are sea-going units which are able to convoy armies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnitType {
    /// A convoyable land-based unit which can traverse inland and coastal terrain.
    #[cfg_attr(feature = "serde", serde(rename = "A"))]
    Army,

    /// A sea-based unit which can traverse sea and coastal terrain.
    #[cfg_attr(feature = "serde", serde(rename = "F"))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Create a view of the unit position that has a reference to its region.
    pub fn as_region_ref(&self) -> UnitPosition<'a, &L> {
        UnitPosition {
            unit: self.unit.clone(),
            region: &self.region,
        }
    }
}

impl<'a, L: Clone> UnitPosition<'a, &L> {
    /// Returns a [`UnitPosition`] that converts the region to an owned value by cloning.
    pub fn with_cloned_region(&self) -> UnitPosition<'a, L> {
        UnitPosition {
            unit: self.unit.clone(),
            region: (*self.region).clone(),
        }
    }
}

impl<'a> FromStr for UnitPosition<'a, RegionKey> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split_ascii_whitespace();
        let nation = if let Some(first_word) = words.next() {
            Nation::from(first_word.trim_end_matches(':'))
        } else {
            return Err(Error::new(ErrorKind::TooFewWords(3), s));
        };

        let unit_type = if let Some(word) = words.next() {
            UnitType::from_str(word)?
        } else {
            return Err(Error::new(ErrorKind::TooFewWords(3), s));
        };

        let region = if let Some(word) = words.next() {
            RegionKey::from_str(word)?
        } else {
            return Err(Error::new(ErrorKind::TooFewWords(3), s));
        };

        Ok(UnitPosition::new(
            Unit::new(Cow::Owned(nation), unit_type),
            region,
        ))
    }
}

impl fmt::Display for UnitPosition<'_, RegionKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} {}",
            self.unit.nation().short_name(),
            self.unit.unit_type().short_name(),
            self.region.short_name()
        )
    }
}

/// Knowledge of unit positions at a point in time.
pub trait UnitPositions<L: Location> {
    /// The current unit positions. The order is unspecified, but every unit should be
    /// returned exactly once.
    fn unit_positions(&self) -> Vec<UnitPosition<'_, &L>>;

    /// Get the unit currently occupying a province.
    ///
    /// This function returns the region of the occupier as well.
    fn find_province_occupier(&self, province: &L::Province) -> Option<UnitPosition<'_, &L>>;

    /// Get the unit currently occupying a specific region.
    fn find_region_occupier(&self, region: &L) -> Option<Unit<'_>>;
}

impl<'a, L: Location> UnitPositions<L> for Vec<UnitPosition<'a, L>> {
    fn unit_positions(&self) -> Vec<UnitPosition<'_, &L>> {
        self.iter()
            .map(|pos| UnitPosition::new(pos.unit.clone(), &pos.region))
            .collect()
    }

    fn find_province_occupier(&self, province: &L::Province) -> Option<UnitPosition<'_, &L>> {
        self.iter()
            .find(|pos| pos.region.province() == province)
            .map(|pos| UnitPosition::new(pos.unit.clone(), &pos.region))
    }

    fn find_region_occupier(&self, region: &L) -> Option<Unit<'_>> {
        self.iter()
            .find(|pos| pos.region == *region)
            .map(|pos| pos.unit.clone())
    }
}

/// Infer unit positions from a collection of orders. This assumes orders are trustworthy
/// and complete:
///
/// 1. There is an order for every unit.
/// 2. Orders are only issued to units that exist.
/// 3. There is at most one order per province.
impl<L: Location, C: Command<L>> UnitPositions<L> for Vec<Order<L, C>> {
    fn unit_positions(&self) -> Vec<UnitPosition<'_, &L>> {
        self.iter().map(UnitPosition::from).collect()
    }

    fn find_province_occupier(&self, province: &L::Province) -> Option<UnitPosition<'_, &L>> {
        self.iter()
            .find(|ord| ord.region.province() == province)
            .map(UnitPosition::from)
    }

    fn find_region_occupier(&self, region: &L) -> Option<Unit<'_>> {
        self.iter()
            .find(|ord| ord.region == *region)
            .map(Unit::from)
    }
}

/// Infer unit positions from a collection of orders. This assumes orders are trustworthy
/// and complete:
///
/// 1. There is an order for every unit.
/// 2. Orders are only issued to units that exist.
/// 3. There is at most one order per province.
impl<L: Location, C: Command<L>> UnitPositions<L> for Vec<&'_ Order<L, C>> {
    fn unit_positions(&self) -> Vec<UnitPosition<'_, &L>> {
        self.iter().copied().map(UnitPosition::from).collect()
    }

    fn find_province_occupier(&self, province: &L::Province) -> Option<UnitPosition<'_, &L>> {
        self.iter()
            .copied()
            .find(|ord| ord.region.province() == province)
            .map(UnitPosition::from)
    }

    fn find_region_occupier(&self, region: &L) -> Option<Unit<'_>> {
        self.iter()
            .copied()
            .find(|ord| ord.region == *region)
            .map(Unit::from)
    }
}

impl<L, K, H> UnitPositions<L> for HashMap<K, UnitPosition<'_, &L>, H>
where
    L: Location + Eq,
    L::Province: Eq + Hash,
    K: Borrow<L::Province> + Eq + Hash,
    H: BuildHasher,
{
    fn unit_positions(&self) -> Vec<UnitPosition<'_, &L>> {
        self.values().cloned().collect()
    }

    fn find_province_occupier(&self, province: &L::Province) -> Option<UnitPosition<'_, &L>> {
        self.get(province).cloned()
    }

    fn find_region_occupier(&self, region: &L) -> Option<Unit<'_>> {
        let up = self.get(region.province())?;
        if up.region == region {
            Some(up.unit.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{UnitPosition, UnitType};
    use crate::{geo::RegionKey, Nation};

    #[test]
    fn parse_unit_type() {
        assert_eq!(Ok(UnitType::Army), "Army".parse());
        assert_eq!(Ok(UnitType::Fleet), "Fleet".parse());
        assert_eq!(Ok(UnitType::Army), "A".parse());
        assert_eq!(Ok(UnitType::Fleet), "F".parse());
        assert_eq!(Ok(UnitType::Army), "a".parse());
        assert_eq!(Ok(UnitType::Fleet), "f".parse());
    }

    #[test]
    fn parse_unit_position() {
        let pos: UnitPosition<'_, RegionKey> = "FRA: F bre".parse().unwrap();
        assert_eq!(pos.nation(), &Nation::from("FRA"));
    }
}
