//! Types for declaring and running test cases of a Diplomacy adjudicator.

use std::fmt;

use diplomacy::{
    geo::RegionKey,
    judge::{MappedBuildOrder, MappedMainOrder, MappedRetreatOrder, OrderState, Rulebook},
    Nation, UnitPosition,
};
use indexmap::IndexMap;

use crate::MapKey;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum Edition {
    #[serde(rename = "1971")]
    Edition1971,
    #[serde(rename = "1982")]
    Edition1982,
    #[serde(rename = "2023")]
    Edition2023,
    #[serde(rename = "dptg")]
    Dptg,
}

impl From<Edition> for Rulebook {
    fn from(edition: Edition) -> Self {
        match edition {
            Edition::Edition1971 => Rulebook::edition_1971(),
            Edition::Edition1982 => Rulebook::edition_1982(),
            Edition::Edition2023 => Rulebook::edition_2023(),
            Edition::Dptg => Rulebook::edition_dptg(),
        }
    }
}

impl fmt::Display for Edition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edition::Edition1971 => write!(f, "1971"),
            Edition::Edition1982 => write!(f, "1982"),
            Edition::Edition2023 => write!(f, "2023"),
            Edition::Dptg => write!(f, "dptg"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Cases<T> {
    pub cases: Vec<T>,
}

pub trait DidPass {
    /// Returns `true` if the test case passed, meaning all expected outcomes matched the actual outcomes.
    fn did_pass(&self) -> bool;
}

pub mod main {
    use diplomacy::{
        geo,
        judge::{OrderOutcome, Outcome, Rulebook, Submission},
    };

    use crate::{with_map_key, OrderOutcomeWithState};

    use super::*;

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[non_exhaustive]
    pub struct TestCase {
        /// The edition of the rules to use for this test case.
        /// If `None`, the test behaves identically for all editions of the standard rules.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub edition: Option<Edition>,
        /// Unit locations at the start of the test.
        ///
        /// If `None`, the test will infer the starting positions from the orders.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub starting_state: Option<Vec<MapKey<UnitPosition<'static, RegionKey>>>>,
        #[serde(with = "with_map_key")]
        pub orders: IndexMap<MappedMainOrder, Option<OrderState>>,
    }

    impl TestCase {
        fn starting_state(&self) -> Option<Vec<UnitPosition<'static, RegionKey>>> {
            self.starting_state
                .as_ref()
                .map(|positions| positions.iter().cloned().map(MapKey::into_inner).collect())
        }

        pub fn run(&self) -> TestResult {
            let submission = self.to_submission();
            self.to_test_result(
                &submission.adjudicate(self.edition.map(Rulebook::from).unwrap_or_default()),
            )
        }

        pub fn to_submission<'a>(&'a self) -> Submission<'a> {
            let orders = self.orders.keys().cloned().collect();
            match self.starting_state() {
                Some(state) => Submission::new(geo::standard_map(), &state, orders),
                None => Submission::with_inferred_state(geo::standard_map(), orders),
            }
        }

        pub fn to_test_result(&self, outcome: &Outcome<'_, Rulebook>) -> TestResult {
            TestResult {
                outcomes: outcome
                    .all_orders_with_outcomes()
                    .map(|(order, outcome)| {
                        (
                            order.clone(),
                            OrderOutcomeWithState::new(outcome.map_order(|o| MapKey(o.clone()))),
                        )
                    })
                    .collect(),
                mismatches: self
                    .orders
                    .iter()
                    .filter_map(|(order, expected)| {
                        let expected = (*expected)?;
                        let order_outcome = *outcome
                            .get(order)
                            .expect("All orders should be present in the outcome");
                        if expected == order_outcome.into() {
                            None
                        } else {
                            Some(MapKey(order.clone()))
                        }
                    })
                    .collect(),
            }
        }
    }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    pub struct TestResult {
        #[serde(with = "with_map_key")]
        outcomes:
            IndexMap<MappedMainOrder, OrderOutcomeWithState<OrderOutcome<MapKey<MappedMainOrder>>>>,
        mismatches: Vec<MapKey<MappedMainOrder>>,
    }

    impl DidPass for TestResult {
        fn did_pass(&self) -> bool {
            self.mismatches.is_empty()
        }
    }
}

pub mod retreat {
    use diplomacy::judge::{
        retreat::{Context, OrderOutcome},
        Rulebook,
    };

    use crate::{with_map_key, OrderOutcomeWithState};

    use super::*;

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[non_exhaustive]
    pub struct PrecedingMainPhase {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub starting_state: Option<Vec<MapKey<UnitPosition<'static, RegionKey>>>>,
        #[serde(with = "with_map_key")]
        pub orders: IndexMap<MappedMainOrder, Option<OrderState>>,
    }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[non_exhaustive]
    pub struct TestCase {
        /// The edition of the rules to use for this test case.
        /// If `None`, the test behaves identically for all editions of the standard rules.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub edition: Option<Edition>,
        pub preceding_main_phase: PrecedingMainPhase,
        #[serde(with = "with_map_key")]
        pub orders: IndexMap<MappedRetreatOrder, Option<OrderState>>,
    }

    impl TestCase {
        pub fn run(&self) -> TestResult {
            // First, adjudicate the preceding main phase to set up the retreat phase.
            let main_phase = main::TestCase::from(self);
            let main_phase_submission = main_phase.to_submission();
            let main_phase_outcome = main_phase_submission.adjudicate(Rulebook::default());
            let main_phase_result = main_phase.to_test_result(&main_phase_outcome);

            // If the main phase did not pass, then the retreat phase is not set up as expected.
            if !main_phase_result.did_pass() {
                return TestResult {
                    preceding_main_phase: main_phase_result,
                    outcomes: IndexMap::new(),
                    mismatches: Vec::new(),
                };
            }

            // Now start the actual test
            let retreat_start = main_phase_outcome.to_retreat_start();
            let ctx = Context::new(&retreat_start, self.orders.keys().cloned());
            let outcome = ctx.resolve();

            TestResult {
                preceding_main_phase: main_phase_result,
                outcomes: outcome
                    .order_outcomes()
                    .map(|(order, outcome)| {
                        (
                            order.clone(),
                            OrderOutcomeWithState::new(outcome.map_order(|o| MapKey(o.clone()))),
                        )
                    })
                    .collect(),
                mismatches: self
                    .orders
                    .iter()
                    .filter_map(|(order, expectation)| Some((order, (*expectation)?)))
                    .filter_map(|(order, expected)| {
                        let order_outcome =
                            *outcome.get(order).expect("All orders should be present");
                        if expected == order_outcome.into() {
                            None
                        } else {
                            Some(MapKey(order.clone()))
                        }
                    })
                    .collect(),
            }
        }
    }

    impl From<&TestCase> for main::TestCase {
        fn from(value: &TestCase) -> Self {
            main::TestCase {
                edition: value.edition,
                starting_state: value.preceding_main_phase.starting_state.clone(),
                orders: value.preceding_main_phase.orders.clone(),
            }
        }
    }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    pub struct TestResult {
        preceding_main_phase: main::TestResult,
        #[serde(with = "with_map_key")]
        outcomes: IndexMap<
            MappedRetreatOrder,
            OrderOutcomeWithState<OrderOutcome<MapKey<MappedRetreatOrder>>>,
        >,
        mismatches: Vec<MapKey<MappedRetreatOrder>>,
    }

    impl DidPass for TestResult {
        fn did_pass(&self) -> bool {
            self.preceding_main_phase.did_pass() && self.mismatches.is_empty()
        }
    }
}

pub mod build {
    use std::collections::{HashMap, HashSet};

    use crate::{with_map_key, OrderOutcomeWithState};

    use super::*;
    use diplomacy::{
        geo::{self, ProvinceKey},
        judge::build::{to_initial_ownerships, OrderOutcome, WorldState},
        UnitType,
    };

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[non_exhaustive]
    pub struct TestCase {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub occupiers: Option<IndexMap<ProvinceKey, Nation>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub starting_state: Option<Vec<MapKey<UnitPosition<'static, RegionKey>>>>,
        #[serde(with = "with_map_key")]
        pub orders: IndexMap<MappedBuildOrder, Option<OrderState>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub civil_disorder: Option<Vec<MapKey<UnitPosition<'static, RegionKey>>>>,
    }

    impl TestCase {
        pub fn run(&self) -> TestResult {
            let world = World::from(self);
            let last_time = to_initial_ownerships(geo::standard_map());
            let ctx = diplomacy::judge::build::Submission::new(
                geo::standard_map(),
                &last_time,
                &world,
                self.orders.keys().cloned(),
            );

            let outcome = ctx.adjudicate(Rulebook::default());

            let civil_disorder: Vec<_> = outcome
                .to_civil_disorder()
                .into_iter()
                .map(MapKey)
                .collect();

            TestResult {
                outcomes: outcome
                    .order_outcomes()
                    .map(|(order, outcome)| (order.clone(), OrderOutcomeWithState::new(*outcome)))
                    .collect(),
                mismatches: self
                    .orders
                    .iter()
                    .filter_map(|(order, expectation)| Some((order, (*expectation)?)))
                    .filter_map(|(order, expected)| {
                        let order_outcome =
                            *outcome.get(order).expect("All orders should be present");
                        if expected == order_outcome.into() {
                            None
                        } else {
                            Some(order.clone().into())
                        }
                    })
                    .chain(self.civil_disorder.iter().flatten().filter_map(|pos| {
                        // Any expected civil disorder positions that did not materialize are mismatches.
                        // Tests are not required to specify all civil disorder positions, so the
                        // reverse is not true.
                        (!civil_disorder.contains(pos))
                            .then_some(pos.clone())
                            .map(Mismatch::from)
                    }))
                    .collect(),
                civil_disorder,
            }
        }
    }

    struct World {
        nations: HashSet<Nation>,
        occupiers: HashMap<ProvinceKey, Nation>,
        units: HashMap<Nation, HashSet<(UnitType, RegionKey)>>,
    }

    impl From<&'_ TestCase> for World {
        fn from(value: &'_ TestCase) -> Self {
            let mut nations = HashSet::new();
            let mut occupiers = HashMap::new();
            let mut units = HashMap::<Nation, HashSet<(UnitType, RegionKey)>>::new();

            if let Some(occ) = &value.occupiers {
                nations.extend(occ.values().cloned());
                for (province, nation) in occ {
                    occupiers.insert(province.clone(), nation.clone());
                }
            }

            if let Some(unit_locations) = &value.starting_state {
                nations.extend(unit_locations.iter().map(|pos| pos.nation().clone()));
                for pos in unit_locations {
                    units
                        .entry(pos.nation().clone())
                        .or_default()
                        .insert((pos.unit.unit_type(), pos.region.clone()));
                    occupiers.insert(pos.region.province().clone(), pos.nation().clone());
                }
            }

            Self {
                nations,
                occupiers,
                units,
            }
        }
    }

    impl WorldState for World {
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

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    #[non_exhaustive]
    pub enum Mismatch {
        Order(MapKey<MappedBuildOrder>),
        Unit(MapKey<UnitPosition<'static, RegionKey>>),
    }

    impl From<MappedBuildOrder> for Mismatch {
        fn from(order: MappedBuildOrder) -> Self {
            Mismatch::Order(MapKey(order))
        }
    }

    impl From<MapKey<UnitPosition<'static, RegionKey>>> for Mismatch {
        fn from(pos: MapKey<UnitPosition<'static, RegionKey>>) -> Self {
            Mismatch::Unit(pos)
        }
    }

    impl From<UnitPosition<'static, RegionKey>> for Mismatch {
        fn from(pos: UnitPosition<'static, RegionKey>) -> Self {
            Mismatch::Unit(MapKey(pos))
        }
    }

    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    pub struct TestResult {
        civil_disorder: Vec<MapKey<UnitPosition<'static, RegionKey>>>,
        #[serde(with = "with_map_key")]
        outcomes: IndexMap<MappedBuildOrder, OrderOutcomeWithState<OrderOutcome>>,
        mismatches: Vec<Mismatch>,
    }

    impl DidPass for TestResult {
        fn did_pass(&self) -> bool {
            self.mismatches.is_empty()
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[non_exhaustive]
pub struct TestCaseTodo {
    #[serde(flatten)]
    pub info: TestCaseInfo,
    pub todo: String,
}

/// Either a runnable test case, or a todo item.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum RawTestCase {
    Todo(TestCaseTodo),
    Case(TestCase),
}

impl RawTestCase {
    pub fn info(&self) -> &TestCaseInfo {
        match self {
            RawTestCase::Todo(todo) => &todo.info,
            RawTestCase::Case(case) => &case.info,
        }
    }

    pub fn todo(&self) -> Option<&str> {
        match self {
            RawTestCase::Todo(todo) => Some(&todo.todo),
            RawTestCase::Case(_) => None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "phase")]
pub enum TestCaseBody {
    Main(main::TestCase),
    Retreat(retreat::TestCase),
    Build(build::TestCase),
}

impl TestCaseBody {
    pub fn edition(&self) -> Option<Edition> {
        match self {
            TestCaseBody::Main(case) => case.edition,
            TestCaseBody::Retreat(case) => case.edition,
            TestCaseBody::Build(_case) => None,
        }
    }

    pub fn run(&self) -> TestResultBody {
        match self {
            TestCaseBody::Main(case) => TestResultBody::Main(case.run()),
            TestCaseBody::Retreat(case) => TestResultBody::Retreat(case.run()),
            TestCaseBody::Build(case) => TestResultBody::Build(case.run()),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "phase")]
pub enum TestResultBody {
    Main(main::TestResult),
    Retreat(retreat::TestResult),
    Build(build::TestResult),
}

impl DidPass for TestResultBody {
    fn did_pass(&self) -> bool {
        match self {
            TestResultBody::Main(result) => result.did_pass(),
            TestResultBody::Retreat(result) => result.did_pass(),
            TestResultBody::Build(result) => result.did_pass(),
        }
    }
}

/// Wrapper around a test result that adds a `did_pass` field for serialization.
/// This avoids the need for clients to know how to check the pass/fail status themselves.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResult<T> {
    did_pass: bool,
    #[serde(flatten)]
    body: T,
}

impl<T: DidPass> TestResult<T> {
    pub fn new(body: T) -> Self {
        Self {
            did_pass: body.did_pass(),
            body,
        }
    }
}

impl<T> DidPass for TestResult<T> {
    fn did_pass(&self) -> bool {
        self.did_pass
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub struct TestCaseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl fmt::Display for TestCaseInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.name
                .as_ref()
                .or(self.url.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("Unnamed Test Case")
        )
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TestCase {
    #[serde(flatten)]
    pub info: TestCaseInfo,
    #[serde(flatten)]
    pub body: TestCaseBody,
}

impl TestCase {
    pub fn run(&self) -> TestResult<TestResultBody> {
        TestResult::new(self.body.run())
    }
}

#[cfg(test)]
mod tests {
    use crate::case::Cases;

    use super::{DidPass, RawTestCase, TestCase};

    #[test]
    fn build_example() {
        // DO NOT USE json! macro, as it does not preserve key order.
        let test_case: TestCase = serde_json::from_str(
            r#"{
            "name": "t6i01_too_many_build_orders",
            "url": "https://webdiplomacy.net/doc/DATC_v3_0.html#6.I.1",
            "phase": "Build",
            "starting_state": ["GER: F den", "GER: A ruh", "GER: A pru"],
            "orders": {
                "GER: A war build": "Fails",
                "GER: A ber build": "Succeeds",
                "GER: A mun build": "Fails"
            }
        }"#,
        )
        .unwrap();

        let result = test_case.run();
        eprintln!("{}", serde_json::to_string_pretty(&result).unwrap());
    }

    #[test]
    fn t6j01_too_many_remove_orders() {
        // DO NOT USE json! macro, as it does not preserve key order.
        let test_case: TestCase = serde_json::from_str(
            r#"{
            "name": "t6j01_too_many_remove_orders",
            "url": "https://webdiplomacy.net/doc/DATC_v3_0.html#6.J.1",
            "phase": "Build",
            "occupiers": {
                "bre": "ENG",
                "mar": "ITA"
            },
            "starting_state": ["FRA: A pic", "FRA: A par"],
            "orders": {
                "FRA: F lyo disband": "Fails",
                "FRA: A pic disband": "Succeeds",
                "FRA: A par disband": "Fails"
            }
        }"#,
        )
        .unwrap();

        assert!(test_case.run().did_pass(), "Test case should pass");
    }

    #[test]
    fn retreat_example() {
        // DO NOT USE json! macro, as it does not preserve key order.
        let test_case: TestCase = serde_json::from_str(
            r#"{
            "name": "t6h06_unit_may_not_retreat_to_a_contested_area",
            "url": "https://webdiplomacy.net/doc/DATC_v3_0.html#6.H.6",
            "phase": "Retreat",
            "preceding_main_phase": {
                "orders": {
                    "AUS: A bud Supports A tri -> vie": null,
                    "AUS: A tri -> vie": "Succeeds",
                    "GER: A mun -> boh": "Fails",
                    "GER: A sil -> boh": "Fails",
                    "ITA: A vie Hold": "Fails"
                }
            },
            "orders": {
                "ITA: A vie -> boh": "Fails"
            }
        }"#,
        )
        .unwrap();
        let result = test_case.run();
        eprintln!("{}", serde_json::to_string_pretty(&result).unwrap());
        assert!(result.did_pass(), "Test case should pass");
    }

    #[test]
    fn reads_all() {
        let test_cases: Vec<TestCase> =
            serde_json::from_str::<Cases<RawTestCase>>(include_str!("../datc.json"))
                .unwrap()
                .cases
                .into_iter()
                .filter_map(|v| match v {
                    RawTestCase::Case(test_case) => Some(test_case),
                    RawTestCase::Todo(e) => {
                        eprintln!("Skipping test case '{}'. {}", e.info, e.todo);
                        None
                    }
                })
                .collect();
        for test in test_cases {
            let result = test.run();
            if !result.did_pass() {
                eprintln!(
                    "Test case '{}' failed with mismatches: {}\n{}",
                    test.info,
                    serde_json::to_string_pretty(&test.body).unwrap(),
                    serde_json::to_string_pretty(&result).unwrap()
                );
            }
        }
    }
}
