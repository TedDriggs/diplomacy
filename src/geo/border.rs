use geo::{RegionKey, Terrain};

/// An undirected edge between two regions in a graph of the map. Units move
/// between regions via borders.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Border(RegionKey, RegionKey, Terrain);

impl Border {
    
    /// Create a new border between two regions.
    pub fn new(r1: RegionKey, r2: RegionKey, t: Terrain) -> Border {
        Border(r1, r2, t)
    }
    
    pub fn terrain(&self) -> &Terrain {
        &self.2
    }
    
    pub fn sides<'a>(&'a self) -> (&'a RegionKey, &'a RegionKey) {
        (&self.0, &self.1)
    }
    
    /// Returns true when either of the border's edges are `r`.
    pub fn contains<'a, PE: PartialEq<&'a RegionKey>>(&'a self, r: PE) -> bool {
        r == &self.0 || r == &self.1
    }
    
    /// Returns true when the border contains both `r1` and `r2`.
    pub fn connects<'a, IR1: PartialEq<&'a RegionKey>, IR2: PartialEq<&'a RegionKey>>(&'a self, r1: IR1, r2: IR2) -> bool {
        self.contains(r1) && self.contains(r2)
    }
    
    /// If this region contains `r`, returns the other region in the border.
    pub fn dest_from<'a, IR: PartialEq<RegionKey>>(&self, r: &IR) -> Option<&RegionKey> {
        if r == &self.0 {
            Some(&self.1)
        } else if r == &self.1 {
            Some(&self.0)
        } else {
            None
        }
    }
}