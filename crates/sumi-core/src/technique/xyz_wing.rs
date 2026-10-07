//! XYZ-Wing: a cell with candidates x, y and z (the pivot) that sees one
//! cell with x and z and another with y and z. One of the three is z, so no
//! cell that sees all three can be.

use crate::grid::{Cell, sees};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, bivalue_cells, seen_by_all, shared_house};

pub struct XyzWing;

impl Technique for XyzWing {
    fn name(&self) -> &'static str {
        "XYZ-Wing"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn all(&self, p: &Position) -> Vec<Step> {
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
                    steps.extend(Step::removing(
                        &XyzWing,
                        p,
                        xyz,
                        vec![shared_house(pivot, a), shared_house(pivot, b)],
                        vec![pivot, a, b],
                        targets.into_iter().map(|c| (c, z)),
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
    use crate::technique::examples::{XYZ_WING_1, XYZ_WING_2, check};

    #[test]
    fn removes_z_where_all_three_are_seen() {
        check(&XyzWing, &XYZ_WING_1, "r9c2<>7");
        check(&XyzWing, &XYZ_WING_2, "r3c56<>6");
    }
}
