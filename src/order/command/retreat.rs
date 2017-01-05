use geo::Region;

/// Valid commands for the retreat phase of a turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RetreatCommand<'a> {
    Hold,
    Move(&'a Region<'a>)
}