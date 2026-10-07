//! Hidden single: a digit with only one spot left in a row, column or box.

use crate::grid::House;
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique};

pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn name(&self) -> &'static str {
        "Hidden single"
    }

    fn level(&self) -> Level {
        Level::Easy
    }

    fn all(&self, p: &Position) -> Vec<Step> {
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
