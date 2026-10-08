//! Hidden single: a digit with only one spot left in a row, column or box.

use crate::grid::House;
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique};
use crate::words;

pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn name(&self) -> &'static str {
        "Hidden single"
    }

    fn level(&self) -> Level {
        Level::Easy
    }

    fn score(&self) -> u32 {
        14
    }

    fn definition(&self, _step: &Step) -> String {
        "A digit with only one possible cell in a row, column or box.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let (cell, digit) = step.placement.expect("a hidden single places a digit");
        let house = words::house(step.houses[0]);
        Explanation {
            area: format!("On digit {digit}, {house}."),
            area_houses: step.houses.clone(),
            effect: format!("Place {digit} in {}.", words::cell(cell)),
            reason: format!("No other cell in {house} can hold {digit}."),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for house in House::ALL {
            for digit in 1..=9 {
                if let [cell] = p.spots(house, digit)[..] {
                    steps.push(Step::placing(&HiddenSingle, cell, digit, vec![house]));
                }
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{HIDDEN_SINGLE_1, HIDDEN_SINGLE_2, check};

    #[test]
    fn places_a_digit_with_one_spot_left() {
        check(&HiddenSingle, &HIDDEN_SINGLE_1, "r3c4=6");
        check(&HiddenSingle, &HIDDEN_SINGLE_2, "r6c4=3");
    }
}
