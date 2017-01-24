use super::{MappedMainOrder};

pub trait Strength {
    /// Compute the strength of an action from its result.
    fn strength(&self) -> usize;
}

/// A collection of orders which support a specific order; used in strength calculations.
pub type Supporters<'a> = Vec<&'a MappedMainOrder>;

/// The intermediate state for a prevent strength calculation. Prevent strength
/// determines how much force is applied to stop any other units from entering the
/// destination province.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prevent<'a> {
    /// The preventing unit cannot reach its destination.
    NoPath,

    /// The order lost a head-to-head battle. It cannot prevent others from
    /// entering its destination.
    LostHeadToHead,

    /// The order attempts to prevent others from moving to the destination province with support.
    Prevents(&'a MappedMainOrder, Supporters<'a>),
}

impl<'a> Strength for Prevent<'a> {
    fn strength(&self) -> usize {
        use self::Prevent::*;
        match *self {
            NoPath | LostHeadToHead => 0,
            Prevents(_, ref sup) => 1 + sup.len(),
        }
    }
}

impl<T: Strength> Strength for Option<T> {
    fn strength(&self) -> usize {
        if let &Some(ref streng) = self {
            streng.strength()
        } else {
            0
        }
    }
}