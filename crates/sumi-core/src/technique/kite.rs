//! 2-String Kite: a row and a column where a digit has only two spots each,
//! with one spot of each in the same box. Those two can't both hold the
//! digit, so one of the far ends does, and no cell that sees both far ends
//! can.

use crate::digits::Digits;
use crate::grid::{Cell, House, box_of};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, seen_by_all, strong_links};

pub struct Kite;

impl Technique for Kite {
    fn name(&self) -> &'static str {
        "2-String Kite"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn all(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for digit in 1..=9 {
            let only: Digits = [digit].into_iter().collect();
            let rows = strong_links(p, digit, &House::ALL[9..18]);
            let cols = strong_links(p, digit, &House::ALL[18..]);
            for &(r1, r2, row) in &rows {
                for &(c1, c2, col) in &cols {
                    for (row_box, row_end) in [(r1, r2), (r2, r1)] {
                        for (col_box, col_end) in [(c1, c2), (c2, c1)] {
                            let b = box_of(row_box);
                            if row_box == col_box
                                || box_of(col_box) != b
                                || box_of(row_end) == b
                                || box_of(col_end) == b
                            {
                                continue;
                            }
                            let cells = vec![row_box, row_end, col_box, col_end];
                            let targets: Vec<Cell> = seen_by_all(&[row_end, col_end])
                                .filter(|c| !cells.contains(c))
                                .collect();
                            steps.extend(Step::removing(
                                &Kite,
                                p,
                                only,
                                vec![row, col, House::Box(b)],
                                cells,
                                targets.into_iter().map(|c| (c, only)),
                            ));
                        }
                    }
                }
            }
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{KITE_1, KITE_2, check};

    #[test]
    fn removes_the_digit_where_both_far_ends_are_seen() {
        check(&Kite, &KITE_1, "r2c4<>5");
        check(&Kite, &KITE_2, "r7c6<>9");
    }
}
