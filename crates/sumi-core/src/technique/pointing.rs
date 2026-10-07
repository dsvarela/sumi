//! Pointing: when a digit's spots in a box all lie in one row or column, the
//! digit goes in that box, so the rest of the line can't have it.

use crate::digits::Digits;
use crate::grid::{Cell, House, col, row};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique};

pub struct Pointing;

impl Technique for Pointing {
    /// "Pointing triple" when there are three cells, which the hint wording
    /// decides from the step.
    fn name(&self) -> &'static str {
        "Pointing pair"
    }

    fn level(&self) -> Level {
        Level::Medium
    }

    fn score(&self) -> u32 {
        50
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for b in 0..9 {
            let house = House::Box(b);
            for digit in 1..=9 {
                let spots = p.spots(house, digit);
                if spots.len() < 2 {
                    continue;
                }
                for line in [House::Row(row(spots[0])), House::Col(col(spots[0]))] {
                    if !spots.iter().all(|&c| line.contains(c)) {
                        continue;
                    }
                    let outside: Vec<Cell> = line
                        .cells()
                        .into_iter()
                        .filter(|&c| !house.contains(c))
                        .collect();
                    let only: Digits = [digit].into_iter().collect();
                    steps.extend(Step::removing(
                        &Pointing,
                        p,
                        only,
                        vec![house, line],
                        spots.clone(),
                        outside.into_iter().map(|c| (c, only)),
                    ));
                }
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{POINTING_1, POINTING_2, check};

    #[test]
    fn removes_the_digit_from_the_rest_of_the_line() {
        check(&Pointing, &POINTING_1, "r3c7<>5");
        check(&Pointing, &POINTING_2, "r7c123789<>1");
    }
}
