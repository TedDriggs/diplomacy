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
    
    /// Returns true when either of the border's edges are `r`.
    pub fn contains<'a, IR: Into<&'a RegionKey>>(&self, r: IR) -> bool {
        let rk = r.into();
        &self.0 == rk || &self.1 == rk
    }
    
    /// Returns true when the border contains both `r1` and `r2`.
    pub fn connects<'a, 'b, IR1: Into<&'a RegionKey>, IR2: Into<&'b RegionKey>>(&self, r1: IR1, r2: IR2) -> bool {
        self.contains(r1) && self.contains(r2)
    }
    
    /// If this region contains `r`, returns the other region in the border.
    pub fn dest_from<'a, IR: Into<&'a RegionKey>>(&self, r: IR) -> Option<&RegionKey> {
        let rk = r.into();
        if &self.0 == rk {
            Some(&self.1)
        } else if &self.1 == rk {
            Some(&self.0)
        } else {
            None
        }
    }
}