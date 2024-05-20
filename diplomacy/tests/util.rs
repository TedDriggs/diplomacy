#![allow(dead_code)]
#![cfg(test)]

use diplomacy::{
    geo::{self, Coast, ProvinceKey, RegionKey},
    judge::{MappedBuildOrder, MappedMainOrder, MappedRetreatOrder, OrderState, Rulebook},
    Nation,
};
use std::collections::HashMap;

pub fn prov(s: &str) -> ProvinceKey {
    ProvinceKey::new(s)
}

pub fn reg(s: &str) -> RegionKey {
    reg_coast(s, None)
}

pub fn reg_coast(s: &str, c: impl Into<Option<Coast>>) -> RegionKey {
    RegionKey::new(prov(s), c)
}

#[macro_export]
macro_rules! submit_main_phase {
    ($($rule:tt $(: $outcome:expr)?),+) => {
        submit_main_phase!(@inner $($rule $(: $outcome)?,)*)
    };
    ($($rule:tt $(: $outcome:expr)?,)+) => {
        {
            use ::diplomacy::judge::{MappedMainOrder, Submission};
            let orders = vec![$($rule),*].into_iter().map(ord).collect::<Vec<_>>();
            let expectations: ::std::collections::HashMap<MappedMainOrder, _> = vec![$($((ord($rule), $outcome),)?)*]
                .into_iter()
                .collect();
            (Submission::with_inferred_state(orders), expectations)
        }
    };
}

#[macro_export]
macro_rules! resolve_main {
    ($context:expr, $expectation:expr) => {{
        let outcome = $context.adjudicate(
            ::diplomacy::geo::standard_map(),
            ::diplomacy::judge::Rulebook,
        );

        // We refer back to the submitted orders to ensure we visit orders in the same
        // order across test runs. This makes output diffing easier.
        for order in $context.submitted_orders() {
            println!("{:?}: {:?}", order, outcome.get(order).unwrap());
        }

        for order in $context.submitted_orders() {
            if let Some(expectation) = $expectation.get(order) {
                assert_eq!(
                    *expectation,
                    ::diplomacy::judge::OrderState::from(outcome.get(order).unwrap()),
                    "{}",
                    order
                );
            }
        }

        outcome
    }};
}

#[macro_export]
macro_rules! assert_state {
    ($results:ident, $order:tt, $expectation:expr) => {
        assert_eq!(
            $expectation,
            *$results
                .get(&ord($order))
                .expect("Order should be in results"),
            $order
        );
    };
}

/// Adjudicate a set of orders and - if specified - assert that order's success or failure.
#[macro_export]
macro_rules! judge {
    (@using $resolver:path => $($rule:tt $(: $outcome:expr)?),+) => {
        {
            let results = $resolver(vec![$($rule),*]);

            $(
                $(assert_state!(results, $rule, $outcome);)*
            )*

            results
        }
    };
    ($($rule:tt $(: $outcome:expr)?),+) => {
        judge!(@using get_results => $($rule $(: $outcome)*),*)
    };
    ($($rule:tt $(: $outcome:expr)?,)+) => {
        judge!(@using get_results => $($rule $(: $outcome)*),*)
    };
}

/// Adjudicate a retreat phase that occurs after the provided main phase
#[macro_export]
macro_rules! judge_retreat {
    ($main_phase:expr, $($rule:tt $(: $expected:expr)?),+) => {
        judge_retreat!($main_phase, $($rule $(: $expected)?,)+)
    };
    ($main_phase:expr, $($rule:tt $(: $expected:expr)?,)+) => {
        let results = $main_phase.to_retreat_start();
        let retreat_context = ::diplomacy::judge::retreat::Context::new(&results, vec![$($rule),*].into_iter().map(retreat_ord));
        let outcome = retreat_context.resolve();
        $(
            $(
                assert_eq!(
                    $expected,
                    *outcome.get(&retreat_ord($rule)).expect("Order should be in results"),
                    $rule
                );
            )*
        )*
    };
}

/// Adjudicate a build phase that occurs in the provided world
#[macro_export]
macro_rules! judge_build {
    ($world:expr) => {
        judge_build!($world, )
    };
    ($world:expr, $($rule:tt $(: $expected:expr)?),+) => {
        judge_build!($world, $($rule $(: $expected)?,)+)
    };
    ($world:expr, $($rule:tt $(: $expected:expr)?,)*) => {
        {
            let map = geo::standard_map();
            let last_time = initial_ownerships();
            let world = $world;
            let build_context = ::diplomacy::judge::build::Context::new(&map, &last_time, &world, vec![$($rule),*].into_iter().map(build_ord));
            let outcome = build_context.resolve();
            $(
                $(
                    assert_eq!(
                        $expected,
                        *outcome.get(&build_ord($rule)).expect("Order should be in results"),
                        $rule
                    );
                )*
            )*

            let final_units = outcome.final_units.into_iter().map(|(k, v)| (k.clone(), v)).collect::<std::collections::HashMap<_, _>>();

            (final_units, outcome.civil_disorder)
        }
    };
}

pub fn ord(s: &str) -> MappedMainOrder {
    s.parse()
        .unwrap_or_else(|e| panic!("'{}' should be a valid order: {}", s, e))
}

pub fn retreat_ord(s: &str) -> MappedRetreatOrder {
    s.parse()
        .unwrap_or_else(|e| panic!("'{}' should be a valid order: {}", s, e))
}

pub fn build_ord(s: &str) -> MappedBuildOrder {
    s.parse()
        .unwrap_or_else(|e| panic!("'{}' should be a valid order: {}", s, e))
}

pub fn get_results(orders: Vec<&str>) -> HashMap<MappedMainOrder, OrderState> {
    let parsed = orders.into_iter().map(ord).collect::<Vec<_>>();
    let ctx = diplomacy::judge::Submission::with_inferred_state(parsed);

    let out = ctx.adjudicate(geo::standard_map(), Rulebook);
    for o in ctx.submitted_orders() {
        println!("{:?}: {:?}", o, out.get(o).unwrap());
    }

    out.into()
}

pub fn initial_ownerships() -> HashMap<ProvinceKey, Nation> {
    vec![
        ("ENG", vec!["edi", "lvp", "lon"]),
        ("FRA", vec!["bre", "par", "mar"]),
        ("GER", vec!["kie", "ber", "mun"]),
        ("ITA", vec!["ven", "rom", "nap"]),
        ("AUS", vec!["vie", "bud", "tri"]),
        ("RUS", vec!["stp", "mos", "war", "sev"]),
        ("TUR", vec!["con", "ank", "smy"]),
    ]
    .into_iter()
    .flat_map(|(nat, provs)| {
        provs
            .into_iter()
            .map(move |prov| (ProvinceKey::from(prov), Nation::from(nat)))
    })
    .collect()
}
