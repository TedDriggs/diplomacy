use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::From;

use crate::Nation;
use crate::ShortName;

/// A controllable area of the environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Province {
    pub full_name: String,
    pub short_name: String,
    pub supply_center_for: Option<Nation>,
}

impl ShortName for Province {
    fn short_name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(&self.short_name)
    }
}

impl PartialEq<ProvinceKey> for Province {
    fn eq(&self, other: &ProvinceKey) -> bool {
        self.short_name == other.short_name()
    }
}

/// An identifier that can be resolved to a province
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProvinceKey(String);

impl ProvinceKey {
    pub fn new<IS: Into<String>>(short_name: IS) -> Self {
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
    fn short_name<'a>(&'a self) -> Cow<'a, str> {
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
