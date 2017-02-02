use game::Turn;
use geo::Map;

#[derive(Clone)]
pub struct World<'a> {
    map: &'a Map,
    history: Vec<Turn>,
}