use crate::ShortName;
use crate::geo::{Location, ProvinceKey};
use crate::parser::{Error, ErrorKind};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

/// Differentiates regions within a province.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Coast {
    North,
    East,
    South,
    West,
}

impl ShortName for Coast {
    fn short_name(&self) -> Cow<'_, str> {
        use self::Coast::*;
        Cow::Borrowed(match *self {
            North => "(nc)",
            East => "(ec)",
            South => "(sc)",
            West => "(wc)",
        })
    }
}

impl FromStr for Coast {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nc" => Ok(Coast::North),
            "ec" => Ok(Coast::East),
            "sc" => Ok(Coast::South),
            "wc" => Ok(Coast::West),
            _ => Err(Error::new(ErrorKind::BadCoast, s)),
        }
    }
}

/// The type of environment (land, sea, coast). Armies cannot operate at sea, and
/// fleets cannot operate on land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Terrain {
    Land,
    Coast,
    Sea,
}

/// A space to which a unit can move. Provinces are made up of 1 or more regions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Region(ProvinceKey, Option<Coast>, Terrain);

impl Region {
    /// Creates a new region.
    pub fn new(
        province: impl Into<ProvinceKey>,
        coast: impl Into<Option<Coast>>,
        terrain: Terrain,
    ) -> Self {
        Region(province.into(), coast.into(), terrain)
    }

    /// Gets the parent province for the region.
    pub fn province(&self) -> &ProvinceKey {
        &self.0
    }

    /// Gets the coast of the region.
    pub fn coast(&self) -> Option<Coast> {
        self.1
    }

    /// Gets the region's terrain.
    pub fn terrain(&self) -> Terrain {
        self.2
    }
}

impl ShortName for Region {
    fn short_name(&self) -> Cow<'_, str> {
        if let Some(val) = self.coast() {
            Cow::Owned(format!(
                "{}{}",
                self.province().short_name(),
                val.short_name()
            ))
        } else {
            self.province().short_name()
        }
    }
}

impl Location for Region {
    type Province = ProvinceKey;

    fn province(&self) -> &Self::Province {
        &self.0
    }
}

/// An identifier that references a region.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RegionKey(
    ProvinceKey,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    Option<Coast>,
);

impl RegionKey {
    /// Creates a new region.
    pub fn new(province: impl Into<ProvinceKey>, coast: impl Into<Option<Coast>>) -> Self {
        RegionKey(province.into(), coast.into())
    }

    /// Gets the parent province for the region.
    pub fn province(&self) -> &ProvinceKey {
        &self.0
    }

    /// Gets the coast of the region.
    pub fn coast(&self) -> Option<Coast> {
        self.1
    }
}

impl<'a> From<&'a Region> for RegionKey {
    fn from(r: &'a Region) -> Self {
        RegionKey(r.0.clone(), r.1)
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

impl PartialEq<ProvinceKey> for RegionKey {
    fn eq(&self, rhs: &ProvinceKey) -> bool {
        &self.0 == rhs
    }
}

impl PartialEq<RegionKey> for ProvinceKey {
    fn eq(&self, rhs: &RegionKey) -> bool {
        rhs == self
    }
}

impl ShortName for RegionKey {
    fn short_name(&self) -> Cow<'_, str> {
        if let Some(val) = self.coast() {
            Cow::Owned(format!(
                "{}{}",
                self.province().short_name(),
                val.short_name()
            ))
        } else {
            self.province().short_name()
        }
    }
}

impl fmt::Display for RegionKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl Location for RegionKey {
    type Province = ProvinceKey;

    fn province(&self) -> &Self::Province {
        &self.0
    }
}

impl FromStr for RegionKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('(').collect::<Vec<_>>();

        // No parentheses means no coast.
        if parts.len() == 1 {
            Ok(RegionKey::new(String::from(parts[0]), None))
        } else if parts.len() == 2 {
            // Extract the coast identifier.
            let coast_id = parts[1].chars().take(2).collect::<String>();
            Ok(RegionKey::new(
                String::from(parts[0]),
                Coast::from_str(&coast_id)?,
            ))
        } else {
            Err(Error::new(ErrorKind::MalformedRegion, s))
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::{Coast, RegionKey};
    use crate::parser::ErrorKind;

    #[test]
    fn parse_coast() {
        assert_eq!(Coast::North, "nc".parse().expect("nc is a valid coast"));
        assert!(Coast::from_str("gc").is_err());
    }

    #[test]
    fn parse_region() {
        assert_eq!(
            RegionKey::new("aeg", None),
            RegionKey::from_str("aeg").expect("aeg is a valid region")
        );
        assert_eq!(
            &ErrorKind::BadCoast,
            RegionKey::from_str("foo(bar)").unwrap_err().kind()
        );
    }
}
