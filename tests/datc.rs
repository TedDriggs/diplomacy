#![cfg(test)]
#![allow(unused_imports)]
extern crate diplomacy;

use std::collections::HashMap;

use diplomacy::geo;
use diplomacy::judge::{
    self, MappedMainOrder, OrderState, Outcome, ResolverContext, ResolverState, Rulebook,
};
use diplomacy::order::Command;
use diplomacy::Nation;

fn ord(s: &str) -> MappedMainOrder {
    s.parse()
        .expect(&format!("'{}' should be a valid order", s))
}

fn get_results(orders: Vec<&str>) -> HashMap<MappedMainOrder, OrderState> {
    let parsed = orders.into_iter().map(ord).collect::<Vec<_>>();

    judge::adjudicate(geo::standard_map(), parsed)
}

fn get_with_explanation(orders: Vec<&str>) -> HashMap<MappedMainOrder, OrderState> {
    let parsed = orders.into_iter().map(ord).collect::<Vec<_>>();
    let ctx = ResolverContext::new(geo::standard_map(), parsed.clone());

    {
        let state = ctx.resolve_to_state();
        for o in parsed {
            ctx.explain(&mut state.clone(), &o);
        }
    }

    ctx.resolve()
}

fn all_fail(orders: Vec<&str>) {
    let results = get_results(orders);
    for (o, outcome) in results {
        if outcome.into() {
            panic!("{} should have failed", o);
        }
    }
}

fn all_succeed(orders: Vec<&str>) {
    let results = get_results(orders);
    for (o, outcome) in results {
        if outcome == OrderState::Fails {
            panic!("{} should have succeeded", o);
        }
    }
}

fn report_results(map: &HashMap<MappedMainOrder, OrderState>) {
    for (o, r) in map {
        println!("{} {:?}", o, r)
    }
}

#[test]
fn t6a01_move_to_non_neighbor_fails() {
    all_fail(vec!["ENG: F nth -> pic"])
}

#[test]
fn t6a02_move_army_to_sea_fails() {
    all_fail(vec!["ENG: A lvp -> iri"]);
}

#[test]
fn t6a03_move_fleet_to_land_fails() {
    all_fail(vec!["GER: F kie -> mun"]);
}

#[test]
fn t6a04_move_to_own_sector_illegal() {
    all_fail(vec!["GER: F kie -> kie"]);
}

#[test]
fn t6a05_move_to_own_sector_with_convoy() {
    let results = get_with_explanation(vec![
        "ENG: F nth convoys yor -> yor",
        "ENG: A yor -> yor",
        "ENG: A lvp supports A yor -> yor",
        "GER: F lon -> yor",
        "GER: A wal supports F lon -> yor",
    ]);

    assert_eq!(
        &OrderState::Succeeds,
        results.get(&ord("GER: F lon -> yor")).unwrap()
    );
    assert_eq!(
        &OrderState::Fails,
        results.get(&ord("ENG: A yor -> yor")).unwrap()
    );
}

#[test]
fn t6a07_only_armies_can_be_convoyed() {
    let results = get_results(vec!["ENG: F lon -> bel", "ENG: F nth convoys lon -> bel"]);

    for (order, result) in results {
        if order.command.move_dest().is_some() {
            assert_eq!(OrderState::Fails, result);
        } else {
            assert_eq!(OrderState::Succeeds, result);
        }
    }
}

#[test]
fn t6a08_support_to_hold_self_fails() {
    let results = get_results(vec![
        "ITA: A ven -> tri",
        "ITA: A tyr supports A ven -> tri",
        "AUS: F tri supports F tri",
    ]);

    for (o, r) in results {
        if r.into() && o.nation != Nation::from("ITA") {
            panic!("Why did AUS succeed?");
        }
    }
}

#[test]
fn t6a09_fleets_cannot_go_overland() {
    all_fail(vec!["ITA: F rom -> ven"]);
}

#[test]
fn t6a10_support_on_unreachable_destination_not_possible() {
    let results = get_results(vec![
        "AUS: A ven holds",
        "ITA: F rom supports A apu -> ven",
        "ITA: A apu -> ven",
    ]);

    for (order, result) in results {
        if order.nation == Nation(String::from("AUS")) {
            assert_eq!(OrderState::Succeeds, result);
        } else if order.command.move_dest().is_some() {
            assert_eq!(OrderState::Fails, result);
        }
    }
}

#[test]
fn t6a11_simple_bounce() {
    all_fail(vec!["AUS: A vie -> tyr", "ITA: A ven -> tyr"]);
}

#[test]
fn t6a12_bounce_of_three_units() {
    all_fail(vec![
        "AUS: A vie -> tyr",
        "ITA: A ven -> tyr",
        "GER: A mun -> tyr",
    ]);
}

#[test]
fn t6b01_moving_without_required_coast_fails() {
    all_fail(vec!["FRA: F por -> spa"]);
}

#[test]
fn t6b02_moving_with_inferrable_coast_fails() {
    all_fail(vec!["FRA: F gas -> spa"]);
}

#[test]
fn t6b03_moving_with_wrong_coast_when_right_inferrable_fails() {
    all_fail(vec!["FRA: F gas -> spa(sc)"]);
}

#[test]
fn t6b04_support_to_unreachable_coast_allowed() {
    let results = get_results(vec![
        "FRA: F gas -> spa(nc)",
        "FRA: F mar supports F gas -> spa(nc)",
        "ITA: F wes -> spa(sc)",
    ]);

    for (order, result) in results {
        assert_eq!(result, (order.nation == Nation(String::from("FRA"))).into());
    }
}

#[test]
fn t6b05_support_from_unreachable_coast_not_allowed() {
    let results = get_results(vec![
        "FRA: F mar -> lyo",
        "FRA: F spa(nc) supports F mar -> lyo",
        "ITA: F lyo holds",
    ]);

    for (order, result) in results {
        if order.command.move_dest().is_some() {
            assert_eq!(result, OrderState::Fails);
        } else {
            assert_eq!(result, OrderState::Succeeds);
        }
    }
}

#[test]
fn t6b06_support_cut_from_other_coast_succeeds() {
    let orders = vec![
        "ENG: F iri supports F nao -> mao",
        "ENG: F nao -> mao",
        "FRA: F spa(nc) supports F mao",
        "FRA: F mao holds",
        "ITA: F lyo -> spa(sc)",
    ];
    let results = get_results(orders.clone());

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("FRA: F spa(nc) supports F mao"))
    );
}

#[test]
fn t6b13_coastal_crawl_not_allowed() {
    all_fail(vec!["TUR: F bul(sc) -> con", "TUR: F con -> bul(ec)"]);
}

#[test]
fn t6c01_three_army_circular_movement_succeeds() {
    all_succeed(vec![
        "TUR: F ank -> con",
        "TUR: A con -> smy",
        "TUR: A smy -> ank",
    ]);
}

#[test]
fn t6c02_three_army_circular_movement_with_support_succeeds() {
    all_succeed(vec![
        "TUR: F ank -> con",
        "TUR: A con -> smy",
        "TUR: A smy -> ank",
        "TUR: A bul supports F ank -> con",
    ]);
}

#[test]
fn t6c03_three_army_circular_movement_disrupted_bounces() {
    all_fail(vec![
        "TUR: F ank -> con",
        "TUR: A bul -> con",
        "TUR: A smy -> ank",
        "TUR: A con -> smy",
    ]);
}

#[test]
fn t6d01_supported_hold_can_prevent_dislodgement() {
    let results = get_results(vec![
        "AUS: F adr supports A tri -> ven",
        "AUS: A tri -> ven",
        "ITA: A ven hold",
        "ITA: A tyr supports A ven",
    ]);

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("AUS: A tri -> ven"))
    );
}

#[test]
fn t6d02_move_cuts_support_on_hold() {
    let results = get_results(vec![
        "AUS: F adr supports A tri -> ven",
        "AUS: A tri -> ven",
        "AUS: A vie -> tyr",
        "ITA: A ven hold",
        "ITA: A tyr supports A ven",
    ]);

    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("AUS: A tri -> ven"))
    );
}

#[test]
fn t6d03_move_cuts_support_on_move() {
    let results = get_results(vec![
        "AUS: F adr supports A tri -> ven",
        "AUS: A tri -> ven",
        "ITA: A ven hold",
        "ITA: F ion -> adr",
    ]);

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("AUS: A tri -> ven"))
    );
}

#[test]
fn t6d04_support_to_hold_on_unit_supporting_hold_allowed() {
    let results = get_results(vec![
        "GER: A ber supports F kie",
        "GER: F kie supports A ber",
        "RUS: F bal supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("RUS: A pru -> ber"))
    );
}

#[test]
fn t6d05_support_to_hold_on_unit_supporting_move_allowed() {
    let results = get_results(vec![
        "GER: A ber supports A mun -> sil",
        "GER: F kie supports A ber",
        "GER: A mun -> sil",
        "RUS: F bal supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("RUS: A pru -> ber"))
    );
}

#[test]
fn t6d09_support_to_move_on_holding_unit_fails() {
    let results = get_results(vec![
        "ITA: A ven -> tri",
        "ITA: A tyr supports A ven -> tri",
        "AUS: A alb supports A tri -> ser",
        "AUS: A tri holds",
    ]);

    report_results(&results);
    assert_eq!(
        &OrderState::Succeeds,
        results.get(&ord("ITA: A ven -> tri")).unwrap()
    );
}

#[test]
fn t6d33_unwanted_support_allowed() {
    let results = get_results(vec![
        "AUS: A ser -> bud",
        "AUS: A vie -> bud",
        "RUS: A gal supports A ser -> bud",
        "TUR: A bul -> ser",
    ]);

    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("AUS: A ser -> bud"))
    );
    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("TUR: A bul -> ser"))
    );
    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("AUS: A vie -> bud"))
    );
}

#[test]
fn t6d34_support_targeting_own_area_not_allowed() {
    let results = get_results(vec![
        "GER: A ber -> pru",
        "GER: A sil supports A ber -> pru",
        "GER: F bal supports A ber -> pru",
        "ITA: A pru supports A lvn -> pru",
        "RUS: A war supports A lvn -> pru",
        "RUS: A lvn -> pru",
    ]);

    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("GER: A ber -> pru"))
    );
    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("RUS: A lvn -> pru"))
    );
}

#[test]
fn t6e01_dislodged_unit_has_no_effect_on_attacker_area() {
    let results = get_results(vec![
        "GER: A ber -> pru",
        "GER: F kie -> ber",
        "GER: A sil supports A ber -> pru",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("GER: A ber -> pru"))
    );
    assert_eq!(
        Some(&OrderState::Succeeds),
        results.get(&ord("GER: F kie -> ber"))
    );
    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("RUS: A pru -> ber"))
    );
}

#[test]
fn t6e03_no_help_dislodging_own_unit() {
    let results = get_results(vec![
        "GER: A ber -> kie",
        "GER: A mun supports F kie -> ber",
        "ENG: F kie -> ber",
    ]);

    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("GER: A ber -> kie"))
    );
    assert_eq!(
        Some(&OrderState::Fails),
        results.get(&ord("ENG: F kie -> ber"))
    );
}
