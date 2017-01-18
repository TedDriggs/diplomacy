use super::MappedMainOrder;

pub trait Strength {
    /// Compute the strength of an action from its result.
    fn strength(&self) -> usize;
}

/// A collection of orders which support a specific order; used in strength calculations.
pub type Supporters<'a> = Vec<&'a MappedMainOrder>;

/// Intermediate state of an attack order's strength calculation.
#[derive(Debug, Clone, PartialEq)]
pub enum Attack<'a> {
    /// The attacking unit cannot reach its destination.
    NoPath,

    /// The attack would have caused a friendly fire incident.
    FriendlyFire,

    /// The attack is moving into an unoccupied province.
    AgainstVacant(Supporters<'a>),

    /// The attack is entering a province that is being vacated.
    FollowingIn(Supporters<'a>),

    /// The attack is attempting to dislodge a unit in the province.
    AgainstOccupied(Supporters<'a>),
}

impl<'a> Strength for Attack<'a> {
    fn strength(&self) -> usize {
        use self::Attack::*;
        match *self {
            NoPath | FriendlyFire => 0,
            AgainstVacant(ref sup) |
            FollowingIn(ref sup) |
            AgainstOccupied(ref sup) => 1 + sup.len(),
        }
    }
}

/// The intermediate result of a province's hold strength calculation. The hold
/// strength determines how much force is needed to dislodge the unit from the province.
#[derive(Debug, Clone, PartialEq)]
pub enum ProvinceHold<'a> {
    /// No unit occupied the province.
    Empty,

    /// The unit in the province successfully moved elsewhere.
    SuccessfulExit,

    /// The unit in the province attempted to move elsewhere but did not.
    /// A failed exit cannot benefit from hold-support commands.
    FailedExit,

    /// The unit in the province did not attempt to move, so it can benefit from
    /// hold-support commands.
    UnitHolds(Supporters<'a>),
}

impl<'a> Strength for ProvinceHold<'a> {
    fn strength(&self) -> usize {
        use self::ProvinceHold::*;
        match *self {
            Empty | SuccessfulExit => 0,

            // A failed exit cannot benefit from hold-support commands,
            // so it always has strength 1.
            FailedExit => 1,

            UnitHolds(ref sup) => 1 + sup.len(),
        }
    }
}

/// Intermediate state for a defense strength calculation. Defense strength is the amount
/// of force applied in a head-to-head battle.
#[derive(Debug, Clone, PartialEq)]
pub struct Defend<'a>(pub Supporters<'a>);

impl<'a> Strength for Defend<'a> {
    fn strength(&self) -> usize {
        1 + self.0.len()
    }
}

/// The intermediate state for a prevent strength calculation. Prevent strength
/// determines how much force is applied to stop any other units from entering the
/// destination province.
#[derive(Debug, Clone, PartialEq)]
pub enum Prevent<'a> {
    /// The preventing unit cannot reach its destination.
    NoPath,

    /// The order lost a head-to-head battle. It cannot prevent others from
    /// entering its destination.
    LostHeadToHead,

    /// The order attempts to prevent others from moving to the destination province with support.
    Prevents(Supporters<'a>),
}

impl<'a> Strength for Prevent<'a> {
    fn strength(&self) -> usize {
        use self::Prevent::*;
        match *self {
            NoPath | LostHeadToHead => 0,
            Prevents(ref sup) => 1 + sup.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Resistance<'a> {
    Holds(ProvinceHold<'a>),
    HeadToHead(Defend<'a>),
}

impl<'a> Strength for Resistance<'a> {
    fn strength(&self) -> usize {
        match *self {
            Resistance::Holds(ref h) => h.strength(),
            Resistance::HeadToHead(ref d) => d.strength(),
        }

    }
}

impl<'a> From<ProvinceHold<'a>> for Resistance<'a> {
    fn from(p: ProvinceHold<'a>) -> Self {
        Resistance::Holds(p)
    }
}

impl<'a> From<Defend<'a>> for Resistance<'a> {
    fn from(d: Defend<'a>) -> Self {
        Resistance::HeadToHead(d)
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

#[derive(Debug)]
pub struct MoveOutcome<'a> {
    atk: Attack<'a>,
    max_prevent: Option<Prevent<'a>>,
    resistance: Resistance<'a>,
}

impl<'a> MoveOutcome<'a> {
    pub fn new<IP: Into<Option<Prevent<'a>>>, IR: Into<Resistance<'a>>>(atk: Attack<'a>,
                                                                        max_prevent: IP,
                                                                        resistance: IR)
                                                                        -> Self {
        MoveOutcome {
            atk: atk,
            max_prevent: max_prevent.into(),
            resistance: resistance.into(),
        }
    }

    /// Gets whether or not the move succeeds
    pub fn is_successful(&self) -> bool {
        let atk_strength = self.atk.strength();
        let will_succeed = atk_strength > self.max_prevent.strength() &&
                           atk_strength > self.resistance.strength();
        if !will_succeed {
            println!("{:?}", self);
        }

        will_succeed
    }
}