use geo::Region;
use std::convert::From;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderState {
    Succeeds,
    Fails,
}

impl From<bool> for OrderState {
    fn from(b: bool) -> Self {
        match b {
            true => OrderState::Succeeds,
            false => OrderState::Fails,
        }
    }
}

impl From<OrderState> for bool {
    fn from(os: OrderState) -> Self {
        match os {
            OrderState::Succeeds => true,
            OrderState::Fails => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionState {
    Guessing(OrderState),
    Known(OrderState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvinceOutcome<'a> {
    Holds,
    Moves,
    DislodgedBy(&'a Region<'a>),
}