#![cfg(test)]

extern crate diplomacy;
mod util;
mod basic;

use diplomacy::order::{Order, MainCommand, SupportedOrder};
use diplomacy::geo;
use diplomacy::judge::{adjudicate, OrderState};

use diplomacy::{Nation, UnitType};

use util::*;

#[test]
fn it_works() {
    
}

#[test]
fn dipmath_figure6() {
    let map = geo::standard_map();
    let eng = Nation("eng".into());
    let ger = Nation("ger".into());
    let rus = Nation("rus".into());
    
    let orders = vec![
        Order::new(eng, UnitType::Fleet, reg("nwg"), MainCommand::Move(reg("nth"))),
        Order::new(ger.clone(), UnitType::Fleet, reg("ska"), MainCommand::Support(SupportedOrder::Move(reg("nth"), reg("nwy")))),
        Order::new(ger.clone(), UnitType::Fleet, reg("nth"), MainCommand::Move(reg("nwy"))),
        Order::new(rus, UnitType::Fleet, reg("nwy"), MainCommand::Move(reg("nwg")))
    ];
    
    let result = adjudicate(&map, orders);
    for (o, r) in result.iter() {
        println!("{} {:?}", o, r);
    }
    
    for (_, r) in result.iter() {
        assert_eq!(&OrderState::Succeeds, r);
    }
}

#[test]
fn dipmath_figure7() {
    let map = geo::standard_map();
}