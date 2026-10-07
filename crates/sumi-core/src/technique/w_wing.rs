//! W-Wing: two cells with the same two candidates x and y that don't see
//! each other, joined by a digit x whose only two spots in a house each see
//! one of them. One of the two cells must then be y, so no cell that sees
//! both can be.

use crate::digits::Digits;
use crate::grid::{Cell, House, sees};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, bivalue_cells, choose, seen_by_all, strong_links};

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
                    steps.extend(Step::removing(
                        &WWing,
                        p,
                        digits,
                        vec![house],
                        vec![a, b, e1, e2],
                        targets.into_iter().map(|c| (c, y)),
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
    use crate::technique::examples::{W_WING_1, W_WING_2, check};

    #[test]
    fn removes_the_other_digit_where_both_wings_are_seen() {
        check(&WWing, &W_WING_1, "r4c9<>5");
        check(&WWing, &W_WING_2, "r123c7,r89c9<>4");
    }
}
