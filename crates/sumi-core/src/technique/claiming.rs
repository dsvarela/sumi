//! Claiming: a digit's spots in a row or column all lie in one box. The
//! line's copy of the digit is then in that box, so the rest of the box
//! can't have it.

use crate::digits::Digits;
use crate::grid::{Cell, House, box_of};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique};
use crate::words;

pub struct Claiming;

impl Technique for Claiming {
    fn name(&self) -> &'static str {
        "Claiming"
    }

    fn level(&self) -> Level {
        Level::Medium
    }

    fn score(&self) -> u32 {
        50
    }

    fn definition(&self, _step: &Step) -> String {
        "In a row or column, all spots for a digit lie in one box.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let digit = step.digits.only().expect("claiming is about one digit");
        let [line, house] = step.houses[..] else {
            panic!("claiming has a line and a box")
        };
        let (line, house) = (words::house(line), words::house(house));
        Explanation {
            area: format!("On digit {digit}, {line} and {house}."),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "{} must put its {digit} in {house}, so the rest of {house} can't.",
                words::capital(&line)
            ),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for line in &House::ALL[9..] {
            let line = *line;
            for digit in 1..=9 {
                let spots = p.spots(line, digit);
                if spots.len() < 2 {
                    continue;
                }
                let house = House::Box(box_of(spots[0]));
                if !spots.iter().all(|&c| house.contains(c)) {
                    continue;
                }
                let outside: Vec<Cell> = house
                    .cells()
                    .into_iter()
                    .filter(|&c| !line.contains(c))
                    .collect();
                let only: Digits = [digit].into_iter().collect();
                steps.extend(Step::removing(
                    &Claiming,
                    p,
                    only,
                    vec![line, house],
                    spots,
                    outside.into_iter().map(|c| (c, only)),
                ));
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{CLAIMING_1, CLAIMING_2, check};

    #[test]
    fn removes_the_digit_from_the_rest_of_the_box() {
        check(&Claiming, &CLAIMING_1, "r3c2<>7");
        check(&Claiming, &CLAIMING_2, "r1c45,r2c45,r3c45<>4");
    }
}
