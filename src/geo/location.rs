use ShortName;

use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

/// An addressable location in the Diplomacy world.
/// This trait is used during order parsing and mapping to allow for
/// orders that reference regions by name rather than by reference.
pub trait Location
    : ShortName + Clone + Debug + PartialEq + Eq + Hash + Deserialize + Serialize
    {
    type Province: PartialEq;

    fn province(&self) -> &Self::Province;
}