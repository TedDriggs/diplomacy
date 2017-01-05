use ShortName;

/// A controllable area of the environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Province {
    pub full_name : String,
    pub short_name : String,
}

impl ShortName for Province {
    fn short_name(&self) -> String {
        self.short_name.clone()
    }
}