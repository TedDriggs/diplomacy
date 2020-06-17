use std::default::Default;
use crate::game::{Config, Turn};

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