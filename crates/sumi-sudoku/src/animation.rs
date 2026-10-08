//! Short animations: a row, column or box the player completes pulses once,
//! and a wave crosses the board at a win. Each is a function of the time
//! since it started, so drawing stays a function of the view, and every
//! cell lights once: nothing flashes.

use std::time::Duration;

use sumi_core::grid::{Cell, Grid, House, col, row};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    House(House),
    Win,
}

/// How strongly a cell is lit at a moment of an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Light {
    Strong,
    Faint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Animation {
    pub kind: Kind,
    pub elapsed: Duration,
}

/// A completed house is lit strongly for half of this, then faintly.
const HOUSE: Duration = Duration::from_millis(300);
/// The wave takes this long to cross the board's 17 diagonals.
const WIN: Duration = Duration::from_millis(850);
const DIAGONALS: u32 = 17;

impl Animation {
    pub fn new(kind: Kind) -> Animation {
        Animation {
            kind,
            elapsed: Duration::ZERO,
        }
    }

    pub fn done(&self) -> bool {
        self.elapsed
            >= match self.kind {
                Kind::House(_) => HOUSE,
                Kind::Win => WIN,
            }
    }

    /// How `cell` is lit now, if at all.
    pub fn light(&self, cell: Cell) -> Option<Light> {
        if self.done() {
            return None;
        }
        match self.kind {
            Kind::House(house) if house.cells().contains(&cell) => {
                Some(if self.elapsed < HOUSE / 2 {
                    Light::Strong
                } else {
                    Light::Faint
                })
            }
            Kind::House(_) => None,
            // The wave runs from the top left corner to the bottom right,
            // one diagonal at a time, with a faint diagonal behind it.
            Kind::Win => {
                let front =
                    (self.elapsed.as_millis() * u128::from(DIAGONALS) / WIN.as_millis()) as usize;
                let diagonal = row(cell) + col(cell);
                if diagonal == front {
                    Some(Light::Strong)
                } else if diagonal + 1 == front {
                    Some(Light::Faint)
                } else {
                    None
                }
            }
        }
    }
}

/// The houses whose nine cells hold nine different digits.
pub fn complete_houses(board: &Grid) -> [bool; 27] {
    House::ALL.map(|house| {
        let mut seen = 0u16;
        for cell in house.cells() {
            match board.get(cell) {
                Some(digit) => seen |= 1 << digit,
                None => return false,
            }
        }
        seen.count_ones() == 9
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(kind: Kind, ms: u64) -> Animation {
        Animation {
            kind,
            elapsed: Duration::from_millis(ms),
        }
    }

    #[test]
    fn a_house_pulses_strong_then_faint_then_stops() {
        let row = Kind::House(House::Row(2));
        assert_eq!(at(row, 0).light(20), Some(Light::Strong));
        assert_eq!(at(row, 200).light(20), Some(Light::Faint));
        assert_eq!(at(row, 300).light(20), None);
        assert!(at(row, 300).done());
        assert_eq!(at(row, 0).light(29), None, "outside the row");
    }

    /// Each cell is lit strongly once, on its diagonal's turn, and the
    /// wave reaches the last corner before it ends.
    #[test]
    fn the_wave_lights_each_cell_once_in_diagonal_order() {
        for cell in 0..81 {
            let strong: Vec<u64> = (0..850)
                .filter(|&ms| at(Kind::Win, ms).light(cell) == Some(Light::Strong))
                .collect();
            assert!(!strong.is_empty(), "{cell} never lit");
            let span = strong.last().unwrap() - strong.first().unwrap();
            assert_eq!(span as usize + 1, strong.len(), "{cell} lit twice");
        }
        let first = |cell| (0..850).find(|&ms| at(Kind::Win, ms).light(cell).is_some());
        assert!(first(0) < first(1));
        assert!(first(1) < first(80));
        assert!(at(Kind::Win, 850).done());
    }

    #[test]
    fn a_house_is_complete_with_nine_different_digits() {
        let mut board = Grid::EMPTY;
        for (i, cell) in House::Row(0).cells().into_iter().enumerate() {
            board.set(cell, Some(i as u8 + 1));
        }
        let complete = complete_houses(&board);
        assert!(complete[House::ALL.iter().position(|&h| h == House::Row(0)).unwrap()]);
        assert_eq!(complete.iter().filter(|&&c| c).count(), 1);
        board.set(8, Some(1)); // two 1s
        assert!(complete_houses(&board).iter().all(|&c| !c));
    }
}
