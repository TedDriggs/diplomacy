use crate::game::{Config, Turn};
use std::default::Default;

#[derive(Clone)]
pub struct World {
    config: Config,
    history: Vec<Turn>,
}

impl Default for World {
    fn default() -> Self {
        World {
            config: Config::standard(),
            history: vec![],
        }
    }
}
