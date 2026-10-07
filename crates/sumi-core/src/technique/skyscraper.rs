//! Skyscraper: two rows where a digit has only two spots each, with one spot
//! of each in the same column (the base). One of the other two spots (the
//! roof) holds the digit, so no cell that sees both roof cells can.
//! Likewise with rows and columns swapped.

use crate::digits::Digits;
use crate::grid::{Cell, House, col, row};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, choose, seen_by_all, strong_links};

pub struct Skyscraper;

impl Technique for Skyscraper {
    fn name(&self) -> &'static str {
        "Skyscraper"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        130
    }

    fn all(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for digit in 1..=9 {
            let only: Digits = [digit].into_iter().collect();
            for (lines, across, base_house) in [
                (
                    &House::ALL[9..18],
                    col as fn(Cell) -> usize,
                    House::Col as fn(usize) -> House,
                ),
                (&House::ALL[18..], row, House::Row),
            ] {
                let links = strong_links(p, digit, lines);
                for pair in choose(&links, 2) {
                    let [(a1, a2, line_a), (b1, b2, line_b)] = pair[..] else {
                        unreachable!()
                    };
                    for (base_a, roof_a) in [(a1, a2), (a2, a1)] {
                        for (base_b, roof_b) in [(b1, b2), (b2, b1)] {
                            // Roofs in one line too would be an X-Wing.
                            if across(base_a) != across(base_b) || across(roof_a) == across(roof_b)
                            {
                                continue;
                            }
                            let cells = vec![base_a, roof_a, base_b, roof_b];
                            let targets: Vec<Cell> = seen_by_all(&[roof_a, roof_b])
                                .filter(|c| !cells.contains(c))
                                .collect();
                            steps.extend(Step::removing(
                                &Skyscraper,
                                p,
                                only,
                                vec![line_a, line_b, base_house(across(base_a))],
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
    use crate::grid::{CELLS, Grid};
    use crate::technique::Fish;
    use crate::technique::examples::{SKYSCRAPER_1, SKYSCRAPER_2, check};

    /// 1 has two spots in rows 1 and 5, both in columns 1 and 9: an X-Wing,
    /// which is not a Skyscraper though its bases line up.
    #[test]
    fn leaves_an_x_wing_alone() {
        let mut candidates = [Digits::ALL; CELLS];
        for r in [0, 4] {
            for c in 1..8 {
                candidates[r * 9 + c].remove(1);
            }
        }
        let p = Position::with_candidates(&Grid::EMPTY, candidates);
        assert!(!Fish::<2>.all(&p).is_empty());
        assert!(Skyscraper.all(&p).is_empty());
    }

    #[test]
    fn removes_the_digit_from_cells_seeing_both_roofs() {
        check(&Skyscraper, &SKYSCRAPER_1, "r1c78,r3c45<>1");
        check(&Skyscraper, &SKYSCRAPER_2, "r1c4<>4");
    }
}
