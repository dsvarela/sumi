//! W-Wing: two cells with the same two candidates x and y that don't see
//! each other, joined by a digit x whose only two spots in a house each see
//! one of them. One of the two cells must then be y, so no cell that sees
//! both can be.

use crate::digits::Digits;
use crate::grid::{Cell, House, sees};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique, bivalue_cells, choose, seen_by_all, strong_links};
use crate::words;

pub struct WWing;

impl Technique for WWing {
    fn name(&self) -> &'static str {
        "W-Wing"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        150
    }

    fn definition(&self, _step: &Step) -> String {
        "Two matching 2-candidate cells, joined by a digit's 2 spots.".into()
    }

    /// The key cells are the two wings, then the two ends of the link.
    fn explain(&self, step: &Step) -> Explanation {
        let [a, b, e1, e2] = step.key_cells[..] else {
            panic!("a W-Wing names its wings and the ends of its link")
        };
        let removed = step.removals[0].1;
        let link = step
            .digits
            .without(removed)
            .only()
            .expect("a W-Wing links one digit");
        let other = removed.only().expect("a W-Wing removes one digit");
        let house = words::house(step.houses[0]);
        Explanation {
            area: format!("Digits {link} and {other}; the link is {link} in {house}."),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "{} puts its {link} in {} or {}, so {} or {} must be {other}.",
                words::capital(&house),
                words::cell(e1),
                words::cell(e2),
                words::cell(a),
                words::cell(b),
            ),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for pair in choose(&bivalue_cells(p), 2) {
            let [a, b] = pair[..] else { unreachable!() };
            let digits = p.candidates(a);
            if p.candidates(b) != digits || sees(a, b) {
                continue;
            }
            for x in digits.iter() {
                let y: Digits = digits.without([x].into_iter().collect());
                for (e1, e2, house) in strong_links(p, x, &House::ALL) {
                    if !(sees(e1, a) && sees(e2, b) || sees(e1, b) && sees(e2, a)) {
                        continue;
                    }
                    let targets: Vec<Cell> = seen_by_all(&[a, b]).collect();
                    steps.extend(
                        Step::removing(
                            &WWing,
                            p,
                            digits,
                            vec![house],
                            vec![a, b, e1, e2],
                            targets.into_iter().map(|c| (c, y)),
                        )
                        .map(|step| step.keyed(vec![a, b, e1, e2])),
                    );
                }
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{W_WING_1, W_WING_2, check};

    #[test]
    fn removes_the_other_digit_where_both_wings_are_seen() {
        check(&WWing, &W_WING_1, "r4c9<>5");
        check(&WWing, &W_WING_2, "r123c7,r89c9<>4");
    }
}
