use std::str::FromStr;

use parser::{Error, ErrorKind};
use ShortName;

/// The type of a military unit. Armies are convoyable land-based units; fleets
/// are sea-going units which are able to convoy armies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    /// A convoyable land-based unit which can traverse inland and coastal terrain.
    #[serde(rename="A")]
    Army,
    
    /// A sea-based unit which can traverse sea and coastal terrain.
    #[serde(rename="F")]
    Fleet,
}

impl FromStr for UnitType {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "a" | "army" => Ok(UnitType::Army),
            "f" | "fleet" => Ok(UnitType::Fleet),
            _ => Err(Error::new(ErrorKind::InvalidUnitType, s))
        }
    }
}

impl ShortName for UnitType {
    fn short_name(&self) -> String {
        String::from(match *self {
            UnitType::Army => "A",
            UnitType::Fleet => "F"
        })
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