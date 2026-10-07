//! Naked single: a cell with one candidate left.

use crate::grid::{House, box_of, col, row};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique};

pub struct NakedSingle;

impl Technique for NakedSingle {
    fn name(&self) -> &'static str {
        "Naked single"
    }

    fn level(&self) -> Level {
        Level::Easy
    }

    fn score(&self) -> u32 {
        4
    }

    fn all(&self, p: &Position) -> Vec<Step> {
        p.empty_cells()
            .filter_map(|cell| {
                let digit = p.candidates(cell).only()?;
                // Every digit the cell sees rules out a candidate, so its whole
                // row, column and box are the area.
                let houses = vec![
                    House::Box(box_of(cell)),
                    House::Row(row(cell)),
                    House::Col(col(cell)),
                ];
                Some(Step::placing(&NakedSingle, cell, digit, houses))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{NAKED_SINGLE_1, check};

    #[test]
    fn places_the_only_candidate() {
        check(&NakedSingle, &NAKED_SINGLE_1, "r6c7=6");
    }
}
