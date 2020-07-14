#[path = "./util.rs"]
mod util;

use util::*;

#[test]
fn dipmath_fig16() {
    judge! {
        "TUR: F aeg -> ion",
        "TUR: F gre supports F aeg -> ion",
        "AUS: F alb supports F aeg -> ion",
        "ITA: A tun -> gre",
        "ITA: F ion convoys tun -> gre",
    };
}
