#![cfg(test)]
extern crate diplomacy;

use diplomacy::geo::{Coast, RegionKey, ProvinceKey};

pub fn prov(s: &str) -> ProvinceKey {
    ProvinceKey::new(s)
}

pub fn reg(s: &str) -> RegionKey {
    reg_coast(s, None)
}

pub fn reg_coast<IC : Into<Option<Coast>>>(s: &str, c: IC) -> RegionKey {
    RegionKey::new(prov(s), c)
}