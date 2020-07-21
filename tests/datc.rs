#![cfg(test)]
#![allow(unused_variables, clippy::let_and_return)]

#[path = "./util.rs"]
mod util;

use diplomacy::judge::OrderState::{Fails, Succeeds};
use diplomacy::judge::retreat::DestStatus;
use util::*;

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.1
#[test]
fn t6a01_move_to_non_neighbor_fails() {
    judge! { "ENG: F nth -> pic": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.2
#[test]
fn t6a02_move_army_to_sea() {
    judge! { "ENG: A lvp -> iri": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.3
#[test]
fn t6a03_move_fleet_to_land() {
    judge! { "GER: F kie -> mun": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.4
#[test]
fn t6a04_move_to_own_sector() {
    judge! { "GER: F kie -> kie": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.5
#[test]
fn t6a05_move_to_own_sector_with_convoy() {
    judge! {
        "ENG: F nth convoys yor -> yor",
        "ENG: A yor -> yor": Fails,
        "ENG: A lvp supports A yor -> yor": Succeeds,
        "GER: F lon -> yor": Succeeds,
        "GER: A wal supports F lon -> yor": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.6
#[test]
#[ignore]
fn t6a06_ordering_a_unit_of_another_country() {
    judge! { "GER: F lon -> nth" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.7
#[test]
fn t6a07_only_armies_can_be_convoyed() {
    judge! {
        "ENG: F lon -> bel": Fails,
        "ENG: F nth convoys lon -> bel",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.8
#[test]
fn t6a08_support_to_hold_yourself_is_not_possible() {
    judge! {
        "ITA: A ven -> tri": Succeeds,
        "ITA: A tyr supports A ven -> tri": Succeeds,
        "AUS: F tri supports F tri": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.9
#[test]
fn t6a09_fleets_must_follow_coast_if_not_on_sea() {
    judge! { "ITA: F rom -> ven": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.10
#[test]
fn t6a10_support_on_unreachable_destination_not_possible() {
    judge! {
        "AUS: A ven holds": Succeeds,
        "ITA: F rom supports A apu -> ven",
        "ITA: A apu -> ven": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.11
#[test]
fn t6a11_simple_bounce() {
    judge! {
       "AUS: A vie -> tyr": Fails,
       "ITA: A ven -> tyr": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.A.12
#[test]
fn t6a12_bounce_of_three_units() {
    judge! {
       "AUS: A vie -> tyr": Fails,
       "ITA: A ven -> tyr": Fails,
       "GER: A mun -> tyr": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.1
#[test]
fn t6b01_moving_without_required_coast_fails() {
    judge! { "FRA: F por -> spa": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.2
#[test]
fn t6b02_moving_with_unspecified_coast_when_coast_is_not_necessary() {
    judge! { "FRA: F gas -> spa": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.3
#[test]
fn t6b03_moving_with_wrong_coast_when_coast_is_not_necessary() {
    judge! { "FRA: F gas -> spa(sc)": Fails };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.4
#[test]
fn t6b04_support_to_unreachable_coast_allowed() {
    judge! {
        "FRA: F gas -> spa(nc)": Succeeds,
        "FRA: F mar supports F gas -> spa(nc)",
        "ITA: F wes -> spa(sc)": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.5
#[test]
fn t6b05_support_from_unreachable_coast_not_allowed() {
    judge! {
       "FRA: F mar -> lyo": Fails,
       "FRA: F spa(nc) supports F mar -> lyo",
       "ITA: F lyo holds": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.6
#[test]
fn t6b06_support_can_be_cut_with_other_coast() {
    judge! {
       "ENG: F iri supports F nao -> mao": Succeeds,
       "ENG: F nao -> mao": Succeeds,
       "FRA: F spa(nc) supports F mao": Fails,
       "FRA: F mao holds": Fails,
       "ITA: F lyo -> spa(sc)": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.7
#[test]
#[ignore]
fn t6b07_supporting_with_unspecified_coast() {
    judge! {
       "FRA: F por Supports F mao -> spa",
       "FRA: F mao -> spa(nc)",
       "ITA: F lyo Supports F wes -> spa(sc)",
       "ITA: F wes -> spa(sc)",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.8
#[test]
#[ignore]
fn t6b08_supporting_with_unspecified_coast_when_only_one_coast_is_possible() {
    judge! {
       "FRA: F por Supports F gas -> spa",
       "FRA: F gas -> spa(nc)",
       "ITA: F lyo Supports F wes -> spa(sc)",
       "ITA: F wes -> spa(sc)",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.9
#[test]
#[ignore]
fn t6b09_supporting_with_wrong_coast() {
    judge! {
       "FRA: F por Supports F mao -> spa(nc)",
       "FRA: F mao -> spa(sc)",
       "ITA: F lyo Supports F wes -> spa(sc)",
       "ITA: F wes -> spa(sc)",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.10
#[test]
#[ignore]
fn t6b10_unit_ordered_with_wrong_coast() {
    judge! { "FRA: F spa(nc) -> lyo" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.11
#[test]
#[ignore]
fn t6b11_coast_can_not_be_ordered_to_change() {
    judge! { "FRA: F spa(sc) -> lyo" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.12
#[test]
#[ignore]
fn t6b12_army_movement_with_coastal_specification() {
    judge! { "FRA: A gas -> spa(nc)" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.13
#[test]
fn t6b13_coastal_crawl_not_allowed() {
    judge! {
       "TUR: F bul(sc) -> con": Fails,
       "TUR: F con -> bul(ec)": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.B.14
#[test]
#[ignore]
fn t6b14_building_with_unspecified_coast() {
    judge! { "RUS: Build F St Petersburg" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.1
#[test]
fn t6c01_three_army_circular_movement_succeeds() {
    judge! {
       "TUR: F ank -> con": Succeeds,
       "TUR: A con -> smy": Succeeds,
       "TUR: A smy -> ank": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.2
#[test]
fn t6c02_three_army_circular_movement_with_support() {
    judge! {
       "TUR: F ank -> con": Succeeds,
       "TUR: A con -> smy": Succeeds,
       "TUR: A smy -> ank": Succeeds,
       "TUR: A bul supports F ank -> con": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.3
#[test]
fn t6c03_a_disrupted_three_army_circular_movement() {
    judge! {
       "TUR: F ank -> con": Fails,
       "TUR: A bul -> con": Fails,
       "TUR: A smy -> ank": Fails,
       "TUR: A con -> smy": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.4
#[test]
fn t6c04_a_circular_movement_with_attacked_convoy() {
    judge! {
       "AUS: A tri -> ser": Succeeds,
       "AUS: A ser -> bul": Succeeds,
       "TUR: A bul -> tri": Succeeds,
       "TUR: F aeg convoys bul -> tri",
       "TUR: F ion convoys bul -> tri",
       "TUR: F adr convoys bul -> tri",
       "ITA: F nap -> ion": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.5
#[test]
fn t6c05_a_disrupted_circular_movement_due_to_dislodged_convoy() {
    judge! {
       "AUS: A tri -> ser": Fails,
       "AUS: A ser -> bul": Fails,
       "TUR: A bul -> tri": Fails,
       "TUR: F aeg convoys bul -> tri",
       "TUR: F ion convoys bul -> tri",
       "TUR: F adr convoys bul -> tri",
       "ITA: F nap -> ion": Succeeds,
       "ITA: F tun supports F nap -> ion",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.6
#[test]
fn t6c06_two_armies_with_two_convoys() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A lon -> bel": Succeeds,
       "FRA: F eng convoys bel -> lon",
       "FRA: A bel -> lon": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.C.7
#[test]
fn t6c07_disrupted_unit_swap() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A lon -> bel": Fails,
       "FRA: F eng convoys bel -> lon",
       "FRA: A bel -> lon": Fails,
       "FRA: A bur -> bel": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.1
#[test]
fn t6d01_supported_hold_can_prevent_dislodgement() {
    judge! {
       "AUS: F adr supports A tri -> ven",
       "AUS: A tri -> ven": Fails,
       "ITA: A ven hold": Succeeds,
       "ITA: A tyr supports A ven",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.2
#[test]
fn t6d02_a_move_cuts_support_on_hold() {
    judge! {
       "AUS: F adr supports A tri -> ven",
       "AUS: A tri -> ven": Succeeds,
       "AUS: A vie -> tyr": Fails,
       "ITA: A ven hold": Fails,
       "ITA: A tyr supports A ven",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.3
#[test]
fn t6d03_a_move_cuts_support_on_move() {
    judge! {
       "AUS: F adr supports A tri -> ven",
       "AUS: A tri -> ven": Fails,
       "ITA: A ven hold": Succeeds,
       "ITA: F ion -> adr": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.4
#[test]
fn t6d04_support_to_hold_on_unit_supporting_a_hold_allowed() {
    judge! {
       "GER: A ber supports F kie",
       "GER: F kie supports A ber",
       "RUS: F bal supports A pru -> ber",
       "RUS: A pru -> ber": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.5
#[test]
fn t6d05_support_to_hold_on_unit_supporting_a_move_allowed() {
    judge! {
       "GER: A ber supports A mun -> sil",
       "GER: F kie supports A ber",
       "GER: A mun -> sil",
       "RUS: F bal supports A pru -> ber",
       "RUS: A pru -> ber": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.6
#[test]
fn t6d06_support_to_hold_on_convoying_unit_allowed() {
    judge! {
       "GER: A ber -> swe": Succeeds,
       "GER: F bal convoys ber -> swe",
       "GER: F pru Supports F bal",
       "RUS: F lvn -> bal": Fails,
       "RUS: F bot Supports F lvn -> bal",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.7
#[test]
fn t6d07_support_to_hold_on_moving_unit_not_allowed() {
    judge! {
       "GER: F bal -> swe": Fails,
       "GER: F pru Supports F bal",
       "RUS: F lvn -> bal": Succeeds,
       "RUS: F bot Supports F lvn -> bal",
       "RUS: A fin -> swe": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.8
#[test]
fn t6d08_failed_convoy_can_not_receive_hold_support() {
    judge! {
       "AUS: F ion hold",
       "AUS: A ser Supports A alb -> gre",
       "AUS: A alb -> gre": Succeeds,
       "TUR: A gre -> nap": Fails,
       "TUR: A bul Supports A gre",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.9
#[test]
fn t6d09_support_to_move_on_holding_unit_not_allowed() {
    let results = judge! {
       "ITA: A ven -> tri": Succeeds,
       "ITA: A tyr supports A ven -> tri",
       "AUS: A alb supports A tri -> ser",
       "AUS: A tri holds": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.10
#[test]
fn t6d10_self_dislodgment_prohibited() {
    judge! {
       "GER: A ber Hold": Succeeds,
       "GER: F kie -> ber": Fails,
       "GER: A mun Supports F kie -> ber",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.11
#[test]
fn t6d11_no_self_dislodgment_of_returning_unit() {
    judge! {
       "GER: A ber -> pru": Fails,
       "GER: F kie -> ber": Fails,
       "GER: A mun Supports F kie -> ber",
       "RUS: A war -> pru": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.12
#[test]
fn t6d12_supporting_a_foreign_unit_to_dislodge_own_unit_prohibited() {
    judge! {
       "AUS: F tri Hold": Succeeds,
       "AUS: A vie Supports A ven -> tri",
       "ITA: A ven -> tri": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.13
#[test]
fn t6d13_supporting_a_foreign_unit_to_dislodge_a_returning_own_unit_prohibited() {
    judge! {
       "AUS: F tri -> adr": Fails,
       "AUS: A vie Supports A ven -> tri",
       "ITA: A ven -> tri": Fails,
       "ITA: F apu -> adr": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.14
#[test]
fn t6d14_supporting_a_foreign_unit_is_not_enough_to_prevent_dislodgement() {
    judge! {
       "AUS: F tri Hold": Fails,
       "AUS: A vie Supports A ven -> tri",
       "ITA: A ven -> tri": Succeeds,
       "ITA: A tyr Supports A ven -> tri",
       "ITA: F adr Supports A ven -> tri",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.15
#[test]
fn t6d15_defender_can_not_cut_support_for_attack_on_itself() {
    judge! {
       "RUS: F con Supports F bla -> ank",
       "RUS: F bla -> ank": Succeeds,
       "TUR: F ank -> con": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.17
#[test]
fn t6d17_dislodgement_cuts_supports() {
    judge! {
       "RUS: F con Supports F bla -> ank": Fails,
       "RUS: F bla -> ank": Fails,
       "TUR: F ank -> con": Succeeds,
       "TUR: A smy Supports F ank -> con",
       "TUR: A arm -> ank": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.18
#[test]
fn t6d18_a_surviving_unit_will_sustain_support() {
    judge! {
       "RUS: F con Supports F bla -> ank": Succeeds,
       "RUS: F bla -> ank": Succeeds,
       "RUS: A bul Supports F con": Succeeds,
       "TUR: F ank -> con": Fails,
       "TUR: A smy Supports F ank -> con": Succeeds,
       "TUR: A arm -> ank": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.19
#[test]
fn t6d19_even_when_surviving_is_in_alternative_way() {
    judge! {
       "RUS: F con Supports F bla -> ank",
       "RUS: F bla -> ank": Succeeds,
       "RUS: A smy Supports F ank -> con",
       "TUR: F ank -> con": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.20
#[test]
fn t6d20_unit_can_not_cut_support_of_its_own_country() {
    judge! {
       "ENG: F lon Supports F nth -> eng",
       "ENG: F nth -> eng": Succeeds,
       "ENG: A yor -> lon": Fails,
       "FRA: F eng Hold": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.21
#[test]
fn t6d21_dislodging_does_not_cancel_a_support_cut() {
    judge! {
       "AUS: F tri Hold": Succeeds,
       "ITA: A ven -> tri": Fails,
       "ITA: A tyr supports A ven -> tri",
       "GER: A mun -> tyr": Fails,
       "RUS: A sil -> mun": Succeeds,
       "RUS: A ber Supports A sil -> mun",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.22
#[test]
fn t6d22_impossible_fleet_move_can_not_be_supported() {
    judge! {
       "GER: F kie -> mun",
       "GER: A bur Supports F kie -> mun",
       "RUS: A mun -> kie",
       "RUS: A ber Supports A mun -> kie",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.24
#[test]
fn t6d24_impossible_army_move_can_not_be_supported() {
    judge! {
       "FRA: A mar -> lyo",
       "FRA: F spa(sc) Supports A mar -> lyo",
       "ITA: F lyo Hold",
       "TUR: F tys Supports F wes -> lyo",
       "TUR: F wes -> lyo",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.25
#[test]
fn t6d25_failing_hold_support_can_be_supported() {
    judge! {
       "GER: A ber Supports A pru",
       "GER: F kie Supports A ber",
       "RUS: F bal Supports A pru -> ber",
       "RUS: A pru -> ber",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.26
#[test]
fn t6d26_failing_move_support_can_be_supported() {
    judge! {
       "GER: A ber Supports A pru -> sil",
       "GER: F kie Supports A ber",
       "RUS: F bal Supports A pru -> ber",
       "RUS: A pru -> ber",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.27
#[test]
fn t6d27_failing_convoy_can_be_supported() {
    judge! {
       "ENG: F swe -> bal",
       "ENG: F den Supports F swe -> bal",
       "GER: A ber Hold",
       "RUS: F bal convoys ber -> lvn",
       "RUS: F pru Supports F bal",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.28
#[test]
#[ignore]
fn t6d28_impossible_move_and_support() {
    judge! {
       "AUS: A bud Supports F rum",
       "RUS: F rum -> hol",
       "TUR: F bla -> rum",
       "TUR: A bul Supports F bla -> rum",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.29
#[test]
#[ignore]
fn t6d29_move_to_impossible_coast_and_support() {
    judge! {
       "AUS: A bud Supports F rum",
       "RUS: F rum -> bul(sc)",
       "TUR: F bla -> rum",
       "TUR: A bul Supports F bla -> rum",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.30
#[test]
#[ignore]
fn t6d30_move_without_coast_and_support() {
    judge! {
       "ITA: F aeg Supports F con",
       "RUS: F con -> bul",
       "TUR: F bla -> con",
       "TUR: A bul Supports F bla -> con",
    };
}

/// In this case the proposed behavior is that the fleet order should be treated as illegal and
/// dropped entirely. It's not clear why that would be the case in computerized games, so this
/// test will remain ignored.
///
/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.31
#[test]
#[ignore]
fn t6d31_a_tricky_impossible_support() {
    judge! {
       "AUS: A rum -> arm",
       "TUR: F bla Supports A rum -> arm",
    };
}

/// In this case it is proposed that the army order be treated as illegal and dropped entirely.
/// It's not clear why that should be the case, so this will remain ignored for now.
///
/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.32
#[test]
#[ignore]
fn t6d32_a_missing_fleet() {
    judge! {
       "ENG: F edi Supports A lvp -> yor",
       "ENG: A lvp -> yor",
       "FRA: F lon Supports A yor",
       "GER: A yor -> hol",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.33
#[test]
fn t6d33_unwanted_support_allowed() {
    judge! {
       "AUS: A ser -> bud": Succeeds,
       "AUS: A vie -> bud": Fails,
       "RUS: A gal supports A ser -> bud",
       "TUR: A bul -> ser": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.D.34
#[test]
fn t6d34_support_targeting_own_area_not_allowed() {
    judge! {
       "GER: A ber -> pru": Succeeds,
       "GER: A sil supports A ber -> pru",
       "GER: F bal supports A ber -> pru",
       "ITA: A pru supports A lvn -> pru": Fails,
       "RUS: A war supports A lvn -> pru",
       "RUS: A lvn -> pru": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.1
#[test]
fn t6e01_dislodged_unit_has_no_effect_on_attacker_area() {
    judge! {
       "GER: A ber -> pru": Succeeds,
       "GER: F kie -> ber": Succeeds,
       "GER: A sil supports A ber -> pru",
       "RUS: A pru -> ber": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.2
#[test]
fn t6e02_no_self_dislodgement_in_head_to_head_battle() {
    judge! {
       "GER: A ber -> kie": Fails,
       "GER: F kie -> ber": Fails,
       "GER: A mun Supports A ber -> kie",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.3
#[test]
fn t6e03_no_help_in_dislodging_own_unit() {
    judge! {
       "GER: A ber -> kie": Fails,
       "GER: A mun supports F kie -> ber",
       "ENG: F kie -> ber": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.4
#[test]
fn t6e04_non_dislodged_loser_has_still_effect() {
    judge! {
       "GER: F hol -> nth": Fails,
       "GER: F hel Supports F hol -> nth",
       "GER: F ska Supports F hol -> nth",
       "FRA: F nth -> hol": Fails,
       "FRA: F bel Supports F nth -> hol",
       "ENG: F edi Supports F nwg -> nth",
       "ENG: F yor Supports F nwg -> nth",
       "ENG: F nwg -> nth": Fails,
       "AUS: A kie Supports A ruh -> hol",
       "AUS: A ruh -> hol": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.5
#[test]
fn t6e05_loser_dislodged_by_another_army_has_still_effect() {
    judge! {
       "GER: F hol -> nth": Fails,
       "GER: F hel Supports F hol -> nth",
       "GER: F ska Supports F hol -> nth",
       "FRA: F nth -> hol": Fails,
       "FRA: F bel Supports F nth -> hol",
       "ENG: F edi Supports F nwg -> nth",
       "ENG: F yor Supports F nwg -> nth",
       "ENG: F nwg -> nth": Succeeds,
       "ENG: F lon Supports F nwg -> nth",
       "AUS: A kie Supports A ruh -> hol",
       "AUS: A ruh -> hol": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.6
#[test]
fn t6e06_not_dislodge_because_of_own_support_has_still_effect() {
    judge! {
       "GER: F hol -> nth": Fails,
       "GER: F hel Supports F hol -> nth",
       "FRA: F nth -> hol": Fails,
       "FRA: F bel Supports F nth -> hol",
       "FRA: F eng Supports F hol -> nth",
       "AUS: A kie Supports A ruh -> hol",
       "AUS: A ruh -> hol": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.7
#[test]
fn t6e07_no_self_dislodgement_with_beleaguered_garrison() {
    judge! {
       "ENG: F nth Hold",
       "ENG: F yor Supports F nwy -> nth",
       "GER: F hol Supports F hel -> nth",
       "GER: F hel -> nth",
       "RUS: F ska Supports F nwy -> nth",
       "RUS: F nwy -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.8
#[test]
fn t6e08_no_self_dislodgement_with_beleaguered_garrison_and_head_to_head_battle() {
    judge! {
       "ENG: F nth -> nwy",
       "ENG: F yor Supports F nwy -> nth",
       "GER: F hol Supports F hel -> nth",
       "GER: F hel -> nth",
       "RUS: F ska Supports F nwy -> nth",
       "RUS: F nwy -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.9
#[test]
fn t6e09_almost_self_dislodgement_with_beleaguered_garrison() {
    judge! {
       "ENG: F nth -> nwg",
       "ENG: F yor Supports F nwy -> nth",
       "GER: F hol Supports F hel -> nth",
       "GER: F hel -> nth",
       "RUS: F ska Supports F nwy -> nth",
       "RUS: F nwy -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.10
#[test]
fn t6e10_almost_circular_movement_with_no_self_dislodgement_with_beleaguered_garrison() {
    judge! {
       "ENG: F nth -> den",
       "ENG: F yor Supports F nwy -> nth",
       "GER: F hol Supports F hel -> nth",
       "GER: F hel -> nth",
       "GER: F den -> hel",
       "RUS: F ska Supports F nwy -> nth",
       "RUS: F nwy -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.11
#[test]
fn t6e11_no_self_dislodgement_with_beleaguered_garrison_unit_swap_with_adjacent_convoying_and_two_coasts(
) {
    judge! {
       "FRA: A spa -> por via Convoy",
       "FRA: F mao convoys spa -> por",
       "FRA: F lyo Supports F por -> spa(nc)",
       "GER: A mar Supports A gas -> spa",
       "GER: A gas -> spa",
       "ITA: F por -> spa(nc)",
       "ITA: F wes Supports F por -> spa(nc)",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.12
#[test]
fn t6e12_support_on_attack_on_own_unit_can_be_used_for_other_means() {
    judge! {
       "AUS: A bud -> rum": Fails,
       "AUS: A ser Supports A vie -> bud",
       "ITA: A vie -> bud": Fails,
       "RUS: A gal -> bud": Fails,
       "RUS: A rum Supports A gal -> bud",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.13
#[test]
fn t6e13_three_way_beleaguered_garrison() {
    judge! {
       "ENG: F edi Supports F yor -> nth",
       "ENG: F yor -> nth": Fails,
       "FRA: F bel -> nth": Fails,
       "FRA: F eng Supports F bel -> nth",
       "GER: F nth Hold": Succeeds,
       "RUS: F nwg -> nth": Fails,
       "RUS: F nwy Supports F nwg -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.14
#[test]
fn t6e14_illegal_head_to_head_battle_can_still_defend() {
    judge! {
        "ENG: A lvp -> edi": Fails,
        "RUS: F edi -> lvp": Fails
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.E.15
#[test]
fn t6e15_the_friendly_head_to_head_battle() {
    judge! {
       "ENG: F hol Supports A ruh -> kie",
       "ENG: A ruh -> kie": Fails,
       "FRA: A kie -> ber": Fails,
       "FRA: A mun Supports A kie -> ber",
       "FRA: A sil Supports A kie -> ber",
       "GER: A ber -> kie": Fails,
       "GER: F den Supports A ber -> kie",
       "GER: F hel Supports A ber -> kie",
       "RUS: F bal Supports A pru -> ber",
       "RUS: A pru -> ber": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.1
#[test]
fn t6f01_no_convoy_in_coastal_areas() {
    judge! {
        "TUR: A gre -> sev": Fails,
        "TUR: F aeg convoys gre -> sev",
        "TUR: F con convoys gre -> sev",
        "TUR: F bla convoys gre -> sev",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.2
#[test]
fn t6f02_an_army_being_convoyed_can_bounce_as_normal() {
    judge! {
       "ENG: F eng convoys lon -> bre",
       "ENG: A lon -> bre": Fails,
       "FRA: A par -> bre": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.3
#[test]
fn t6f03_an_army_being_convoyed_can_receive_support() {
    judge! {
       "ENG: F eng convoys lon -> bre",
       "ENG: A lon -> bre": Succeeds,
       "ENG: F mao Supports A lon -> bre": Succeeds,
       "FRA: A par -> bre": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.4
#[test]
fn t6f04_an_attacked_convoy_is_not_disrupted() {
    judge! {
       "ENG: F nth convoys lon -> hol",
       "ENG: A lon -> hol": Succeeds,
       "GER: F ska -> nth": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.5
#[test]
fn t6f05_a_beleaguered_convoy_is_not_disrupted() {
    judge! {
       "ENG: F nth convoys lon -> hol",
       "ENG: A lon -> hol": Succeeds,
       "FRA: F eng -> nth": Fails,
       "FRA: F bel Supports F eng -> nth",
       "GER: F ska -> nth": Fails,
       "GER: F den Supports F ska -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.6
#[test]
fn t6f06_dislodged_convoy_does_not_cut_support() {
    judge! {
       "ENG: F nth convoys lon -> hol",
       "ENG: A lon -> hol",
       "GER: A hol Supports A bel",
       "GER: A bel Supports A hol",
       "GER: F hel Supports F ska -> nth",
       "GER: F ska -> nth",
       "FRA: A pic -> bel",
       "FRA: A bur Supports A pic -> bel",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.7
#[test]
fn t6f07_dislodged_convoy_does_not_cause_contested_area() {
    judge! {
       "ENG: F nth convoys lon -> hol",
       "ENG: A lon -> hol": Fails,
       "GER: F hel Supports F ska -> nth",
       "GER: F ska -> nth": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.8
#[test]
fn t6f08_dislodged_convoy_does_not_cause_a_bounce() {
    judge! {
       "ENG: F nth convoys lon -> hol",
       "ENG: A lon -> hol": Fails,
       "GER: F hel Supports F ska -> nth",
       "GER: F ska -> nth": Succeeds,
       "GER: A bel -> hol": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.9
#[test]
fn t6f09_dislodge_of_multi_route_convoy() {
    judge! {
       "ENG: F eng convoys lon -> bel",
       "ENG: F nth convoys lon -> bel",
       "ENG: A lon -> bel",
       "FRA: F bre Supports F mao -> eng",
       "FRA: F mao -> eng",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.10
#[test]
fn t6f10_dislodge_of_multi_route_convoy_with_foreign_fleet() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A lon -> bel",
       "GER: F eng convoys lon -> bel",
       "FRA: F bre Supports F mao -> eng",
       "FRA: F mao -> eng",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.11
#[test]
fn t6f11_dislodge_of_multi_route_convoy_with_only_foreign_fleets() {
    judge! {
       "ENG: A lon -> bel",
       "GER: F eng convoys lon -> bel",
       "RUS: F nth convoys lon -> bel",
       "FRA: F bre Supports F mao -> eng",
       "FRA: F mao -> eng",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.12
#[test]
fn t6f12_dislodged_convoying_fleet_not_on_route() {
    judge! {
       "ENG: F eng convoys lon -> bel",
       "ENG: A lon -> bel": Succeeds,
       "ENG: F iri convoys lon -> bel": Fails,
       "FRA: F nao Supports F mao -> iri",
       "FRA: F mao -> iri": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.13
#[test]
fn t6f13_the_unwanted_alternative() {
    judge! {
       "ENG: A lon -> bel",
       "ENG: F nth convoys lon -> bel",
       "FRA: F eng convoys lon -> bel",
       "GER: F hol Supports F den -> nth",
       "GER: F den -> nth",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.14
#[test]
fn t6f14_simple_convoy_paradox() {
    judge! {
        "ENG: F lon Supports F wal -> eng",
        "ENG: F wal -> eng": Succeeds,
        "FRA: A bre -> lon": Fails,
        "FRA: F eng convoys bre -> lon": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.15
#[test]
fn t6f15_simple_convoy_paradox_with_additional_convoy() {
    judge! {
       "ENG: F lon Supports F wal -> eng",
       "ENG: F wal -> eng": Succeeds,
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon",
       "ITA: F iri convoys naf -> wal",
       "ITA: F mao convoys naf -> wal",
       "ITA: A naf -> wal": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.16
#[test]
fn t6f16_pandins_paradox() {
    judge! {
       "ENG: F lon Supports F wal -> eng",
       "ENG: F wal -> eng": Fails,
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon",
       "GER: F nth Supports F bel -> eng",
       "GER: F bel -> eng": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.17
#[test]
fn t6f17_pandins_extended_paradox() {
    judge! {
       "ENG: F lon Supports F wal -> eng",
       "ENG: F wal -> eng": Fails,
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon",
       "FRA: F yor Supports A bre -> lon",
       "GER: F nth Supports F bel -> eng",
       "GER: F bel -> eng": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.18
#[test]
fn t6f18_betrayal_paradox() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A lon -> bel": Fails,
       "ENG: F eng Supports A lon -> bel",
       "FRA: F bel Supports F nth",
       "GER: F hel Supports F ska -> nth",
       "GER: F ska -> nth": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.19
#[test]
fn t6f19_multi_route_convoy_disruption_paradox() {
    judge! {
       "FRA: A tun -> nap": Fails,
       "FRA: F tys convoys tun -> nap",
       "FRA: F ion convoys tun -> nap",
       "ITA: F nap Supports F rom -> tys",
       "ITA: F rom -> tys": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.20
#[test]
fn t6f20_unwanted_multi_route_convoy_paradox() {
    judge! {
       "FRA: A tun -> nap",
       "FRA: F tys convoys tun -> nap",
       "ITA: F nap Supports F ion",
       "ITA: F ion convoys tun -> nap",
       "TUR: F aeg Supports F eas -> ion",
       "TUR: F eas -> ion": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.21
#[test]
fn t6f21_dads_army_convoy() {
    judge! {
       "RUS: A edi Supports A nwy -> cly",
       "RUS: F nwg convoys nwy -> cly",
       "RUS: A nwy -> cly",
       "FRA: F iri Supports F mao -> nao",
       "FRA: F mao -> nao": Succeeds,
       "ENG: A lvp -> cly via Convoy": Fails,
       "ENG: F nao convoys lvp -> cly",
       "ENG: F cly Supports F nao",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.22
#[test]
fn t6f22_second_order_paradox_with_two_resolutions() {
    judge! {
       "ENG: F edi -> nth": Succeeds,
       "ENG: F lon Supports F edi -> nth",
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon": Fails,
       "GER: F bel Supports F pic -> eng",
       "GER: F pic -> eng": Succeeds,
       "RUS: A nwy -> bel": Fails,
       "RUS: F nth convoys nwy -> bel": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.23
#[test]
fn t6f23_second_order_paradox_with_two_exclusive_convoys() {
    judge! {
       "ENG: F edi -> nth": Fails,
       "ENG: F yor Supports F edi -> nth",
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon",
       "GER: F bel Supports F eng",
       "GER: F lon Supports F nth",
       "ITA: F mao -> eng": Fails,
       "ITA: F iri Supports F mao -> eng",
       "RUS: A nwy -> bel": Fails,
       "RUS: F nth convoys nwy -> bel",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.F.24
#[test]
fn t6f24_second_order_paradox_with_no_resolution() {
    judge! {
       "ENG: F edi -> nth",
       "ENG: F lon Supports F edi -> nth",
       "ENG: F iri -> eng",
       "ENG: F mao Supports F iri -> eng",
       "FRA: A bre -> lon": Fails,
       "FRA: F eng convoys bre -> lon",
       "FRA: F bel Supports F eng",
       "RUS: A nwy -> bel": Fails,
       "RUS: F nth convoys nwy -> bel": Fails,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.2
#[test]
fn t6g02_kidnapping_an_army() {
    judge! {
       "ENG: A nwy -> swe": Succeeds,
       "RUS: F swe -> nwy": Succeeds,
       "GER: F ska convoys nwy -> swe",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.3
#[test]
#[ignore]
fn t6g03_kidnapping_with_a_disrupted_convoy() {
    judge! {
       "FRA: F bre -> eng",
       "FRA: A pic -> bel",
       "FRA: A bur Supports A pic -> bel",
       "FRA: F mao Supports F bre -> eng",
       "ENG: F eng convoys pic -> bel",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.4
#[test]
#[ignore]
fn t6g04_kidnapping_with_a_disrupted_convoy_and_opposite_move() {
    judge! {
       "FRA: F bre -> eng",
       "FRA: A pic -> bel",
       "FRA: A bur Supports A pic -> bel",
       "FRA: F mao Supports F bre -> eng",
       "ENG: F eng convoys pic -> bel",
       "ENG: A bel -> pic",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.5
#[test]
fn t6g05_swapping_with_intent() {
    judge! {
       "ITA: A rom -> apu": Succeeds,
       "ITA: F tys convoys apu -> rom",
       "TUR: A apu -> rom": Succeeds,
       "TUR: F ion convoys apu -> rom",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.6
#[test]
fn t6g06_swapping_with_unintended_intent() {
    judge! {
       "ENG: A lvp -> edi": Succeeds,
       "ENG: F eng convoys lvp -> edi",
       "GER: A edi -> lvp": Succeeds,
       "FRA: F iri Hold",
       "FRA: F nth Hold",
       "RUS: F nwg convoys lvp -> edi",
       "RUS: F nao convoys lvp -> edi",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.7
///
/// The current implementation diverges from the DATC preference.
/// The DATC says that the illegality of the English convoy order should be detected before
/// resolution, counting as an illegal order. This adjudicator instead prefers that the English
/// convoy still count as convoy intent.
#[test]
fn t6g07_swapping_with_illegal_intent() {
    judge! {
       "ENG: F ska convoys swe -> nwy",
       "ENG: F nwy -> swe": Succeeds,
       "RUS: A swe -> nwy": Succeeds,
       "RUS: F bot convoys swe -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.8
///
/// For this case, this adjudicator believes that "via Convoy" should be binding:
/// If the unit asked for a non-existent convoy, it should not move.
/// This diverges from the DATC opinion that this should be dropped if no such convoy exists.
#[test]
fn t6g08_explicit_convoy_that_isnt_there() {
    judge! {
       "FRA: A bel -> hol via Convoy": Fails,
       "ENG: F nth -> hel": Succeeds,
       "ENG: A hol -> kie": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.9
#[test]
fn t6g09_swapped_or_dislodged() {
    judge! {
       "ENG: A nwy -> swe": Succeeds,
       "ENG: F ska convoys nwy -> swe",
       "ENG: F fin Supports A nwy -> swe",
       "RUS: A swe -> nwy": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.10
#[test]
fn t6g10_swapped_or_an_head_to_head_battle() {
    judge! {
       "ENG: A nwy -> swe via Convoy": Succeeds,
       "ENG: F den Supports A nwy -> swe",
       "ENG: F fin Supports A nwy -> swe",
       "GER: F ska convoys nwy -> swe",
       "RUS: A swe -> nwy": Fails,
       "RUS: F bar supports A swe -> nwy",
       "FRA: F nwg -> nwy": Fails,
       "FRA: F nth Supports F nwg -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.11
#[test]
fn t6g11_a_convoy_to_an_adjacent_place_with_a_paradox() {
    judge! {
       "ENG: F nwy Supports F nth -> ska",
       "ENG: F nth -> ska": Succeeds,
       "RUS: A swe -> nwy": Fails,
       "RUS: F ska convoys swe -> nwy",
       "RUS: F bar Supports A swe -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.12
#[test]
fn t6g12_swapping_two_units_with_two_convoys() {
    judge! {
       "ENG: A lvp -> edi via Convoy": Succeeds,
       "ENG: F nao convoys lvp -> edi",
       "ENG: F nwg convoys lvp -> edi",
       "GER: A edi -> lvp via Convoy": Succeeds,
       "GER: F nth convoys edi -> lvp",
       "GER: F eng convoys edi -> lvp",
       "GER: F iri convoys edi -> lvp",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.13
#[test]
fn t6g13_support_cut_on_attack_on_itself_via_convoy() {
    judge! {
       "AUS: F adr convoys tri -> ven",
       "AUS: A tri -> ven via Convoy": Fails,
       "ITA: A ven Supports F alb -> tri",
       "ITA: F alb -> tri": Succeeds,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.14
#[test]
fn t6g14_bounce_by_convoy_to_adjacent_place() {
    judge! {
       "ENG: A nwy -> swe": Succeeds,
       "ENG: F den Supports A nwy -> swe",
       "ENG: F fin Supports A nwy -> swe",
       "FRA: F nwg -> nwy": Fails,
       "FRA: F nth Supports F nwg -> nwy",
       "GER: F ska convoys swe -> nwy",
       "RUS: A swe -> nwy via Convoy": Fails,
       "RUS: F bar Supports A swe -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.15
#[test]
fn t6g15_bounce_and_dislodge_with_double_convoy() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A hol Supports A lon -> bel",
       "ENG: A yor -> lon",
       "ENG: A lon -> bel via Convoy",
       "FRA: F eng convoys bel -> lon",
       "FRA: A bel -> lon via Convoy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.16
///
///
/// See decision details 5.B.6. If the 'PREVENT STRENGTH' is incorrectly implemented,
/// due to the fact that it does not take into account that the 'PREVENT STRENGTH'
/// is only zero when the unit is engaged in a head to head battle, then this goes
/// wrong in this test case. The 'PREVENT STRENGTH' of Sweden would be zero, because
/// the opposing unit in Norway successfully moves. Since, this strength would be zero,
/// the fleet in the North Sea would move to Norway.
///
/// However, although the 'PREVENT STRENGTH' is zero, the army in Sweden would also
/// move to Norway. So, the final result would contain two units that successfully
/// moved to Norway.
///
/// Of course, this is incorrect. Norway will indeed successfully move to Sweden
/// while the army in Sweden ends in Norway, because it is stronger than the fleet
/// in the North Sea. This fleet will stay in the North Sea.
#[test]
fn t6g16_the_two_unit_in_one_area_bug_moving_by_convoy() {
    judge! {
       "ENG: A nwy -> swe": Succeeds,
       "ENG: A den Supports A nwy -> swe",
       "ENG: F bal Supports A nwy -> swe",
       "ENG: F nth -> nwy": Fails,
       "RUS: A swe -> nwy via Convoy": Succeeds,
       "RUS: F ska convoys swe -> nwy",
       "RUS: F nwg Supports A swe -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.17
#[test]
fn t6g17_the_two_unit_in_one_area_bug_moving_over_land() {
    judge! {
       "ENG: A nwy -> swe via Convoy": Succeeds,
       "ENG: A den Supports A nwy -> swe",
       "ENG: F bal Supports A nwy -> swe",
       "ENG: F ska convoys nwy -> swe",
       "ENG: F nth -> nwy": Fails,
       "RUS: A swe -> nwy": Succeeds,
       "RUS: F nwg Supports A swe -> nwy",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.G.18
#[test]
fn t6g18_the_two_unit_in_one_area_bug_with_double_convoy() {
    judge! {
       "ENG: F nth convoys lon -> bel",
       "ENG: A hol Supports A lon -> bel",
       "ENG: A yor -> lon": Fails,
       "ENG: A lon -> bel": Succeeds,
       "ENG: A ruh Supports A lon -> bel",
       "FRA: F eng convoys bel -> lon",
       "FRA: A bel -> lon": Succeeds,
       "FRA: A wal Supports A bel -> lon",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.2
#[test]
fn t6h02_no_supports_from_retreating_unit() {
    judge! {
       "ENG: A lvp -> edi",
       "ENG: F yor Supports A lvp -> edi",
       "ENG: F nwy Hold",
       "GER: A kie Supports A ruh -> hol",
       "GER: A ruh -> hol",
       "RUS: F edi Hold",
       "RUS: A swe Supports A fin -> nwy",
       "RUS: A fin -> nwy",
       "RUS: F hol Hold",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.3
#[test]
fn t6h03_no_convoy_during_retreat() {
    judge! {
       "ENG: F nth Hold",
       "ENG: A hol Hold",
       "GER: F kie Supports A ruh -> hol",
       "GER: A ruh -> hol",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.4
#[test]
fn t6h04_no_other_moves_during_retreat() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "ENG: F nth Hold": Succeeds,
       "ENG: A hol Hold": Fails,
       "GER: F kie Supports A ruh -> hol",
       "GER: A ruh -> hol": Succeeds,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ENG: F nth -> nwg": InvalidRecipient,
        "ENG: A hol -> bel": Moves,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.5
#[test]
fn t6h05_a_unit_may_not_retreat_to_the_area_from_which_it_is_attacked() {
    let (context, expected) = context_and_expectation! {
       "RUS: F con Supports F bla -> ank",
       "RUS: F bla -> ank": Succeeds,
       "TUR: F ank Hold": Fails,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "TUR: F ank -> bla": DestStatus::BlockedByDislodger,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.6
#[test]
fn t6h06_unit_may_not_retreat_to_a_contested_area() {
    let (context, expected) = context_and_expectation! {
       "AUS: A bud Supports A tri -> vie",
       "AUS: A tri -> vie": Succeeds,
       "GER: A mun -> boh": Fails,
       "GER: A sil -> boh": Fails,
       "ITA: A vie Hold": Fails,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ITA: A vie -> boh": DestStatus::Contested,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.7
#[test]
fn t6h07_multiple_retreat_to_same_area_will_disband_units() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "AUS: A bud Supports A tri -> vie",
       "AUS: A tri -> vie": Succeeds,
       "GER: A mun Supports A sil -> boh",
       "GER: A sil -> boh": Succeeds,
       "ITA: A vie Hold": Fails,
       "ITA: A boh Hold": Fails,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ITA: A vie -> tyr": Prevented(&retreat_ord("ITA: A boh -> tyr")),
        "ITA: A boh -> tyr": Prevented(&retreat_ord("ITA: A vie -> tyr")),
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.8
#[test]
fn t6h08_triple_retreat_to_same_area_will_disband_units() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "ENG: A lvp -> edi": Succeeds,
       "ENG: F yor Supports A lvp -> edi",
       "ENG: F nwy Hold": Fails,
       "GER: A kie Supports A ruh -> hol",
       "GER: A ruh -> hol": Succeeds,
       "RUS: F edi Hold": Fails,
       "RUS: A swe Supports A fin -> nwy",
       "RUS: A fin -> nwy": Succeeds,
       "RUS: F hol Hold": Fails,
    };

    let outcome = resolve_main!(context, expected);

    // If this test fails because of the preventing order, that's okay.
    judge_retreat! {
        outcome,
        "ENG: F nwy -> nth": Prevented(&retreat_ord("RUS: F edi -> nth")),
        "RUS: F edi -> nth": Prevented(&retreat_ord("RUS: F hol -> nth")),
        "RUS: F hol -> nth": Prevented(&retreat_ord("RUS: F edi -> nth")),
    }
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.9
#[test]
fn t6h09_dislodged_unit_will_not_make_attackers_area_contested() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "ENG: F hel -> kie": Succeeds,
       "ENG: F den Supports F hel -> kie",
       "GER: A ber -> pru": Succeeds,
       "GER: F kie Hold": Fails,
       "GER: A sil Supports A ber -> pru",
       "RUS: A pru -> ber": Fails,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "GER: F kie -> ber": Moves,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.10
#[test]
fn t6h10_not_retreating_to_attacker_does_not_mean_contested() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "ENG: A kie Hold": Fails,
       "GER: A ber -> kie": Succeeds,
       "GER: A mun Supports A ber -> kie",
       "GER: A pru Hold": Fails,
       "RUS: A war -> pru": Succeeds,
       "RUS: A sil Supports A war -> pru",
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ENG: A kie -> ber": DestStatus::BlockedByDislodger,
        "GER: A pru -> ber": Moves,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.11
#[test]
fn t6h11_retreat_when_dislodged_by_adjacent_convoy() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "FRA: A gas -> mar via Convoy": Succeeds,
       "FRA: A bur Supports A gas -> mar",
       "FRA: F mao convoys gas -> mar",
       "FRA: F wes convoys gas -> mar",
       "FRA: F lyo convoys gas -> mar",
       "ITA: A mar Hold": Fails,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ITA: A mar -> gas": Moves
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.12
#[test]
fn t6h12_retreat_when_dislodged_by_adjacent_convoy_while_trying_to_do_the_same() {
    judge! {
       "ENG: A lvp -> edi via Convoy",
       "ENG: F iri convoys lvp -> edi",
       "ENG: F eng convoys lvp -> edi",
       "ENG: F nth convoys lvp -> edi",
       "FRA: F bre -> eng",
       "FRA: F mao Supports F bre -> eng",
       "RUS: A edi -> lvp via Convoy",
       "RUS: F nwg convoys edi -> lvp",
       "RUS: F nao convoys edi -> lvp",
       "RUS: A cly Supports A edi -> lvp",
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.13
#[test]
fn t6h13_no_retreat_with_convoy_in_main_phase() {
    let (context, expected) = context_and_expectation! {
       "ENG: A pic Hold": Fails,
       "ENG: F eng convoys pic -> lon",
       "FRA: A par -> pic": Succeeds,
       "FRA: A bre Supports A par -> pic",
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ENG: A pic -> lon": DestStatus::Unreachable,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.14
#[test]
fn t6h14_no_retreat_with_support_in_main_phase() {
    use diplomacy::judge::retreat::OrderOutcome::*;

    let (context, expected) = context_and_expectation! {
       "ENG: A pic Hold": Fails,
       "ENG: F eng Supports A pic -> bel",
       "FRA: A par -> pic": Succeeds,
       "FRA: A bre Supports A par -> pic",
       "FRA: A bur Hold": Fails,
       "GER: A mun Supports A mar -> bur",
       "GER: A mar -> bur": Succeeds,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ENG: A pic -> bel": Prevented(&retreat_ord("FRA: A bur -> bel")),
        "FRA: A bur -> bel": Prevented(&retreat_ord("ENG: A pic -> bel")),
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.15
#[test]
fn t6h15_no_coastal_crawl_in_retreat() {
    let (context, expected) = context_and_expectation! {
       "ENG: F por Hold": Fails,
       "FRA: F spa(sc) -> por": Succeeds,
       "FRA: F mao Supports F spa(sc) -> por",
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "ENG: F por -> spa(nc)": DestStatus::BlockedByDislodger,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.H.16
#[test]
fn t6h16_contested_for_both_coasts() {
    let (context, expected) = context_and_expectation! {
       "FRA: F mao -> spa(nc)": Fails,
       "FRA: F gas -> spa(nc)": Fails,
       "FRA: F wes Hold": Fails,
       "ITA: F tun Supports F tys -> wes",
       "ITA: F tys -> wes": Succeeds,
    };

    let outcome = resolve_main!(context, expected);

    judge_retreat! {
        outcome,
        "FRA: F wes -> spa(sc)": DestStatus::Contested,
    };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.2
#[test]
#[ignore]
fn t6i02_fleets_can_not_be_build_in_land_areas() {
    judge! { "RUS: Build F mos" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.3
#[test]
#[ignore]
fn t6i03_supply_center_must_be_empty_for_building() {
    judge! { "GER: Build A ber" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.4
#[test]
#[ignore]
fn t6i04_both_coasts_must_be_empty_for_building() {
    judge! { "RUS: Build A St Petersburg(nc)" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.5
#[test]
#[ignore]
fn t6i05_building_in_home_supply_center_that_is_not_owned() {
    judge! { "GER: Build A ber" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.6
#[test]
#[ignore]
fn t6i06_building_in_owned_supply_center_that_is_not_a_home_supply_center() {
    judge! { "GER: Build A war" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.I.7
#[test]
#[ignore]
fn t6i07_only_one_build_in_a_home_supply_center() {
    judge! { "RUS: Build A mos", "RUS: Build A mos" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.2
#[test]
#[ignore]
fn t6j02_removing_the_same_unit_twice() {
    judge! { "FRA: Remove A par", "FRA: Remove A par" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.3
#[test]
#[ignore]
fn t6j03_civil_disorder_two_armies_with_different_distance() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.6
#[test]
#[ignore]
fn t6j06_civil_disorder_two_fleets_with_equal_distance() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.7
#[test]
#[ignore]
fn t6j07_civil_disorder_two_fleets_and_army_with_equal_distance() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.8
#[test]
#[ignore]
fn t6j08_civil_disorder_a_fleet_with_shorter_distance_then_the_army() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.9
#[test]
#[ignore]
fn t6j09_civil_disorder_must_be_counted_from_both_coasts() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.10
#[test]
#[ignore]
fn t6j10_civil_disorder_counting_convoying_distance() {
    judge! { "" };
}

/// http://web.inter.nl.net/users/L.B.Kruijswijk/#6.J.11
#[test]
#[ignore]
fn t6j11_civil_disorder_counting_distance_without_convoying_fleet() {
    judge! { "" };
}
