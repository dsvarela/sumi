//! Full house: the last empty cell of a row, column or box takes the one
//! digit the others don't hold.

use crate::digits::Digits;
use crate::grid::{Cell, House};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique};
use crate::words;

pub struct FullHouse;

impl Technique for FullHouse {
    fn name(&self) -> &'static str {
        "Full house"
    }

    fn level(&self) -> Level {
        Level::Easy
    }

    fn score(&self) -> u32 {
        4
    }

    fn definition(&self, _step: &Step) -> String {
        "A row, column or box with one empty cell left.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let (cell, digit) = step.placement.expect("a full house places a digit");
        let house = words::house(step.houses[0]);
        Explanation {
            area: format!("{}.", words::capital(&house)),
            area_houses: step.houses.clone(),
            effect: format!("Place {digit} in {}.", words::cell(cell)),
            reason: format!("It is the only digit {house} lacks."),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for house in House::ALL {
            let cells = house.cells();
            let empty: Vec<Cell> = cells
                .into_iter()
                .filter(|&c| p.grid().get(c).is_none())
                .collect();
            let [cell] = empty[..] else { continue };
            let placed: Digits = cells.iter().filter_map(|&c| p.grid().get(c)).collect();
            if let Some(digit) = Digits::ALL.without(placed).only()
                && p.candidates(cell).contains(digit)
            {
                steps.push(Step::placing(&FullHouse, cell, digit, vec![house]));
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{FULL_HOUSE_1, check};

    #[test]
    fn places_the_last_digit_of_a_house() {
        check(&FullHouse, &FULL_HOUSE_1, "r9c6=6");
    }
}
