pub trait Strength {
    /// Compute the strength of an action from its result.
    fn strength(&self) -> usize;
}

/// A collection of orders which support a specific order; used in strength calculations.
pub(crate) type Supporters<O> = Vec<O>;

/// The intermediate state for a prevent strength calculation. Prevent strength
/// determines how much force is applied to stop any other units from entering the
/// destination province.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Prevent<O> {
    /// The preventing unit cannot reach its destination.
    NoPath,

    /// The order lost a head-to-head battle. It cannot prevent others from
    /// entering its destination.
    LostHeadToHead,

    /// The order attempts to prevent others from moving to the destination province with support.
    Prevents(O, Supporters<O>),
}

impl<O: std::fmt::Debug + Copy> Prevent<O> {
    pub fn unwrap_order(&self) -> O {
        let Self::Prevents(order, _) = self else {
            panic!("Attempted to unwrap {:?} value", self);
        };

        *order
    }
}

impl<O> Strength for Prevent<O> {
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
        self.as_ref().map(Strength::strength).unwrap_or_default()
    }
}
