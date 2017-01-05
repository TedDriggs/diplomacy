/// Valid orders during build seasons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildCommand {
    Build,
    Disband,
}