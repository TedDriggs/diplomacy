#[path = "./util.rs"]
mod util;

use diplomacy::judge::AttackOutcome::*;
use util::*;

#[test]
fn dipmath_fig16() {
    judge! {
        "TUR: F aeg -> ion": Succeeds,
        "TUR: F gre supports F aeg -> ion",
        "AUS: F alb supports F aeg -> ion",
        "ITA: A tun -> gre": NoPath,
        "ITA: F ion convoys tun -> gre",
    };
}
