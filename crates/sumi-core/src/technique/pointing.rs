//! Pointing: a digit's spots in a box all lie in one row or column. The
//! box's copy of the digit is then in that line, so the rest of the line
//! can't have it.

use crate::digits::Digits;
use crate::grid::{Cell, House, col, row};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique};
use crate::words;

pub struct Pointing;

impl Technique for Pointing {
    /// The name for two cells. With three it is a pointing triple, which
    /// only the step's cells tell apart.
    fn name(&self) -> &'static str {
        "Pointing pair"
    }

    fn level(&self) -> Level {
        Level::Medium
    }

    fn score(&self) -> u32 {
        50
    }

    fn title(&self, step: &Step) -> String {
        if step.cells.len() == 3 {
            "Pointing triple".into()
        } else {
            self.name().into()
        }
    }

    fn definition(&self, _step: &Step) -> String {
        "In a box, all spots for a digit lie in one row or column.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let digit = step.digits.only().expect("pointing is about one digit");
        let [house, line] = step.houses[..] else {
            panic!("pointing has a box and a line")
        };
        let (house, line) = (words::house(house), words::house(line));
        Explanation {
            area: format!("On digit {digit}, {house} and {line}."),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "{} must put its {digit} in {line}, so the rest of {line} can't.",
                words::capital(&house)
            ),
            second_group: Vec::new(),
        }
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
