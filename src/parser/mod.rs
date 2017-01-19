use geo::{Location, RegionKey};
use order::{Order, MainCommand, BuildCommand, SupportedOrder};
use ShortName;
use Nation;
use UnitType;

use std::default::Default;
use std::str::FromStr;

mod error;

pub use self::error::{Error, ErrorKind};

/// An order that has not yet been resolved against a world map.
pub type UnmappedOrder<C> = Order<RegionKey, C>;

type OrderResult<T> = Result<T, Error>;

impl ShortName for String {
    fn short_name(&self) -> String {
        self.clone()
    }
}

impl Location for String {}

impl<'a> ShortName for &'a str {
    fn short_name(&self) -> String {
        self.to_string()
    }
}

impl<'a> Location for &'a str {}

fn parse_shared(s: &str) -> OrderResult<(Nation, UnitType, RegionKey)> {
    let words = s.split_whitespace().collect::<Vec<_>>();
    let unit_type = words[1].parse().or(Err(Error::default()))?;
    let location = words[2].parse().or(Err(Error::default()))?;
    Ok((Nation(words[0].trim_right_matches(":").into()), unit_type, location))
}

impl FromStr for UnmappedOrder<MainCommand<RegionKey>> {
    type Err = Error;

    fn from_str(s: &str) -> OrderResult<Self> {
        let words = s.split_whitespace().collect::<Vec<_>>();
        if words[0] == "build" {
            unimplemented!()
        } else {
            let (nation, unit_type, location) = parse_shared(s)?;
            let cmd = match &(words[3].to_lowercase())[..] {
                "->" => Ok(MainCommand::Move(words[4].parse().or(Err(Error::default()))?)),
                "holds" | "hold" => Ok(MainCommand::Hold),
                "supports" => Ok(words_to_supp_comm(&words[4..])?.into()),
                "convoys" => unimplemented!(),
                _ => Err(Error {}),
            }?;

            Ok(Order {
                unit_type: unit_type,
                region: location,
                nation: nation,
                command: cmd,
            })
        }
    }
}

fn words_to_supp_comm(w: &[&str]) -> OrderResult<SupportedOrder<RegionKey>> {
    match w.len() {
        1 => Ok(SupportedOrder::Hold(w[0].parse().or(Err(Error::default()))?)),
        3 => Ok(SupportedOrder::Move(w[0].parse().or(Err(Error::default()))?, w[2].parse().or(Err(Error::default()))?)),
        _ => Err(Error::default())
    }
}

// impl FromStr for UnmappedOrder<RetreatCommand<String>> {
//     type Err = Error;

//     fn from_str(s: &str) -> OrderResult<Self> {
//         let (nation, unit_type, location) = parse_shared(s)?;
//         let words = s.split_whitespace().collect::<Vec<_>>();
//         let cmd = match &words[3].to_lowercase()[..] {
//             "hold" | "holds" => Ok(RetreatCommand::Hold),
//             "->" => Ok(RetreatCommand::Move(words[4].to_string())),
//             _ => Err(Error::default()),
//         }?;

//         Ok(Order::new(nation, unit_type, location, cmd))
//     }
// }

impl FromStr for UnmappedOrder<BuildCommand> {
    type Err = Error;

    fn from_str(s: &str) -> OrderResult<Self> {
        let (nation, unit_type, location) = parse_shared(s)?;
        let words = s.split_whitespace().collect::<Vec<_>>();
        let cmd = match &words[3].to_lowercase()[..] {
            "build" => Ok(BuildCommand::Build),
            "disband" => Ok(BuildCommand::Disband),
            _ => Err(Error::default()),
        }?;

        Ok(Order::new(nation, unit_type, location, cmd))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use order::MainCommand;
    use geo::RegionKey;

    type OrderParseResult = Result<UnmappedOrder<MainCommand<RegionKey>>, Error>;

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
}