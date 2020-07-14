use crate::Nation;
use crate::ShortName;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The supply-center nature of a province. This information is used in the build phase
/// to determine how many units a nation can sustain and where new units can be built.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SupplyCenter {
    /// The province does not grant a build to whoever controls it.
    None,
    /// The province grants a build to its controller, but cannot be used as a build target.
    Neutral,
    /// The province grants a build to its controller, and can be used as a build target by the
    /// specified nation.
    Home(Nation),
}

/// A controllable area of the environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Province {
    pub short_name: String,
    pub supply_center: SupplyCenter,
}

impl Province {
    /// Get if the province is a supply center for whoever controls it.
    pub fn is_supply_center(&self) -> bool {
        self.supply_center != SupplyCenter::None
    }
}

impl ShortName for Province {
    fn short_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.short_name)
    }
}

impl PartialEq<ProvinceKey> for Province {
    fn eq(&self, other: &ProvinceKey) -> bool {
        self.short_name == other.short_name()
    }
}

/// An identifier that can be resolved to a province
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProvinceKey(String);

impl ProvinceKey {
    pub fn new(short_name: impl Into<String>) -> Self {
        ProvinceKey(short_name.into())
    }
}

impl<'a> From<&'a Province> for ProvinceKey {
    fn from(p: &Province) -> Self {
        ProvinceKey(p.short_name().into_owned())
    }
}

impl From<String> for ProvinceKey {
    fn from(s: String) -> Self {
        ProvinceKey(s)
    }
}

impl<'a> From<&'a str> for ProvinceKey {
    fn from(s: &str) -> Self {
        ProvinceKey(String::from(s))
    }
}

impl ShortName for ProvinceKey {
    fn short_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.0)
    }
}

impl<'a> From<&'a ProvinceKey> for &'a str {
    fn from(pk: &'a ProvinceKey) -> Self {
        &pk.0
    }
}

impl PartialEq<Province> for ProvinceKey {
    fn eq(&self, other: &Province) -> bool {
        self.0 == other.short_name
    }
}
