//! This module contains the types needed to build a map of the world in Diplomacy.
//! Terminology in this module comes from the [DATC](http://web.inter.nl.net/users/L.B.Kruijswijk/).

mod border;

pub use self::border::Border;

/// A controllable area of the environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Province;

/// Differentiates regions within a province.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Coast {
    North,
    East,
    South,
    West,
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