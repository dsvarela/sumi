//! Y-Wing: a cell with candidates x and y (the pivot) that sees one cell
//! with x and z and another with y and z (the wings). Whichever the pivot
//! holds, one wing is z, so no cell that sees both wings can be. Also called
//! XY-Wing.

use crate::grid::{Cell, sees};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique, bivalue_cells, removed_cells, seen_by_all, shared_house};
use crate::words;

pub struct YWing;

impl Technique for YWing {
    fn name(&self) -> &'static str {
        "Y-Wing"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        160
    }

    fn definition(&self, _step: &Step) -> String {
        "A cell with 2 candidates sees two cells that share a 3rd.".into()
    }

    /// The key cells are the pivot, then its two wings.
    fn explain(&self, step: &Step) -> Explanation {
        let [pivot, a, b] = step.key_cells[..] else {
            panic!("a Y-Wing names its pivot and wings")
        };
        let z = step.removals[0]
            .1
            .only()
            .expect("a Y-Wing removes one digit");
        Explanation {
            area: format!("Middle cell {}; digit {z}.", words::cell(pivot)),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "Whatever {} is, {} or {} must be {z}, and {}.",
                words::cell(pivot),
                words::cell(a),
                words::cell(b),
                words::see_all(&removed_cells(step), "both")
            ),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let bivalue = bivalue_cells(p);
        let mut steps = Vec::new();
        for &pivot in &bivalue {
            let xy = p.candidates(pivot);
            let wings: Vec<Cell> = bivalue
                .iter()
                .copied()
                .filter(|&w| sees(pivot, w))
                .collect();
            for &a in &wings {
                for &b in &wings {
                    let (xz, yz) = (p.candidates(a), p.candidates(b));
                    // a shares x with the pivot and b shares y; each has z.
                    let z = xz.intersection(yz);
                    if a >= b
                        || z.len() != 1
                        || xz.intersection(xy).len() != 1
                        || yz.intersection(xy).len() != 1
                        || xz.union(yz).union(xy).len() != 3
                    {
                        continue;
                    }
                    let targets: Vec<Cell> = seen_by_all(&[a, b]).filter(|&c| c != pivot).collect();
                    steps.extend(
                        Step::removing(
                            &YWing,
                            p,
                            xy.union(z),
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
    use crate::technique::examples::{Y_WING_1, Y_WING_2, check};

    #[test]
    fn removes_z_where_both_wings_are_seen() {
        check(&YWing, &Y_WING_1, "r2c6<>2");
        check(&YWing, &Y_WING_2, "r4c23,r5c456<>9");
    }
}
