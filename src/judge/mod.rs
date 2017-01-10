mod resolver;
mod strength;
mod state_type;
mod calc;
mod prelude;
mod support;

pub use self::state_type::{
    OrderState,
    ResolutionState
};

use order::{Order, MainCommand};
use geo::Region;

pub type MappedMainOrder<'a> = Order<&'a Region<'a>, MainCommand<&'a Region<'a>>>;