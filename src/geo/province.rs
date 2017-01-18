use std::convert::From;

use ShortName;

/// A controllable area of the environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Province {
    pub full_name : String,
    pub short_name : String,
}

impl ShortName for Province {
    fn short_name(&self) -> String {
        self.short_name.clone()
    }
}

impl PartialEq<ProvinceKey> for Province {
    fn eq(&self, other: &ProvinceKey) -> bool {
        self.short_name == other.short_name()
    }
}

/// An identifier that can be resolved to a province
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvinceKey(String);

impl ProvinceKey {
    pub fn new<IS : Into<String>>(short_name: IS) -> Self {
        ProvinceKey(short_name.into())
    }
}

impl<'a> From<&'a Province> for ProvinceKey {
    fn from(p: &Province) -> Self {
        ProvinceKey(p.short_name())
    }
}

impl From<String> for ProvinceKey {
    fn from(s: String) -> Self {
        ProvinceKey(s)
    }
}

impl ShortName for ProvinceKey {
    fn short_name(&self) -> String {
        self.0.clone()
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