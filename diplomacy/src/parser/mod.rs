//! Contains error types and trait implementations for parsing diplomacy orders.
//!
//! All orders are expected to be in the following format:
//!
//! ```text
//! {Nation}: {UnitType} {Location} {Command}
//! ```
//!
//! # Supported Commands
//! 1. Hold: `hold` or `holds`
//! 1. Move: `-> {Destination}`
//! 1. Support: `supports {UnitType} {Region} [-> {Dest}]`
//! 1. Convoy: `convoys {Region} -> {Dest}`
//! 1. Build: `build` (this is non-idiomatic, but easier to parse)
//! 1. Disband: `disband`

use std::str::FromStr;

use crate::geo::Location;
use crate::order::{
    BuildCommand, Command, ConvoyedMove, MainCommand, MoveCommand, Order, RetreatCommand,
    SupportedOrder,
};
use crate::Nation;

mod error;

pub use self::error::{Error, ErrorKind};

/// A parser which operates on whitespace-delimited words from an input string.
pub trait FromWords: Sized {
    /// The associated error which can be returned from parsing.
    type Err;

    /// Performs the conversion.
    fn from_words(w: &[&str]) -> Result<Self, Self::Err>;
}

type ParseResult<T> = Result<T, Error>;

impl<L: Location + FromStr<Err = Error>, C: Command<L> + FromWords<Err = Error>> FromStr
    for Order<L, C>
{
    type Err = Error;

    fn from_str(s: &str) -> ParseResult<Self> {
        let words = s.split_whitespace().collect::<Vec<_>>();

        let nation = Nation::from(words[0].trim_end_matches(':'));
        let unit_type = words[1].parse()?;
        let location = words[2].parse()?;
        let cmd = C::from_words(&words[3..])?;

        Ok(Order {
            nation,
            unit_type,
            region: location,
            command: cmd,
        })
    }
}

impl<L: Location + FromStr<Err = Error>> FromWords for MainCommand<L> {
    type Err = Error;

    fn from_words(words: &[&str]) -> ParseResult<Self> {
        match &(words[0].to_lowercase())[..] {
            "holds" | "hold" => Ok(MainCommand::Hold),
            "->" => Ok(MoveCommand::from_words(&words[1..])?.into()),
            "supports" => Ok(SupportedOrder::from_words(&words[1..])?.into()),
            "convoys" => Ok(ConvoyedMove::from_words(&words[1..])?.into()),
            cmd => Err(Error::new(ErrorKind::UnknownCommand, cmd)),
        }
    }
}

impl<L: Location + FromStr<Err = Error>> FromWords for MoveCommand<L> {
    type Err = Error;

    fn from_words(w: &[&str]) -> ParseResult<Self> {
        const CONVOY_CASINGS: [&str; 2] = ["convoy", "Convoy"];

        match w.len() {
            1 => Ok(MoveCommand::new(w[0].parse()?)),
            3 if w[1] == "via" && CONVOY_CASINGS.contains(&w[2]) => {
                Ok(MoveCommand::with_mandatory_convoy(w[0].parse()?))
            }
            _ => Err(Error::new(ErrorKind::MalformedMove, w.join(" "))),
        }
    }
}

impl<L: Location + FromStr<Err = Error>> FromWords for SupportedOrder<L> {
    type Err = Error;

    fn from_words(w: &[&str]) -> ParseResult<Self> {
        match w.len() {
            // {unitType} {in}
            2 => Ok(SupportedOrder::Hold(w[0].parse()?, w[1].parse()?)),
            // {unitType} {from} -> {to}
            4 => Ok(SupportedOrder::Move(
                w[0].parse()?,
                w[1].parse()?,
                w[3].parse()?,
            )),
            _ => Err(Error::new(ErrorKind::MalformedSupport, w.join(" "))),
        }
    }
}

impl<L: Location + FromStr<Err = Error>> FromWords for ConvoyedMove<L> {
    type Err = Error;

    fn from_words(w: &[&str]) -> ParseResult<Self> {
        if w.len() == 3 {
            Ok(ConvoyedMove::new(w[0].parse()?, w[2].parse()?))
        } else {
            Err(Error::new(ErrorKind::MalformedConvoy, w.join(" ")))
        }
    }
}

impl<L: Location + FromStr<Err = Error>> FromWords for RetreatCommand<L> {
    type Err = Error;

    fn from_words(w: &[&str]) -> ParseResult<Self> {
        match &w[0].to_lowercase()[..] {
            "hold" | "holds" => Ok(RetreatCommand::Hold),
            "->" => Ok(RetreatCommand::Move(w[1].parse()?)),
            cmd => Err(Error::new(ErrorKind::UnknownCommand, cmd)),
        }
    }
}

impl FromWords for BuildCommand {
    type Err = Error;

    fn from_words(w: &[&str]) -> ParseResult<Self> {
        match &w[0].to_lowercase()[..] {
            "build" => Ok(BuildCommand::Build),
            "disband" => Ok(BuildCommand::Disband),
            cmd => Err(Error::new(ErrorKind::UnknownCommand, cmd)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geo::RegionKey;
    use crate::order::{MainCommand, Order};

    type OrderParseResult = Result<Order<RegionKey, MainCommand<RegionKey>>, Error>;

    #[test]
    fn hold() {
        let h_order: OrderParseResult = "AUS: F Tri hold".parse();
        println!("{}", h_order.unwrap());
    }

    #[test]
    fn army_move() {
        let m_order: OrderParseResult = "ENG: A Lon -> Bel".parse();
        println!("{}", m_order.unwrap());
    }

    #[test]
    fn army_move_via_convoy() {
        let m_order: OrderParseResult = "ENG: A Lon -> Bel via convoy".parse();
        let order = m_order.unwrap();
        assert_eq!(
            order.command.move_dest(),
            Some(&RegionKey::new("Bel", None))
        );

        let alt_casing: OrderParseResult = "ENG: A Lon -> Bel via Convoy".parse();
        assert_eq!(alt_casing.unwrap(), order);

        let no_pref: OrderParseResult = "ENG: A Lon -> Bel".parse();
        assert_ne!(no_pref.unwrap(), order);
    }
}
