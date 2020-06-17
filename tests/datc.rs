#![cfg(test)]
#![allow(unused_variables)]

use std::collections::HashMap;

use diplomacy::judge::OrderState::{Fails, Succeeds};
use diplomacy::judge::{self, MappedMainOrder, OrderState, ResolverContext};
use diplomacy::{geo, order::Command, Nation};

macro_rules! get_state {
    ($results:ident, $order:tt) => {
        *$results
            .get(&ord($order))
            .expect("Order should be in results")
    };
}

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
        if outcome == Fails {
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
fn t6a02_move_army_to_sea() {
    all_fail(vec!["ENG: A lvp -> iri"]);
}

#[test]
fn t6a03_move_fleet_to_land() {
    all_fail(vec!["GER: F kie -> mun"]);
}

#[test]
fn t6a04_move_to_own_sector() {
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

    assert_eq!(Succeeds, get_state!(results, "GER: F lon -> yor"));
    assert_eq!(Fails, get_state!(results, "ENG: A yor -> yor"));
}

#[test]
#[ignore]
fn t6a06_ordering_a_unit_of_another_country() {
    let results = get_results(vec!["GER: F lon -> nor"]);
}

#[test]
fn t6a07_only_armies_can_be_convoyed() {
    let results = get_results(vec!["ENG: F lon -> bel", "ENG: F nth convoys lon -> bel"]);

    for (order, result) in results {
        if order.command.move_dest().is_some() {
            assert_eq!(Fails, result);
        } else {
            assert_eq!(Succeeds, result);
        }
    }
}

#[test]
fn t6a08_support_to_hold_yourself_is_not_possible() {
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
fn t6a09_fleets_must_follow_coast_if_not_on_sea() {
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
            assert_eq!(Succeeds, result);
        } else if order.command.move_dest().is_some() {
            assert_eq!(Fails, result);
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
fn t6b02_moving_with_unspecified_coast_when_coast_is_not_necessary() {
    all_fail(vec!["FRA: F gas -> spa"]);
}

#[test]
fn t6b03_moving_with_wrong_coast_when_coast_is_not_necessary() {
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
            assert_eq!(result, Fails);
        } else {
            assert_eq!(result, Succeeds);
        }
    }
}

#[test]
fn t6b06_support_can_be_cut_with_other_coast() {
    let orders = vec![
        "ENG: F iri supports F nao -> mao",
        "ENG: F nao -> mao",
        "FRA: F spa(nc) supports F mao",
        "FRA: F mao holds",
        "ITA: F lyo -> spa(sc)",
    ];
    let results = get_results(orders.clone());

    assert_eq!(Fails, get_state!(results, "FRA: F spa(nc) supports F mao"));
}

#[test]
#[ignore]
fn t6b07_supporting_with_unspecified_coast() {
    let results = get_results(vec![
        "FRA: F por Supports F mao -> spa",
        "FRA: F mao -> spa(nc)",
        "ITA: F lyo Supports F wes -> spa(sc)",
        "ITA: F wes -> spa(sc)",
    ]);
}

#[test]
#[ignore]
fn t6b08_supporting_with_unspecified_coast_when_only_one_coast_is_possible() {
    let results = get_results(vec![
        "FRA: F por Supports F gas -> spa",
        "FRA: F gas -> spa(nc)",
        "ITA: F lyo Supports F wes -> spa(sc)",
        "ITA: F wes -> spa(sc)",
    ]);
}

#[test]
#[ignore]
fn t6b09_supporting_with_wrong_coast() {
    let results = get_results(vec![
        "FRA: F por Supports F mao -> spa(nc)",
        "FRA: F mao -> spa(sc)",
        "ITA: F lyo Supports F wes -> spa(sc)",
        "ITA: F wes -> spa(sc)",
    ]);
}

#[test]
#[ignore]
fn t6b10_unit_ordered_with_wrong_coast() {
    let results = get_results(vec!["FRA: F spa(nc) -> lyo"]);
}

#[test]
#[ignore]
fn t6b11_coast_can_not_be_ordered_to_change() {
    let results = get_results(vec!["FRA: F spa(sc) -> lyo"]);
}

#[test]
#[ignore]
fn t6b12_army_movement_with_coastal_specification() {
    let results = get_results(vec!["FRA: A gas -> spa(nc)"]);
}

#[test]
fn t6b13_coastal_crawl_not_allowed() {
    all_fail(vec!["TUR: F bul(sc) -> con", "TUR: F con -> bul(ec)"]);
}

#[test]
#[ignore]
fn t6b14_building_with_unspecified_coast() {
    let results = get_results(vec!["RUS: Build F St Petersburg"]);
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
fn t6c02_three_army_circular_movement_with_support() {
    all_succeed(vec![
        "TUR: F ank -> con",
        "TUR: A con -> smy",
        "TUR: A smy -> ank",
        "TUR: A bul supports F ank -> con",
    ]);
}

#[test]
fn t6c03_a_disrupted_three_army_circular_movement() {
    all_fail(vec![
        "TUR: F ank -> con",
        "TUR: A bul -> con",
        "TUR: A smy -> ank",
        "TUR: A con -> smy",
    ]);
}

#[test]
fn t6c04_a_circular_movement_with_attacked_convoy() {
    let results = get_results(vec![
        "AUS: A tri -> ser",
        "AUS: A ser -> bul",
        "TUR: A bul -> tri",
        "TUR: F aeg convoys bul -> tri",
        "TUR: F ion convoys bul -> tri",
        "TUR: F adr convoys bul -> tri",
        "ITA: F nap -> ion",
    ]);

    assert_eq!(Succeeds, get_state!(results, "AUS: A tri -> ser"));
    assert_eq!(Succeeds, get_state!(results, "AUS: A ser -> bul"));
    assert_eq!(Succeeds, get_state!(results, "TUR: A bul -> tri"));
    assert_eq!(Fails, get_state!(results, "ITA: F nap -> ion"));
}

#[test]
fn t6c05_a_disrupted_circular_movement_due_to_dislodged_convoy() {
    let results = get_results(vec![
        "AUS: A tri -> ser",
        "AUS: A ser -> bul",
        "TUR: A bul -> tri",
        "TUR: F aeg convoys bul -> tri",
        "TUR: F ion convoys bul -> tri",
        "TUR: F adr convoys bul -> tri",
        "ITA: F nap -> ion",
        "ITA: F tun supports F nap -> ion",
    ]);

    assert_eq!(Fails, get_state!(results, "AUS: A tri -> ser"));
    assert_eq!(Fails, get_state!(results, "AUS: A ser -> bul"));
    assert_eq!(Fails, get_state!(results, "TUR: A bul -> tri"));
    assert_eq!(Succeeds, get_state!(results, "ITA: F nap -> ion"));
}

#[test]
#[ignore]
fn t6c06_two_armies_with_two_convoys() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A lon -> bel",
        "FRA: F eng convoys bel -> lon",
        "FRA: A bel -> lon",
    ]);
}

#[test]
#[ignore]
fn t6c07_disrupted_unit_swap() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A lon -> bel",
        "FRA: F eng convoys bel -> lon",
        "FRA: A bel -> lon",
        "FRA: A bur -> bel",
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

    assert_eq!(Fails, get_state!(results, "AUS: A tri -> ven"));
}

#[test]
fn t6d02_a_move_cuts_support_on_hold() {
    let results = get_results(vec![
        "AUS: F adr supports A tri -> ven",
        "AUS: A tri -> ven",
        "AUS: A vie -> tyr",
        "ITA: A ven hold",
        "ITA: A tyr supports A ven",
    ]);

    assert_eq!(Succeeds, get_state!(results, "AUS: A tri -> ven"));
}

#[test]
fn t6d03_a_move_cuts_support_on_move() {
    let results = get_results(vec![
        "AUS: F adr supports A tri -> ven",
        "AUS: A tri -> ven",
        "ITA: A ven hold",
        "ITA: F ion -> adr",
    ]);

    assert_eq!(Fails, get_state!(results, "AUS: A tri -> ven"));
}

#[test]
fn t6d04_support_to_hold_on_unit_supporting_a_hold_allowed() {
    let results = get_results(vec![
        "GER: A ber supports F kie",
        "GER: F kie supports A ber",
        "RUS: F bal supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(Fails, get_state!(results, "RUS: A pru -> ber"));
}

#[test]
fn t6d05_support_to_hold_on_unit_supporting_a_move_allowed() {
    let results = get_results(vec![
        "GER: A ber supports A mun -> sil",
        "GER: F kie supports A ber",
        "GER: A mun -> sil",
        "RUS: F bal supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(Fails, get_state!(results, "RUS: A pru -> ber"));
}

#[test]
#[ignore]
fn t6d06_support_to_hold_on_convoying_unit_allowed() {
    let results = get_results(vec![
        "GER: A ber -> swe",
        "GER: F bal convoys ber -> swe",
        "GER: F pru Supports F bal",
        "RUS: F liv -> bal",
        "RUS: F bot Supports F liv -> bal",
    ]);
}

#[test]
#[ignore]
fn t6d07_support_to_hold_on_moving_unit_not_allowed() {
    let results = get_results(vec![
        "GER: F bal -> swe",
        "GER: F pru Supports F bal",
        "RUS: F liv -> bal",
        "RUS: F bot Supports F liv -> bal",
        "RUS: A Finland -> swe",
    ]);
}

#[test]
fn t6d08_failed_convoy_can_not_receive_hold_support() {
    let results = get_results(vec![
        "AUS: F ion hold",
        "AUS: A ser Supports A alb -> gre",
        "AUS: A alb -> gre",
        "TUR: A gre -> nap",
        "TUR: A bul Supports A gre",
    ]);

    assert_eq!(Succeeds, get_state!(results, "AUS: A alb -> gre"));
    assert_eq!(Fails, get_state!(results, "TUR: A gre -> nap"));
}

#[test]
fn t6d09_support_to_move_on_holding_unit_not_allowed() {
    let results = get_results(vec![
        "ITA: A ven -> tri",
        "ITA: A tyr supports A ven -> tri",
        "AUS: A alb supports A tri -> ser",
        "AUS: A tri holds",
    ]);

    report_results(&results);
    assert_eq!(Succeeds, get_state!(results, "ITA: A ven -> tri"));
}

#[test]
fn t6d10_self_dislodgment_prohibited() {
    let results = get_results(vec![
        "GER: A ber Hold",
        "GER: F kie -> ber",
        "GER: A mun Supports F kie -> ber",
    ]);
}

#[test]
fn t6d11_no_self_dislodgment_of_returning_unit() {
    let results = get_results(vec![
        "GER: A ber -> pru",
        "GER: F kie -> ber",
        "GER: A mun Supports F kie -> ber",
        "RUS: A war -> pru",
    ]);
}

#[test]
fn t6d12_supporting_a_foreign_unit_to_dislodge_own_unit_prohibited() {
    let results = get_results(vec![
        "AUS: F tri Hold",
        "AUS: A vie Supports A ven -> tri",
        "ITA: A ven -> tri",
    ]);
}

#[test]
fn t6d13_supporting_a_foreign_unit_to_dislodge_a_returning_own_unit_prohibited() {
    let results = get_results(vec![
        "AUS: F tri -> adr",
        "AUS: A vie Supports A ven -> tri",
        "ITA: A ven -> tri",
        "ITA: F apu -> adr",
    ]);
}

#[test]
fn t6d14_supporting_a_foreign_unit_is_not_enough_to_prevent_dislodgement() {
    let results = get_results(vec![
        "AUS: F tri Hold",
        "AUS: A vie Supports A ven -> tri",
        "ITA: A ven -> tri",
        "ITA: A tyr Supports A ven -> tri",
        "ITA: F adr Supports A ven -> tri",
    ]);
}

#[test]
fn t6d15_defender_can_not_cut_support_for_attack_on_itself() {
    let results = get_results(vec![
        "RUS: F con Supports F bla -> ank",
        "RUS: F bla -> ank",
        "TUR: F ank -> con",
    ]);
}

#[test]
fn t6d17_dislodgement_cuts_supports() {
    let results = get_results(vec![
        "RUS: F con Supports F bla -> ank",
        "RUS: F bla -> ank",
        "TUR: F ank -> con",
        "TUR: A smy Supports F ank -> con",
        "TUR: A arm -> ank",
    ]);
}

#[test]
fn t6d18_a_surviving_unit_will_sustain_support() {
    let results = get_results(vec![
        "RUS: F con Supports F bla -> ank",
        "RUS: F bla -> ank",
        "RUS: A bul Supports F con",
        "TUR: F ank -> con",
        "TUR: A smy Supports F ank -> con",
        "TUR: A arm -> ank",
    ]);
}

#[test]
fn t6d19_even_when_surviving_is_in_alternative_way() {
    let results = get_results(vec![
        "RUS: F con Supports F bla -> ank",
        "RUS: F bla -> ank",
        "RUS: A smy Supports F ank -> con",
        "TUR: F ank -> con",
    ]);
}

#[test]
fn t6d20_unit_can_not_cut_support_of_its_own_country() {
    let results = get_results(vec![
        "ENG: F lon Supports F nor -> eng",
        "ENG: F nor -> eng",
        "ENG: A yor -> lon",
        "FRA: F eng Hold",
    ]);
}

#[test]
fn t6d21_dislodging_does_not_cancel_a_support_cut() {
    let results = get_results(vec![
        "AUS: F tri Hold",
        "ITA: A ven -> tri",
        "ITA: A tyr supports A ven -> tri",
        "GER: A mun -> tyr",
        "RUS: A sil -> mun",
        "RUS: A ber Supports A sil -> mun",
    ]);

    assert_eq!(Succeeds, get_state!(results, "AUS: F tri Hold"));
}

#[test]
fn t6d22_impossible_fleet_move_can_not_be_supported() {
    let results = get_results(vec![
        "GER: F kie -> mun",
        "GER: A bur Supports F kie -> mun",
        "RUS: A mun -> kie",
        "RUS: A ber Supports A mun -> kie",
    ]);
}

#[test]
fn t6d24_impossible_army_move_can_not_be_supported() {
    let results = get_results(vec![
        "FRA: A mar -> lyo",
        "FRA: F spa(sc) Supports A mar -> lyo",
        "ITA: F lyo Hold",
        "TUR: F tyr Supports F wes -> lyo",
        "TUR: F wes -> lyo",
    ]);
}

#[test]
fn t6d25_failing_hold_support_can_be_supported() {
    let results = get_results(vec![
        "GER: A ber Supports A pru",
        "GER: F kie Supports A ber",
        "RUS: F bal Supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);
}

#[test]
fn t6d26_failing_move_support_can_be_supported() {
    let results = get_results(vec![
        "GER: A ber Supports A pru -> sil",
        "GER: F kie Supports A ber",
        "RUS: F bal Supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);
}

#[test]
fn t6d27_failing_convoy_can_be_supported() {
    let results = get_results(vec![
        "ENG: F swe -> bal",
        "ENG: F den Supports F swe -> bal",
        "GER: A ber Hold",
        "RUS: F bal convoys ber -> liv",
        "RUS: F pru Supports F bal",
    ]);
}

#[test]
#[ignore]
fn t6d28_impossible_move_and_support() {
    let results = get_results(vec![
        "AUS: A bud Supports F rum",
        "RUS: F rum -> hol",
        "TUR: F bla -> rum",
        "TUR: A bul Supports F bla -> rum",
    ]);
}

#[test]
#[ignore]
fn t6d29_move_to_impossible_coast_and_support() {
    let results = get_results(vec![
        "AUS: A bud Supports F rum",
        "RUS: F rum -> bul(sc)",
        "TUR: F bla -> rum",
        "TUR: A bul Supports F bla -> rum",
    ]);
}

#[test]
#[ignore]
fn t6d30_move_without_coast_and_support() {
    let results = get_results(vec![
        "ITA: F aeg Supports F con",
        "RUS: F con -> bul",
        "TUR: F bla -> con",
        "TUR: A bul Supports F bla -> con",
    ]);
}

/// In this case the proposed behavior is that the fleet order should be treated as illegal and
/// dropped entirely. It's not clear why that would be the case in computerized games, so this
/// test will remain ignored.
#[test]
#[ignore]
fn t6d31_a_tricky_impossible_support() {
    let results = get_results(vec![
        "AUS: A rum -> arm",
        "TUR: F bla Supports A rum -> arm",
    ]);
}

#[test]
fn t6d32_a_missing_fleet() {
    let results = get_results(vec![
        "ENG: F edi Supports A lvp -> yor",
        "ENG: A lvp -> yor",
        "FRA: F lon Supports A yor",
        "GER: A yor -> hol",
    ]);
}

#[test]
fn t6d33_unwanted_support_allowed() {
    let results = get_results(vec![
        "AUS: A ser -> bud",
        "AUS: A vie -> bud",
        "RUS: A gal supports A ser -> bud",
        "TUR: A bul -> ser",
    ]);

    assert_eq!(Succeeds, get_state!(results, "AUS: A ser -> bud"));
    assert_eq!(Succeeds, get_state!(results, "TUR: A bul -> ser"));
    assert_eq!(Fails, get_state!(results, "AUS: A vie -> bud"));
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

    assert_eq!(Succeeds, get_state!(results, "GER: A ber -> pru"));
    assert_eq!(Fails, get_state!(results, "RUS: A lvn -> pru"));
}

#[test]
fn t6e01_dislodged_unit_has_no_effect_on_attacker_area() {
    let results = get_results(vec![
        "GER: A ber -> pru",
        "GER: F kie -> ber",
        "GER: A sil supports A ber -> pru",
        "RUS: A pru -> ber",
    ]);

    assert_eq!(Succeeds, get_state!(results, "GER: A ber -> pru"));
    assert_eq!(Succeeds, get_state!(results, "GER: F kie -> ber"));
    assert_eq!(Fails, get_state!(results, "RUS: A pru -> ber"));
}

#[test]
fn t6e03_no_help_dislodging_own_unit() {
    let results = get_results(vec![
        "GER: A ber -> kie",
        "GER: A mun supports F kie -> ber",
        "ENG: F kie -> ber",
    ]);

    assert_eq!(Fails, get_state!(results, "GER: A ber -> kie"));
    assert_eq!(Fails, get_state!(results, "ENG: F kie -> ber"));
}

#[test]
fn t6e02_no_self_dislodgement_in_head_to_head_battle() {
    let results = get_results(vec![
        "GER: A ber -> kie",
        "GER: F kie -> ber",
        "GER: A mun Supports A ber -> kie",
    ]);
}

#[test]
fn t6e03_no_help_in_dislodging_own_unit() {
    let results = get_results(vec![
        "GER: A ber -> kie",
        "GER: A mun supports F kie -> ber",
        "ENG: F kie -> ber",
    ]);

    assert_eq!(Fails, get_state!(results, "GER: A ber -> kie"));
    assert_eq!(Fails, get_state!(results, "ENG: F kie -> ber"));
}

#[test]
fn t6e04_non_dislodged_loser_has_still_effect() {
    let results = get_results(vec![
        "GER: F hol -> nor",
        "GER: F hel Supports F hol -> nor",
        "GER: F ska Supports F hol -> nor",
        "FRA: F nor -> hol",
        "FRA: F bel Supports F nor -> hol",
        "ENG: F edi Supports F nwg -> nor",
        "ENG: F yor Supports F nwg -> nor",
        "ENG: F nwg -> nor",
        "AUS: A kie Supports A ruh -> hol",
        "AUS: A ruh -> hol",
    ]);
}

#[test]
fn t6e05_loser_dislodged_by_another_army_has_still_effect() {
    let results = get_results(vec![
        "GER: F hol -> nor",
        "GER: F hel Supports F hol -> nor",
        "GER: F ska Supports F hol -> nor",
        "FRA: F nor -> hol",
        "FRA: F bel Supports F nor -> hol",
        "ENG: F edi Supports F nwg -> nor",
        "ENG: F yor Supports F nwg -> nor",
        "ENG: F nwg -> nor",
        "ENG: F lon Supports F nwg -> nor",
        "AUS: A kie Supports A ruh -> hol",
        "AUS: A ruh -> hol",
    ]);
}

#[test]
fn t6e06_not_dislodge_because_of_own_support_has_still_effect() {
    let results = get_results(vec![
        "GER: F hol -> nor",
        "GER: F hel Supports F hol -> nor",
        "FRA: F nor -> hol",
        "FRA: F bel Supports F nor -> hol",
        "FRA: F eng Supports F hol -> nor",
        "AUS: A kie Supports A ruh -> hol",
        "AUS: A ruh -> hol",
    ]);
}

#[test]
fn t6e07_no_self_dislodgement_with_beleaguered_garrison() {
    let results = get_results(vec![
        "ENG: F nor Hold",
        "ENG: F yor Supports F nwy -> nor",
        "GER: F hol Supports F hel -> nor",
        "GER: F hel -> nor",
        "RUS: F ska Supports F nwy -> nor",
        "RUS: F nwy -> nor",
    ]);
}

#[test]
fn t6e08_no_self_dislodgement_with_beleaguered_garrison_and_head_to_head_battle() {
    let results = get_results(vec![
        "ENG: F nor -> nwy",
        "ENG: F yor Supports F nwy -> nor",
        "GER: F hol Supports F hel -> nor",
        "GER: F hel -> nor",
        "RUS: F ska Supports F nwy -> nor",
        "RUS: F nwy -> nor",
    ]);
}

#[test]
fn t6e09_almost_self_dislodgement_with_beleaguered_garrison() {
    let results = get_results(vec![
        "ENG: F nor -> nwg",
        "ENG: F yor Supports F nwy -> nor",
        "GER: F hol Supports F hel -> nor",
        "GER: F hel -> nor",
        "RUS: F ska Supports F nwy -> nor",
        "RUS: F nwy -> nor",
    ]);
}

#[test]
fn t6e10_almost_circular_movement_with_no_self_dislodgement_with_beleaguered_garrison() {
    let results = get_results(vec![
        "ENG: F nor -> den",
        "ENG: F yor Supports F nwy -> nor",
        "GER: F hol Supports F hel -> nor",
        "GER: F hel -> nor",
        "GER: F den -> hel",
        "RUS: F ska Supports F nwy -> nor",
        "RUS: F nwy -> nor",
    ]);
}

#[test]
fn t6e11_no_self_dislodgement_with_beleaguered_garrison_unit_swap_with_adjacent_convoying_and_two_coasts(
) {
    let results = get_results(vec![
        "FRA: A spa -> por via Convoy",
        "FRA: F mao convoys spa -> por",
        "FRA: F lyo Supports F por -> spa(nc)",
        "GER: A mar Supports A gas -> spa",
        "GER: A gas -> spa",
        "ITA: F por -> spa(nc)",
        "ITA: F wes Supports F por -> spa(nc)",
    ]);
}

#[test]
fn t6e12_support_on_attack_on_own_unit_can_be_used_for_other_means() {
    let results = get_results(vec![
        "AUS: A bud -> rum",
        "AUS: A ser Supports A vie -> bud",
        "ITA: A vie -> bud",
        "RUS: A gal -> bud",
        "RUS: A rum Supports A gal -> bud",
    ]);
}

#[test]
fn t6e13_three_way_beleaguered_garrison() {
    let results = get_results(vec![
        "ENG: F edi Supports F yor -> nor",
        "ENG: F yor -> nor",
        "FRA: F bel -> nor",
        "FRA: F eng Supports F bel -> nor",
        "GER: F nor Hold",
        "RUS: F nwg -> nor",
        "RUS: F nwy Supports F nwg -> nor",
    ]);
}

#[test]
fn t6e14_illegal_head_to_head_battle_can_still_defend() {
    let results = get_results(vec!["ENG: A lvp -> edi", "RUS: F edi -> lvp"]);
}

#[test]
fn t6e15_the_friendly_head_to_head_battle() {
    let results = get_results(vec![
        "ENG: F hol Supports A ruh -> kie",
        "ENG: A ruh -> kie",
        "FRA: A kie -> ber",
        "FRA: A mun Supports A kie -> ber",
        "FRA: A sil Supports A kie -> ber",
        "GER: A ber -> kie",
        "GER: F den Supports A ber -> kie",
        "GER: F hel Supports A ber -> kie",
        "RUS: F bal Supports A pru -> ber",
        "RUS: A pru -> ber",
    ]);
}

#[test]
fn t6f02_an_army_being_convoyed_can_bounce_as_normal() {
    let results = get_results(vec![
        "ENG: F eng convoys lon -> bre",
        "ENG: A lon -> bre",
        "FRA: A par -> bre",
    ]);
}

#[test]
fn t6f03_an_army_being_convoyed_can_receive_support() {
    let results = get_results(vec![
        "ENG: F eng convoys lon -> bre",
        "ENG: A lon -> bre",
        "ENG: F mao Supports A lon -> bre",
        "FRA: A par -> bre",
    ]);
}

#[test]
fn t6f04_an_attacked_convoy_is_not_disrupted() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> hol",
        "ENG: A lon -> hol",
        "GER: F ska -> nor",
    ]);
}

#[test]
fn t6f05_a_beleaguered_convoy_is_not_disrupted() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> hol",
        "ENG: A lon -> hol",
        "FRA: F eng -> nor",
        "FRA: F bel Supports F eng -> nor",
        "GER: F ska -> nor",
        "GER: F den Supports F ska -> nor",
    ]);
}

#[test]
fn t6f06_dislodged_convoy_does_not_cut_support() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> hol",
        "ENG: A lon -> hol",
        "GER: A hol Supports A bel",
        "GER: A bel Supports A hol",
        "GER: F hel Supports F ska -> nor",
        "GER: F ska -> nor",
        "FRA: A pic -> bel",
        "FRA: A bur Supports A pic -> bel",
    ]);
}

#[test]
fn t6f07_dislodged_convoy_does_not_cause_contested_area() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> hol",
        "ENG: A lon -> hol",
        "GER: F hel Supports F ska -> nor",
        "GER: F ska -> nor",
    ]);
}

#[test]
fn t6f08_dislodged_convoy_does_not_cause_a_bounce() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> hol",
        "ENG: A lon -> hol",
        "GER: F hel Supports F ska -> nor",
        "GER: F ska -> nor",
        "GER: A bel -> hol",
    ]);
}

#[test]
fn t6f09_dislodge_of_multi_route_convoy() {
    let results = get_results(vec![
        "ENG: F eng convoys lon -> bel",
        "ENG: F nor convoys lon -> bel",
        "ENG: A lon -> bel",
        "FRA: F bre Supports F mao -> eng",
        "FRA: F mao -> eng",
    ]);
}

#[test]
fn t6f10_dislodge_of_multi_route_convoy_with_foreign_fleet() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A lon -> bel",
        "GER: F eng convoys lon -> bel",
        "FRA: F bre Supports F mao -> eng",
        "FRA: F mao -> eng",
    ]);
}

#[test]
fn t6f11_dislodge_of_multi_route_convoy_with_only_foreign_fleets() {
    let results = get_results(vec![
        "ENG: A lon -> bel",
        "GER: F eng convoys lon -> bel",
        "RUS: F nor convoys lon -> bel",
        "FRA: F bre Supports F mao -> eng",
        "FRA: F mao -> eng",
    ]);
}

#[test]
fn t6f12_dislodged_convoying_fleet_not_on_route() {
    let results = get_results(vec![
        "ENG: F eng convoys lon -> bel",
        "ENG: A lon -> bel",
        "ENG: F iri convoys lon -> bel",
        "FRA: F nao Supports F mao -> iri",
        "FRA: F mao -> iri",
    ]);
}

#[test]
fn t6f13_the_unwanted_alternative() {
    let results = get_results(vec![
        "ENG: A lon -> bel",
        "ENG: F nor convoys lon -> bel",
        "FRA: F eng convoys lon -> bel",
        "GER: F hol Supports F den -> nor",
        "GER: F den -> nor",
    ]);
}

#[test]
#[should_panic]
fn t6f14_simple_convoy_paradox() {
    let results = get_results(vec![
        "ENG: F lon Supports F wal -> eng",
        "ENG: F wal -> eng",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
    ]);
}

#[test]
#[should_panic]
fn t6f15_simple_convoy_paradox_with_additional_convoy() {
    let results = get_results(vec![
        "ENG: F lon Supports F wal -> eng",
        "ENG: F wal -> eng",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "ITA: F iri convoys naf -> wal",
        "ITA: F mao convoys naf -> wal",
        "ITA: A naf -> wal",
    ]);
}

#[test]
#[should_panic]
fn t6f16_pandins_paradox() {
    let results = get_results(vec![
        "ENG: F lon Supports F wal -> eng",
        "ENG: F wal -> eng",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "GER: F nor Supports F bel -> eng",
        "GER: F bel -> eng",
    ]);
}

#[test]
#[should_panic]
fn t6f17_pandins_extended_paradox() {
    let results = get_results(vec![
        "ENG: F lon Supports F wal -> eng",
        "ENG: F wal -> eng",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "FRA: F yor Supports A bre -> lon",
        "GER: F nor Supports F bel -> eng",
        "GER: F bel -> eng",
    ]);
}

#[test]
fn t6f18_betrayal_paradox() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A lon -> bel",
        "ENG: F eng Supports A lon -> bel",
        "FRA: F bel Supports F nor",
        "GER: F hel Supports F ska -> nor",
        "GER: F ska -> nor",
    ]);

    assert_eq!(Fails, get_state!(results, "ENG: A lon -> bel"));
    assert_eq!(Fails, get_state!(results, "GER: F ska -> nor"));
}

#[test]
fn t6f19_multi_route_convoy_disruption_paradox() {
    let results = get_results(vec![
        "FRA: A tun -> nap",
        "FRA: F tyr convoys tun -> nap",
        "FRA: F ion convoys tun -> nap",
        "ITA: F nap Supports F Rome -> tyr",
        "ITA: F Rome -> tyr",
    ]);
}

#[test]
#[ignore]
fn t6f20_unwanted_multi_route_convoy_paradox() {
    let results = get_results(vec![
        "FRA: A tun -> nap",
        "FRA: F tyr convoys tun -> nap",
        "ITA: F nap Supports F ion",
        "ITA: F ion convoys tun -> nap",
        "TUR: F aeg Supports F eas -> ion",
        "TUR: F eas -> ion",
    ]);
}

#[test]
fn t6f21_dads_army_convoy() {
    let results = get_results(vec![
        "RUS: A edi Supports A nwy -> cly",
        "RUS: F nwg convoys nwy -> cly",
        "RUS: A nwy -> cly",
        "FRA: F iri Supports F mao -> nao",
        "FRA: F mao -> nao",
        "ENG: A lvp -> cly via Convoy",
        "ENG: F nao convoys lvp -> cly",
        "ENG: F cly Supports F nao",
    ]);
}

#[test]
fn t6f22_second_order_paradox_with_two_resolutions() {
    let results = get_results(vec![
        "ENG: F edi -> nor",
        "ENG: F lon Supports F edi -> nor",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "GER: F bel Supports F pic -> eng",
        "GER: F pic -> eng",
        "RUS: A nwy -> bel",
        "RUS: F nor convoys nwy -> bel",
    ]);
}

#[test]
fn t6f23_second_order_paradox_with_two_exclusive_convoys() {
    let results = get_results(vec![
        "ENG: F edi -> nor",
        "ENG: F yor Supports F edi -> nor",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "GER: F bel Supports F eng",
        "GER: F lon Supports F nor",
        "ITA: F mao -> eng",
        "ITA: F iri Supports F mao -> eng",
        "RUS: A nwy -> bel",
        "RUS: F nor convoys nwy -> bel",
    ]);
}

#[test]
fn t6f24_second_order_paradox_with_no_resolution() {
    let results = get_results(vec![
        "ENG: F edi -> nor",
        "ENG: F lon Supports F edi -> nor",
        "ENG: F iri -> eng",
        "ENG: F mao Supports F iri -> eng",
        "FRA: A bre -> lon",
        "FRA: F eng convoys bre -> lon",
        "FRA: F bel Supports F eng",
        "RUS: A nwy -> bel",
        "RUS: F nor convoys nwy -> bel",
    ]);
}

#[test]
fn t6g02_kidnapping_an_army() {
    let results = get_results(vec![
        "ENG: A nwy -> swe",
        "RUS: F swe -> nwy",
        "GER: F ska convoys nwy -> swe",
    ]);
}

#[test]
#[ignore]
fn t6g03_kidnapping_with_a_disrupted_convoy() {
    let results = get_results(vec![
        "FRA: F bre -> eng",
        "FRA: A pic -> bel",
        "FRA: A bur Supports A pic -> bel",
        "FRA: F mao Supports F bre -> eng",
        "ENG: F eng convoys pic -> bel",
    ]);
}

#[test]
#[ignore]
fn t6g04_kidnapping_with_a_disrupted_convoy_and_opposite_move() {
    let results = get_results(vec![
        "FRA: F bre -> eng",
        "FRA: A pic -> bel",
        "FRA: A bur Supports A pic -> bel",
        "FRA: F mao Supports F bre -> eng",
        "ENG: F eng convoys pic -> bel",
        "ENG: A bel -> pic",
    ]);
}

#[test]
#[ignore]
fn t6g05_swapping_with_intent() {
    let results = get_results(vec![
        "ITA: A Rome -> apu",
        "ITA: F tyr convoys apu -> Rome",
        "TUR: A apu -> Rome",
        "TUR: F ion convoys apu -> Rome",
    ]);
}

#[test]
#[ignore]
fn t6g06_swapping_with_unintended_intent() {
    let results = get_results(vec![
        "ENG: A lvp -> edi",
        "ENG: F eng convoys lvp -> edi",
        "GER: A edi -> lvp",
        "FRA: F iri Hold",
        "FRA: F nor Hold",
        "RUS: F nwg convoys lvp -> edi",
        "RUS: F nao convoys lvp -> edi",
    ]);
}

#[test]
#[ignore]
fn t6g07_swapping_with_illegal_intent() {
    let results = get_results(vec![
        "ENG: F ska convoys swe -> nwy",
        "ENG: F nwy -> swe",
        "RUS: A swe -> nwy",
        "RUS: F bot convoys swe -> nwy",
    ]);
}

#[test]
#[ignore]
fn t6g08_explicit_convoy_that_isnt_there() {
    let results = get_results(vec![
        "FRA: A bel -> hol via Convoy",
        "ENG: F nor -> hel",
        "ENG: A hol -> kie",
    ]);
}

#[test]
#[ignore]
fn t6g09_swapped_or_dislodged() {
    let results = get_results(vec![
        "ENG: A nwy -> swe",
        "ENG: F ska convoys nwy -> swe",
        "ENG: F Finland Supports A nwy -> swe",
        "RUS: A swe -> nwy",
    ]);
}

#[test]
fn t6g10_swapped_or_an_head_to_head_battle() {
    let results = get_results(vec![
        "ENG: A nwy -> swe via Convoy",
        "ENG: F den Supports A nwy -> swe",
        "ENG: F Finland Supports A nwy -> swe",
        "GER: F ska convoys nwy -> swe",
        "RUS: A swe -> nor",
        "RUS: F bar supports A swe -> nor",
        "FRA: F nwg -> nwy",
        "FRA: F nor Supports F nwg -> nwy",
    ]);
}

#[test]
fn t6g11_a_convoy_to_an_adjacent_place_with_a_paradox() {
    let results = get_results(vec![
        "ENG: F nwy Supports F nor -> ska",
        "ENG: F nor -> ska",
        "RUS: A swe -> nwy",
        "RUS: F ska convoys swe -> nwy",
        "RUS: F bar Supports A swe -> nwy",
    ]);
}

#[test]
fn t6g12_swapping_two_units_with_two_convoys() {
    let results = get_results(vec![
        "ENG: A lvp -> edi via Convoy",
        "ENG: F nao convoys lvp -> edi",
        "ENG: F nwg convoys lvp -> edi",
        "GER: A edi -> lvp via Convoy",
        "GER: F nor convoys edi -> lvp",
        "GER: F eng convoys edi -> lvp",
        "GER: F iri convoys edi -> lvp",
    ]);
}

#[test]
fn t6g13_support_cut_on_attack_on_itself_via_convoy() {
    let results = get_results(vec![
        "AUS: F adr convoys tri -> ven",
        "AUS: A tri -> ven via Convoy",
        "ITA: A ven Supports F alb -> tri",
        "ITA: F alb -> tri",
    ]);
}

#[test]
fn t6g14_bounce_by_convoy_to_adjacent_place() {
    let results = get_results(vec![
        "ENG: A nwy -> swe",
        "ENG: F den Supports A nwy -> swe",
        "ENG: F Finland Supports A nwy -> swe",
        "FRA: F nwg -> nwy",
        "FRA: F nor Supports F nwg -> nwy",
        "GER: F ska convoys swe -> nwy",
        "RUS: A swe -> nwy via Convoy",
        "RUS: F bar Supports A swe -> nwy",
    ]);
}

#[test]
fn t6g15_bounce_and_dislodge_with_double_convoy() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A hol Supports A lon -> bel",
        "ENG: A yor -> lon",
        "ENG: A lon -> bel via Convoy",
        "FRA: F eng convoys bel -> lon",
        "FRA: A bel -> lon via Convoy",
    ]);
}

#[test]
fn t6g16_the_two_unit_in_one_area_bug_moving_by_convoy() {
    let results = get_results(vec![
        "ENG: A nwy -> swe",
        "ENG: A den Supports A nwy -> swe",
        "ENG: F bal Supports A nwy -> swe",
        "ENG: F nor -> nwy",
        "RUS: A swe -> nwy via Convoy",
        "RUS: F ska convoys swe -> nwy",
        "RUS: F nwg Supports A swe -> nwy",
    ]);
}

#[test]
fn t6g17_the_two_unit_in_one_area_bug_moving_over_land() {
    let results = get_results(vec![
        "ENG: A nwy -> swe via Convoy",
        "ENG: A den Supports A nwy -> swe",
        "ENG: F bal Supports A nwy -> swe",
        "ENG: F ska convoys nwy -> swe",
        "ENG: F nor -> nwy",
        "RUS: A swe -> nwy",
        "RUS: F nwg Supports A swe -> nwy",
    ]);

    assert_eq!(Succeeds, get_state!(results, "ENG: A nwy -> swe"));
    assert_eq!(Fails, get_state!(results, "ENG: F nor -> nwy"));
    assert_eq!(Fails, get_state!(results, "RUS: A swe -> nwy"));
}

#[test]
fn t6g18_the_two_unit_in_one_area_bug_with_double_convoy() {
    let results = get_results(vec![
        "ENG: F nor convoys lon -> bel",
        "ENG: A hol Supports A lon -> bel",
        "ENG: A yor -> lon",
        "ENG: A lon -> bel",
        "ENG: A ruh Supports A lon -> bel",
        "FRA: F eng convoys bel -> lon",
        "FRA: A bel -> lon",
        "FRA: A wal Supports A bel -> lon",
    ]);
}

#[test]
fn t6h02_no_supports_from_retreating_unit() {
    let results = get_results(vec![
        "ENG: A lvp -> edi",
        "ENG: F yor Supports A lvp -> edi",
        "ENG: F nwy Hold",
        "GER: A kie Supports A ruh -> hol",
        "GER: A ruh -> hol",
        "RUS: F edi Hold",
        "RUS: A swe Supports A Finland -> nwy",
        "RUS: A Finland -> nwy",
        "RUS: F hol Hold",
    ]);
}

#[test]
fn t6h03_no_convoy_during_retreat() {
    let results = get_results(vec![
        "ENG: F nor Hold",
        "ENG: A hol Hold",
        "GER: F kie Supports A ruh -> hol",
        "GER: A ruh -> hol",
    ]);
}

#[test]
fn t6h04_no_other_moves_during_retreat() {
    let results = get_results(vec![
        "ENG: F nor Hold",
        "ENG: A hol Hold",
        "GER: F kie Supports A ruh -> hol",
        "GER: A ruh -> hol",
    ]);
}

#[test]
fn t6h05_a_unit_may_not_retreat_to_the_area_from_which_it_is_attacked() {
    let results = get_results(vec![
        "RUS: F con Supports F bla -> ank",
        "RUS: F bla -> ank",
        "TUR: F ank Hold",
    ]);
}

#[test]
fn t6h06_unit_may_not_retreat_to_a_contested_area() {
    let results = get_results(vec![
        "AUS: A bud Supports A tri -> vie",
        "AUS: A tri -> vie",
        "GER: A mun -> Bohemia",
        "GER: A sil -> Bohemia",
        "ITA: A vie Hold",
    ]);
}

#[test]
fn t6h07_multiple_retreat_to_same_area_will_disband_units() {
    let results = get_results(vec![
        "AUS: A bud Supports A tri -> vie",
        "AUS: A tri -> vie",
        "GER: A mun Supports A sil -> Bohemia",
        "GER: A sil -> Bohemia",
        "ITA: A vie Hold",
        "ITA: A Bohemia Hold",
    ]);
}

#[test]
fn t6h08_triple_retreat_to_same_area_will_disband_units() {
    let results = get_results(vec![
        "ENG: A lvp -> edi",
        "ENG: F yor Supports A lvp -> edi",
        "ENG: F nwy Hold",
        "GER: A kie Supports A ruh -> hol",
        "GER: A ruh -> hol",
        "RUS: F edi Hold",
        "RUS: A swe Supports A Finland -> nwy",
        "RUS: A Finland -> nwy",
        "RUS: F hol Hold",
    ]);
}

#[test]
fn t6h09_dislodged_unit_will_not_make_attackers_area_contested() {
    let results = get_results(vec![
        "ENG: F hel -> kie",
        "ENG: F den Supports F hel -> kie",
        "GER: A ber -> pru",
        "GER: F kie Hold",
        "GER: A sil Supports A ber -> pru",
        "RUS: A pru -> ber",
    ]);
}

#[test]
fn t6h10_not_retreating_to_attacker_does_not_mean_contested() {
    let results = get_results(vec![
        "ENG: A kie Hold",
        "GER: A ber -> kie",
        "GER: A mun Supports A ber -> kie",
        "GER: A pru Hold",
        "RUS: A war -> pru",
        "RUS: A sil Supports A war -> pru",
    ]);
}

#[test]
fn t6h11_retreat_when_dislodged_by_adjacent_convoy() {
    let results = get_results(vec![
        "FRA: A gas -> mar via Convoy",
        "FRA: A bur Supports A gas -> mar",
        "FRA: F mao convoys gas -> mar",
        "FRA: F wes convoys gas -> mar",
        "FRA: F lyo convoys gas -> mar",
        "ITA: A mar Hold",
    ]);
}

#[test]
fn t6h12_retreat_when_dislodged_by_adjacent_convoy_while_trying_to_do_the_same() {
    let results = get_results(vec![
        "ENG: A lvp -> edi via Convoy",
        "ENG: F iri convoys lvp -> edi",
        "ENG: F eng convoys lvp -> edi",
        "ENG: F nor convoys lvp -> edi",
        "FRA: F bre -> eng",
        "FRA: F mao Supports F bre -> eng",
        "RUS: A edi -> lvp via Convoy",
        "RUS: F nwg convoys edi -> lvp",
        "RUS: F nao convoys edi -> lvp",
        "RUS: A cly Supports A edi -> lvp",
    ]);
}

#[test]
fn t6h13_no_retreat_with_convoy_in_main_phase() {
    let results = get_results(vec![
        "ENG: A pic Hold",
        "ENG: F eng convoys pic -> lon",
        "FRA: A par -> pic",
        "FRA: A bre Supports A par -> pic",
    ]);
}

#[test]
fn t6h14_no_retreat_with_support_in_main_phase() {
    let results = get_results(vec![
        "ENG: A pic Hold",
        "ENG: F eng Supports A pic -> bel",
        "FRA: A par -> pic",
        "FRA: A bre Supports A par -> pic",
        "FRA: A bur Hold",
        "GER: A mun Supports A mar -> bur",
        "GER: A mar -> bur",
    ]);
}

#[test]
fn t6h15_no_coastal_crawl_in_retreat() {
    let results = get_results(vec![
        "ENG: F por Hold",
        "FRA: F spa(sc) -> por",
        "FRA: F mao Supports F spa(sc) -> por",
    ]);
}

#[test]
fn t6h16_contested_for_both_coasts() {
    let results = get_results(vec![
        "FRA: F mao -> spa(nc)",
        "FRA: F gas -> spa(nc)",
        "FRA: F wes Hold",
        "ITA: F tun Supports F tyr -> wes",
        "ITA: F tyr -> wes",
    ]);
}

#[test]
#[ignore]
fn t6i02_fleets_can_not_be_build_in_land_areas() {
    let results = get_results(vec!["RUS: Build F mos"]);
}

#[test]
#[ignore]
fn t6i03_supply_center_must_be_empty_for_building() {
    let results = get_results(vec!["GER: Build A ber"]);
}

#[test]
#[ignore]
fn t6i04_both_coasts_must_be_empty_for_building() {
    let results = get_results(vec!["RUS: Build A St Petersburg(nc)"]);
}

#[test]
#[ignore]
fn t6i05_building_in_home_supply_center_that_is_not_owned() {
    let results = get_results(vec!["GER: Build A ber"]);
}

#[test]
#[ignore]
fn t6i06_building_in_owned_supply_center_that_is_not_a_home_supply_center() {
    let results = get_results(vec!["GER: Build A war"]);
}

#[test]
#[ignore]
fn t6i07_only_one_build_in_a_home_supply_center() {
    let results = get_results(vec!["RUS: Build A mos", "RUS: Build A mos"]);
}

#[test]
#[ignore]
fn t6j02_removing_the_same_unit_twice() {
    let results = get_results(vec!["FRA: Remove A par", "FRA: Remove A par"]);
}

#[test]
#[ignore]
fn t6j03_civil_disorder_two_armies_with_different_distance() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j06_civil_disorder_two_fleets_with_equal_distance() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j07_civil_disorder_two_fleets_and_army_with_equal_distance() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j08_civil_disorder_a_fleet_with_shorter_distance_then_the_army() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j09_civil_disorder_must_be_counted_from_both_coasts() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j10_civil_disorder_counting_convoying_distance() {
    let results = get_results(vec![""]);
}

#[test]
#[ignore]
fn t6j11_civil_disorder_counting_distance_without_convoying_fleet() {
    let results = get_results(vec![""]);
}
