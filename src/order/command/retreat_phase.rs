use geo::Location;
use super::Command;

use std::fmt;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetreatCommand<L : Location> {
    Hold,
    Move(L)
}

impl<L : Location> Command<L> for RetreatCommand<L> {
    
}

impl<L : Location> fmt::Display for RetreatCommand<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &RetreatCommand::Hold => write!(f, "hold"),
            &RetreatCommand::Move(ref region) => write!(f, "-> {}", region.short_name()),
        }
    }
}

#[cfg(test)]
mod test {
    use geo::{Location, Region, Province, Terrain};
    use ShortName;
    use super::RetreatCommand;
    
    impl ShortName for String {
        fn short_name(&self) -> String {
            self.clone()
        }
    }
    
    impl Location for String {
        
    }
    
    #[test]
    fn with_string() {
        let retreat_to_string = RetreatCommand::Move(String::from("hey"));
        println!("{:?}", retreat_to_string);
    }
    
    #[test]
    fn with_region() {
        let prov = Province { full_name: "Hello World".to_string(), short_name: "hey".to_string() };
        let region = Region::new(&prov, None, Terrain::Land);
        let retreat_to_region = RetreatCommand::Move(&region);
        println!("{:?}", retreat_to_region);
    }
}