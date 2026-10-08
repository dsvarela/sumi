//! Unique rectangle, type 1: four empty cells at the corners of a rectangle
//! spanning two rows, two columns and two boxes, three of them with only the
//! candidates a and b. If the fourth were a or b too, a and b could swap
//! around the rectangle and the puzzle would have two solutions. So in a
//! puzzle with one solution, the fourth cell is neither.

use crate::grid::{Cell, House, box_of};
use crate::position::Position;
use crate::step::{Explanation, Step};
use crate::technique::{Level, Technique, choose};
use crate::words;

pub struct UniqueRectangle;

impl Technique for UniqueRectangle {
    fn name(&self) -> &'static str {
        "Unique rectangle"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        100
    }

    fn definition(&self, _step: &Step) -> String {
        "One solution means two digits can't swap in a rectangle.".into()
    }

    fn explain(&self, step: &Step) -> Explanation {
        let digits = words::digits(step.digits);
        Explanation {
            area: format!("Digits {digits}, {}.", words::houses(&step.houses)),
            area_houses: step.houses.clone(),
            effect: words::removing(&step.removals),
            reason: format!(
                "Otherwise {digits} could swap in the other corners, giving two solutions."
            ),
            second_group: Vec::new(),
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        if !p.is_unique() {
            return Vec::new();
        }
        let lines: Vec<usize> = (0..9).collect();
        let mut steps = Vec::new();
        for rows in choose(&lines, 2) {
            for cols in choose(&lines, 2) {
                let corners: Vec<Cell> = rows
                    .iter()
                    .flat_map(|&r| cols.iter().map(move |&c| r * 9 + c))
                    .collect();
                let mut boxes: Vec<usize> = corners.iter().map(|&c| box_of(c)).collect();
                boxes.sort_unstable();
                boxes.dedup();
                if boxes.len() != 2 || corners.iter().any(|&c| p.grid().get(c).is_some()) {
                    continue;
                }
                for (i, &odd) in corners.iter().enumerate() {
                    let ab = p.candidates(corners[(i + 1) % 4]);
                    let others = corners.iter().filter(|&&c| c != odd);
                    if ab.len() != 2 || others.clone().any(|&c| p.candidates(c) != ab) {
                        continue;
                    }
                    // Four corners of just a and b are the deadly pattern
                    // itself: the puzzle has no single solution to rely on.
                    let fourth = p.candidates(odd);
                    if fourth.intersection(ab) != ab || fourth == ab {
                        continue;
                    }
                    let houses: Vec<House> = rows
                        .iter()
                        .map(|&r| House::Row(r))
                        .chain(cols.iter().map(|&c| House::Col(c)))
                        .collect();
                    steps.extend(Step::removing(
                        &UniqueRectangle,
                        p,
                        ab,
                        houses,
                        corners.clone(),
                        [(odd, ab)],
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
    use crate::digits::Digits;
    use crate::grid::{CELLS, Grid};
    use crate::technique::examples::{UNIQUE_RECTANGLE_1, UNIQUE_RECTANGLE_2, check_position};

    #[test]
    fn removes_both_digits_from_the_fourth_corner() {
        let p = UNIQUE_RECTANGLE_1.position().known_unique();
        check_position(&UniqueRectangle, &p, "r2c2<>8, r2c2<>9");
        let p = UNIQUE_RECTANGLE_2.position().known_unique();
        check_position(&UniqueRectangle, &p, "r9c6<>2, r9c6<>3");
    }

    /// r1c1, r1c4 and r2c1 hold only 1 and 2, and r2c4 has 3 as well. On a
    /// rectangle over two boxes that rules 1 and 2 out of r2c4; moved to rows
    /// 1 and 4 it spans four boxes, where 1 and 2 can't swap, so it proves
    /// nothing.
    #[test]
    fn needs_a_rectangle_over_exactly_two_boxes() {
        let rectangle = |rows: [usize; 2]| {
            let mut candidates = [Digits::ALL; CELLS];
            for r in rows {
                for c in [0, 3] {
                    candidates[r * 9 + c] = [1, 2].into_iter().collect();
                }
            }
            candidates[rows[1] * 9 + 3] = [1, 2, 3].into_iter().collect();
            Position::with_candidates(&Grid::EMPTY, candidates).known_unique()
        };
        let found = UniqueRectangle.all(&rectangle([0, 1]));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].removals, [(12, [1, 2].into_iter().collect())]);
        assert!(UniqueRectangle.all(&rectangle([0, 3])).is_empty());

        let mut candidates = [Digits::ALL; CELLS];
        for cell in [0, 3, 9, 12] {
            candidates[cell] = [1, 2].into_iter().collect();
        }
        let deadly = Position::with_candidates(&Grid::EMPTY, candidates).known_unique();
        assert!(UniqueRectangle.all(&deadly).is_empty());
    }

    #[test]
    fn finds_nothing_unless_the_solution_is_known_to_be_unique() {
        assert!(
            UniqueRectangle
                .all(&UNIQUE_RECTANGLE_1.position())
                .is_empty()
        );
    }
}
