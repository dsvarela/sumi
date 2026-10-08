//! Naked single: a cell with one candidate left.

use crate::grid::{House, box_of, col, row};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique};
use crate::words;

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

    fn definition(&self, _step: &Step) -> String {
        "A cell with only one candidate left.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let (cell, digit) = step.placement.expect("a naked single places a digit");
        Explanation {
            area: format!("{}: its row, column and box.", words::cell(cell)),
            area_houses: step.houses.clone(),
            effect: format!("Place {digit} in {}.", words::cell(cell)),
            reason: "Every other digit is ruled out in it.".into(),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        p.empty_cells()
            .filter_map(|cell| {
                let digit = p.candidates(cell).only()?;
                // The digits in its row, column and box together leave one,
                // so all three are its houses.
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
