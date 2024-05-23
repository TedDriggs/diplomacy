//! Types for representing the passage of in-game time.
//!
//! The calendar is used to determine what season and phase a game should enter
//! at the conclusion of a turn.

use crate::time::{Phase, Season, Time};
use std::collections::BTreeSet;

/// Identifier of a specific turn in a game.
pub type Month = (Season, Phase);

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    NoMonths,
    DuplicateMonth(Month),
    StartingMonthNotInCalendar,
}

/// The calendar dictates the sequence of turns in a game.
///
/// # Purpose
/// The calendar allows conversion between unsigned integers and game turns.
/// This allows a persistence layer to store turns as numbers while relying on `Calendar` to
/// determine which kinds of order should be accepted at any given time.
///
/// # Turns Without Orders
/// Some turns neither need nor accept orders, such as a retreat phase after a main phase in which
/// no units are dislodged. In these scenarios, the caller should still advance its turn counter to
/// indicate the completion of that turn.
pub struct Calendar {
    starting_year: usize,
    starting_month_index: usize,
    months: Vec<Month>,
}

impl Calendar {
    /// Create a new calendar starting at the specified time, with retreat phases automatically
    /// inserted after each main phase.
    pub fn new(start: Time, mut months: Vec<Month>) -> Result<Self, Error> {
        let mut insertion_points = months
            .iter()
            .enumerate()
            .filter(|(_, month)| month.1 == Phase::Main)
            .map(|(idx, month)| (idx + 1, month.0))
            .collect::<Vec<_>>();

        // Reverse the insertion points so that insertions don't invalidate indices of
        // subsequent insertions.
        insertion_points.reverse();

        for (idx, season) in insertion_points {
            months.insert(idx, (season, Phase::Retreat));
        }

        Calendar::with_explicit_retreats(start, months)
    }

    fn with_explicit_retreats(start: Time, months: Vec<Month>) -> Result<Self, Error> {
        if months.is_empty() {
            return Err(Error::NoMonths);
        }

        if let Some(first_duplicate) = find_first_duplicate(&months) {
            return Err(Error::DuplicateMonth(*first_duplicate));
        }

        let mut base = Self {
            starting_year: start.year(),
            starting_month_index: 0,
            months,
        };

        base.starting_month_index = base
            .position(&start)
            .ok_or(Error::StartingMonthNotInCalendar)?;

        Ok(base)
    }

    /// Create an infinite iterator that goes through all past and future `Time` values in the game.
    pub fn iter(&self) -> Iter {
        Iter {
            calendar: self,
            step: 0,
        }
    }

    /// Get the time associated with the `nth` turn of the game.
    pub fn nth(&self, turn: usize) -> Time {
        let month_idx = self.starting_month_index + turn;
        let year = self.starting_year + (month_idx / self.months.len());
        let (season, phase) = self.months[month_idx % self.months.len()];
        Time::new(season, year, phase)
    }

    pub fn position(&self, time: &Time) -> Option<usize> {
        let month_idx = self.month_position((time.season(), time.phase()))?;
        let years_passed = if time.year() < self.starting_year {
            return None;
        } else {
            time.year() - self.starting_year
        };

        if years_passed == 0 && month_idx < self.starting_month_index {
            return None;
        }

        Some((years_passed * self.months.len() + month_idx) - self.starting_month_index)
    }

    fn month_position(&self, month: Month) -> Option<usize> {
        self.months.iter().position(|m| *m == month)
    }
}

/// Time iterator for a calendar, producing all times in order starting with the first turn
/// of the game.
pub struct Iter<'a> {
    calendar: &'a Calendar,
    step: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Time;

    fn next(&mut self) -> Option<Time> {
        let time = self.calendar.nth(self.step);
        self.step += 1;
        Some(time)
    }
}

fn find_first_duplicate<T: Eq + Ord>(items: &[T]) -> Option<&T> {
    let mut seen = BTreeSet::new();
    items.iter().find(|item| !seen.insert(*item))
}

#[cfg(test)]
mod tests {
    use super::Calendar;
    use crate::time::{Phase::*, Season::*, Time};

    #[test]
    fn iter() {
        let calendar = Calendar::new(
            Time::new(Spring, 1901, Main),
            vec![(Spring, Main), (Fall, Main), (Winter, Build)],
        )
        .unwrap();

        let upcoming = calendar.iter().take(10).collect::<Vec<_>>();

        assert_eq!(
            upcoming,
            vec![
                Time::new(Spring, 1901, Main),
                Time::new(Spring, 1901, Retreat),
                Time::new(Fall, 1901, Main),
                Time::new(Fall, 1901, Retreat),
                Time::new(Winter, 1901, Build),
                Time::new(Spring, 1902, Main),
                Time::new(Spring, 1902, Retreat),
                Time::new(Fall, 1902, Main),
                Time::new(Fall, 1902, Retreat),
                Time::new(Winter, 1902, Build),
            ]
        )
    }

    #[test]
    fn iter_chaos_variant() {
        let calendar = Calendar::new(
            Time::new(Winter, 1900, Build),
            vec![(Spring, Main), (Fall, Main), (Winter, Build)],
        )
        .unwrap();

        let upcoming = calendar.iter().take(11).collect::<Vec<_>>();

        assert_eq!(
            upcoming,
            vec![
                Time::new(Winter, 1900, Build),
                Time::new(Spring, 1901, Main),
                Time::new(Spring, 1901, Retreat),
                Time::new(Fall, 1901, Main),
                Time::new(Fall, 1901, Retreat),
                Time::new(Winter, 1901, Build),
                Time::new(Spring, 1902, Main),
                Time::new(Spring, 1902, Retreat),
                Time::new(Fall, 1902, Main),
                Time::new(Fall, 1902, Retreat),
                Time::new(Winter, 1902, Build),
            ]
        )
    }

    #[test]
    fn position_month_before_start() {
        let calendar = Calendar::new(
            Time::new(Winter, 1900, Build),
            vec![(Spring, Main), (Fall, Main), (Winter, Build)],
        )
        .unwrap();

        assert_eq!(calendar.position(&Time::new(Spring, 1900, Main)), None);
    }
}
