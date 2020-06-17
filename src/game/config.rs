use geo::{self, Map};
use game::{Phase, Season};

#[derive(Debug, Clone)]
pub struct Config {
    map: &'static Map,
    year_pattern: Vec<(Season, Phase)>,
    starting_year: usize,
}

impl Config {
    /// Gets the standard Diplomacy configuration.
    ///
    /// * Standard map
    /// * Spring and Fall seasons
    /// * Build phase after Fall retreat
    /// * Starting year 1901
    pub fn standard() -> Self {
        Config {
            map: geo::standard_map(),
            year_pattern: vec![
                (Season::Spring, Phase::Main),
                (Season::Spring, Phase::Retreat),
                (Season::Fall, Phase::Main),
                (Season::Fall, Phase::Retreat),
                (Season::Fall, Phase::Build),
            ],
            starting_year: 1901,
        }
    }
}