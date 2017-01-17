#![cfg(test)]
extern crate diplomacy;

use diplomacy::order::{MainCommand, Order, SupportedOrder};
use diplomacy::{Nation, UnitType};

#[test]
fn support_equals() {
    let ger = Nation("ger".into());
    // let orders = vec![
    //     // Order::new(ger.clone(), UnitType::Fleet, reg("ska"), MainCommand::Support(SupportedOrder::Move(reg("nth"), reg("nwy")))),
    //     // Order::new(ger.clone(), UnitType::Fleet, reg("nth"), MainCommand::Move(reg("nwy")))
    // ];
}