//! XYZ-Wing: a cell with candidates x, y and z (the pivot) that sees one
//! cell with x and z and another with y and z. One of the three is z, so no
//! cell that sees all three can be.

use crate::grid::{Cell, sees};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique, bivalue_cells, removed_cells, seen_by_all, shared_house};
use crate::words;

pub struct XyzWing;

impl Technique for XyzWing {
    fn name(&self) -> &'static str {
        "XYZ-Wing"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        180
    }

    fn definition(&self, _step: &Step) -> String {
        "A Y-Wing whose middle cell also holds the shared digit.".into()
    }

    /// The key cells are the pivot, then its two wings.
    fn explain(&self, step: &Step) -> Explanation {
        let [pivot, a, b] = step.key_cells[..] else {
            panic!("an XYZ-Wing names its pivot and wings")
        };
        let z = step.removals[0]
            .1
            .only()
            .expect("an XYZ-Wing removes one digit");
        let three = words::list(&[pivot, a, b].map(words::cell));
        Explanation {
            area: format!("Middle cell {}; digit {z}.", words::cell(pivot)),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "One of {three} must be {z}, and {}.",
                words::see_all(&removed_cells(step), "all three")
            ),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let bivalue = bivalue_cells(p);
        let mut steps = Vec::new();
        for pivot in p.empty_cells() {
            let xyz = p.candidates(pivot);
            if xyz.len() != 3 {
                continue;
            }
            // Wings: cells that see the pivot and have two of its digits.
            let wings: Vec<Cell> = bivalue
                .iter()
                .copied()
                .filter(|&w| sees(pivot, w) && p.candidates(w).without(xyz).is_empty())
                .collect();
            for &a in &wings {
                for &b in &wings {
                    let z = p.candidates(a).intersection(p.candidates(b));
                    if a >= b || z.len() != 1 {
                        continue;
                    }
                    let targets: Vec<Cell> = seen_by_all(&[pivot, a, b]).collect();
                    steps.extend(
                        Step::removing(
                            &XyzWing,
                            p,
                            xyz,
                            vec![shared_house(pivot, a), shared_house(pivot, b)],
                            vec![pivot, a, b],
                            targets.into_iter().map(|c| (c, z)),
                        )
                        .map(|step| step.keyed(vec![pivot, a, b])),
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
    use crate::technique::examples::{XYZ_WING_1, XYZ_WING_2, check};

    #[test]
    fn removes_z_where_all_three_are_seen() {
        check(&XyzWing, &XYZ_WING_1, "r9c2<>7");
        check(&XyzWing, &XYZ_WING_2, "r3c56<>6");
    }
}
