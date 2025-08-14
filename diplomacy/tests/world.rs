#![allow(dead_code)]
#![cfg(test)]

use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
};

use diplomacy::{
    geo::{ProvinceKey, RegionKey},
    judge::build::WorldState,
    Nation, UnitPosition, UnitType,
};

pub(crate) struct TestWorld {
    nations: HashSet<Nation>,
    occupiers: HashMap<ProvinceKey, Nation>,
    units: HashMap<Nation, HashSet<(UnitType, RegionKey)>>,
}

impl TestWorld {
    pub fn empty() -> Self {
        Self {
            nations: Default::default(),
            occupiers: Default::default(),
            units: Default::default(),
        }
    }

    /// Mark a province as occupied by a country without having
    /// to set a unit that is present in it.
    pub fn with_occupier(
        mut self,
        province: impl Into<ProvinceKey>,
        nation: impl Into<Nation>,
    ) -> Self {
        let nation = nation.into();
        self.nations.insert(nation.clone());
        self.occupiers.insert(province.into(), nation);
        self
    }

    /// Add a unit to the world. This will also set the nation's unit as the province
    /// occupier, the same as calling [`Self::with_occupier`].
    pub fn with_unit(mut self, position_text: &str) -> Self {
        let position: UnitPosition<'_, RegionKey> = position_text.parse().unwrap();
        self.nations.insert(position.nation().clone());
        self.units
            .entry(position.nation().clone())
            .or_default()
            .insert((position.unit.unit_type(), position.region.clone()));
        self.occupiers.insert(
            position.region.province().clone(),
            position.nation().clone(),
        );

        self
    }
}

impl WorldState for TestWorld {
    fn nations(&self) -> HashSet<&Nation> {
        self.nations.iter().collect()
    }

    fn occupier(&self, province: &ProvinceKey) -> Option<&Nation> {
        self.occupiers.get(province)
    }

    fn unit_count(&self, nation: &Nation) -> u8 {
        self.units
            .get(nation)
            .map(|u| u.len())
            .unwrap_or_default()
            .try_into()
            .unwrap()
    }

    fn units(&self, nation: &Nation) -> HashSet<(UnitType, RegionKey)> {
        self.units.get(nation).cloned().unwrap_or_default()
    }
}

impl WorldState for &TestWorld {
    fn nations(&self) -> HashSet<&Nation> {
        self.nations.iter().collect()
    }

    fn occupier(&self, province: &ProvinceKey) -> Option<&Nation> {
        self.occupiers.get(province)
    }

    fn unit_count(&self, nation: &Nation) -> u8 {
        self.units
            .get(nation)
            .map(|u| u.len())
            .unwrap_or_default()
            .try_into()
            .unwrap()
    }

    fn units(&self, nation: &Nation) -> HashSet<(UnitType, RegionKey)> {
        self.units.get(nation).cloned().unwrap_or_default()
    }
}
