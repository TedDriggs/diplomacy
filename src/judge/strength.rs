use super::{MappedMainOrder, Outcome};

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

/// The resistance a move order faces from its destination province.
/// If the unit attempts to HOLD, SUPPORT, or CONVOY then this will be
/// the HOLD STRENGTH for that unit. If the unit is attempting to move
/// to the province where the original move is coming from, a head-to-head
/// battle will occur.
#[derive(Debug, Clone)]
pub enum DestResistance<'a> {
    Holds(ProvinceHold<'a>),
    HeadToHead(Defend<'a>),
}

impl<'a> Strength for DestResistance<'a> {
    fn strength(&self) -> usize {
        match *self {
            DestResistance::Holds(ref h) => h.strength(),
            DestResistance::HeadToHead(ref d) => d.strength(),
        }

    }
}

impl<'a> From<ProvinceHold<'a>> for DestResistance<'a> {
    fn from(p: ProvinceHold<'a>) -> Self {
        DestResistance::Holds(p)
    }
}

impl<'a> From<Defend<'a>> for DestResistance<'a> {
    fn from(d: Defend<'a>) -> Self {
        DestResistance::HeadToHead(d)
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

/// Struct containing the intermediate data needed to compute
/// the success or failure of a MOVE command.
#[derive(Debug)]
pub struct MoveOutcome<'a> {
    atk: Attack<'a>,
    max_prevent: Option<Prevent<'a>>,
    dest_hold: Option<ProvinceHold<'a>>,
    h2h: Option<Defend<'a>>,
}

impl<'a> MoveOutcome<'a> {
    /// Create a new MoveOutcome.
    pub fn new<IP: Into<Option<Prevent<'a>>>,
               IH: Into<Option<ProvinceHold<'a>>>,
               ID: Into<Option<Defend<'a>>>>
        (atk: Attack<'a>,
         max_prevent: IP,
         dest_hold: IH,
         h2h: ID)
         -> Self {
        MoveOutcome {
            atk: atk,
            max_prevent: max_prevent.into(),
            dest_hold: dest_hold.into(),
            h2h: h2h.into(),
        }
    }

    /// A MOVE decision of a unit ordered to move results in 'moves' (success) when:
    /// The minimum of the ATTACK STRENGTH is larger than the maximum of the
    /// DEFEND STRENGTH of the opposing unit in case of a head to head battle
    /// or otherwise larger than the maximum of the HOLD STRENGTH of the attacked area.
    /// And in all cases the minimum of the ATTACK STRENGTH is larger than the maximum
    /// of the PREVENT STRENGTH of all of the units moving to the same area.
    /// [DATC 5.B.1](http://web.inter.nl.net/users/L.B.Kruijswijk/#5.B.1)
    pub fn is_successful(&self) -> bool {
        let atk_strength = self.atk.strength();
        let will_succeed = atk_strength > self.max_prevent.strength() &&
                           atk_strength > self.dest_hold.strength() &&
                           atk_strength > self.h2h.strength();

        will_succeed
    }
}

impl<'a> Outcome for MoveOutcome<'a> {}