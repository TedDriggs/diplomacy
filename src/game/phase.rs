use std::cmp::{PartialOrd, Ordering};
use std::convert::From;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use ShortName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Phase {
    Main = 0,
    Retreat = 1,
    Build = 2,
}

impl ShortName for Phase {
    fn short_name(&self) -> String {
        match *self {
            Phase::Main => String::from("M"),
            Phase::Retreat => String::from("R"),
            Phase::Build => String::from("B"),
        }
    }
}

impl FromStr for Phase {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "M" => Ok(Phase::Main),
            "R" => Ok(Phase::Retreat),
            "B" => Ok(Phase::Build),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Season {
    Spring = 0,
    Fall = 1,
}

impl ShortName for Season {
    fn short_name(&self) -> String {
        match *self {
            Season::Spring => String::from("S"),
            Season::Fall => String::from("F"),
        }
    }
}

impl FromStr for Season {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "S" => Ok(Season::Spring),
            "F" => Ok(Season::Fall),
            _ => Err(()),
        }
    }
}

/// Represents a specific point in game time.
#[derive(Debug, Clone, PartialEq, Eq, Ord, Hash)]
pub struct Time(Season, usize, Phase);

impl Time {
    pub fn season(&self) -> Season {
        self.0
    }

    pub fn year(&self) -> usize {
        self.1
    }

    pub fn phase(&self) -> Phase {
        self.2
    }

    fn ord_id(&self) -> usize {
        (self.year() << 6) + ((self.season() as usize) << 3) + (self.phase() as usize)
    }
}

impl ShortName for Time {
    fn short_name(&self) -> String {
        format!("{}{}{}",
                self.season().short_name(),
                self.year(),
                self.phase().short_name())
    }
}

impl From<(Season, usize, Phase)> for Time {
    fn from((s, y, p): (Season, usize, Phase)) -> Self {
        Time(s, y, p)
    }
}

impl<'a> From<&'a Time> for usize {
    fn from(t: &Time) -> Self {
        t.ord_id()
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, rhs: &Time) -> Option<Ordering> {
        usize::partial_cmp(&self.ord_id(), &rhs.ord_id())
    }
}

impl FromStr for Time {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 6 {
            Ok(Time(s[0..1].parse()?,
                    s[1..5].parse().or(Err(()))?,
                    s[5..6].parse()?))
        } else {
            Err(())
        }
    }
}

impl Serialize for Time {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.short_name())
    }
}

impl Deserialize for Time {
    fn deserialize<D: Deserializer>(d: D) -> Result<Self, D::Error> {

        struct TimeVisitor;

        impl de::Visitor for TimeVisitor {
            type Value = Time;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f,
                       "a string representing a diplomacy time, such as 'S1901M'")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse().or(Err(E::custom(format!("Unable to parse time '{}'", v))))
            }
        }

        d.deserialize_str(TimeVisitor)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn parse_time() {
        assert_eq!(Time::from((Season::Spring, 1901, Phase::Main)),
                   "S1901M".parse().unwrap());
    }

    #[test]
    fn cmp() {
        let turns = vec!["S1901M", "S1901R", "F1901M", "F1901R", "F1901B"];

        let parsed = turns.iter().map(|t| Time::from_str(t).unwrap()).collect::<Vec<_>>();
        let mut sorted = parsed.clone();
        sorted.sort();
        assert_eq!(parsed, sorted);
    }
}