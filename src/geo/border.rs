use geo::{Region, Terrain};

/// An undirected edge between two regions in a graph of the map. Units move
/// between regions via borders.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Border<'a>(&'a Region<'a>, &'a Region<'a>, Terrain);

impl<'a> Border<'a> {
    
    /// Create a new border between two regions.
    pub fn new(r1: &'a Region<'a>, r2: &'a Region<'a>, t: Terrain) -> Border<'a> {
        Border(r1, r2, t)
    }
    
    /// Returns true when either of the border's edges are `r`.
    pub fn contains<'b>(&self, r: &'a Region<'b>) -> bool {
        self.0 == r || self.1 == r
    }
    
    /// Returns true when the border contains both `r1` and `r2`.
    pub fn connects<'b>(&self, r1: &Region<'b>, r2: &Region<'b>) -> bool {
        self.contains(r1) && self.contains(r2)
    }
    
    /// If this region contains `r`, returns the other region in the border.
    pub fn dest_from(&self, r: &Region<'a>) -> Option<&'a Region<'a>> {
        if self.0 == r {
            Some(self.1)
        } else if self.1 == r {
            Some(self.0)
        } else {
            None
        }
    }
}