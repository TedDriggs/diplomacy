#[path = "./world.rs"]
mod world;

#[path = "./util.rs"]
mod util;

use diplomacy::judge::OrderState::*;
use util::*;

/// In this example the French army in Munich supports the move of the German army
/// in Ruhr instead of the French army in Burgundy. This makes that the ATTACK STRENGTH,
/// the PREVENT STRENGTH and the DEFEND STRENGTH of the German army in Ruhr are all different.
/// The ATTACK STRENGTH is one, because the French support should not be counted for the attack.
/// The PREVENT STRENGTH is zero, because it is dislodged by the French army in Burgundy
/// and therefore it can not prevent the army in Marseilles to go to Burgundy. However, the
/// DEFEND STRENGTH contains all supports and is therefore two. Still this DEFEND STRENGTH
/// is insufficient in the head to head battle, since the French army in Burgundy has an
/// ATTACK STRENGTH of three.
///
/// See [`Rulebook::adjudicate`].
#[test]
fn all_strengths_different() {
    judge! {
        "FRA: A bel supports A bur -> ruh",
        "FRA: A hol Supports A bur -> ruh",
        "FRA: A bur -> ruh": Succeeds,
        "FRA: A mun supports A ruh -> bur",
        "FRA: A mar -> bur": Succeeds,
        "GER: A ruh -> bur": Fails
    };
}

/// Variation of the above test where there is not enough strength for the attack
/// to succeed without the French support against themselves.
///
/// Right now, this is resulting in both combatants getting `LostHeadToHead` as
/// their outcome, which seems incorrect, as `OccupierDefended` would be more accurate.
#[test]
fn all_strengths_different_no_movement() {
    judge! {
        "FRA: A bel supports A bur -> ruh",
        "FRA: A bur -> ruh": Fails,
        "FRA: A mun supports A ruh -> bur",
        "FRA: A mar -> bur": Fails,
        "GER: A ruh -> bur": Fails
    };
}
