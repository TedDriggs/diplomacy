#[path = "./util.rs"]
mod util;

use diplomacy::judge::{AttackOutcome, MappedMainOrder};
use util::*;

#[test]
fn dipmath_fig16() {
    judge! {
        "TUR: F aeg -> ion": AttackOutcome::<&MappedMainOrder>::Succeeds,
        "TUR: F gre supports F aeg -> ion",
        "AUS: F alb supports F aeg -> ion",
        "ITA: A tun -> gre": AttackOutcome::<&MappedMainOrder>::NoPath,
        "ITA: F ion convoys tun -> gre",
    };
}
