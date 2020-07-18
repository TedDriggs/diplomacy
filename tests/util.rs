#![allow(dead_code)]
#![cfg(test)]

use diplomacy::geo;
use diplomacy::geo::{Coast, ProvinceKey, RegionKey};
use diplomacy::judge::{MappedMainOrder, MappedRetreatOrder, OrderState, ResolverContext};
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
macro_rules! context_and_expectation {
    (@inner $($rule:tt $(: $outcome:ident)?),+) => {
        {
            use ::diplomacy::judge::{MappedMainOrder, OrderState, ResolverContext};
            let orders = vec![$($rule),*].into_iter().map(ord).collect::<Vec<_>>();
            let expectations: ::std::collections::HashMap<MappedMainOrder, OrderState> = vec![$($((ord($rule), $outcome),)?)*]
                .into_iter()
                .collect();
            (ResolverContext::new(::diplomacy::geo::standard_map(), orders), expectations)
        }
    };
    ($($rule:tt $(: $outcome:ident)?),+) => {
        context_and_expectation!(@inner $($rule $(: $outcome)*),*)
    };
    ($($rule:tt $(: $outcome:ident)?,)+) => {
        context_and_expectation!(@inner $($rule $(: $outcome)*),*)
    };
}

#[macro_export]
macro_rules! resolve_main {
    ($context:expr, $expectation:expr) => {{
        let outcome = $context.resolve();

        for order in $context.orders() {
            println!("{:?}: {:?}", order, outcome.get(order).unwrap());
        }

        for order in $context.orders() {
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
    ($results:ident, $order:tt, $expectation:ident) => {
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
    (@using $resolver:path => $($rule:tt $(: $outcome:ident)?),+) => {
        {
            let results = $resolver(vec![$($rule),*]);

            // XXX in the case where no outcomes are asserted, clippy gets unhappy.
            // We use this temporary variable to make it happy since we can't suppress
            // the warning with an attribute.
            let _foo = &results;

            $(
                $(assert_state!(results, $rule, $outcome);)*
            )*

            results
        }
    };
    ($($rule:tt $(: $outcome:ident)?),+) => {
        judge!(@using get_results => $($rule $(: $outcome)*),*)
    };
    ($($rule:tt $(: $outcome:ident)?,)+) => {
        judge!(@using get_results => $($rule $(: $outcome)*),*)
    };
}

/// Adjudicate a retreat phase that occurs after the provided main phase
#[macro_export]
macro_rules! judge_retreat {
    ($main_phase:expr, $($rule:tt $(: $expected:expr)?),+) => {
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

pub fn ord(s: &str) -> MappedMainOrder {
    s.parse()
        .unwrap_or_else(|_| panic!(format!("'{}' should be a valid order", s)))
}

pub fn retreat_ord(s: &str) -> MappedRetreatOrder {
    s.parse()
        .unwrap_or_else(|_| panic!(format!("'{}' should be a valid order", s)))
}

pub fn get_results(orders: Vec<&str>) -> HashMap<MappedMainOrder, OrderState> {
    let parsed = orders.into_iter().map(ord).collect::<Vec<_>>();
    let ctx = ResolverContext::new(geo::standard_map(), parsed.clone());

    let out = ctx.resolve();
    for o in parsed {
        println!("{:?}: {:?}", o, out.get(&o).unwrap());
    }

    out.into()
}
