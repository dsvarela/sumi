//! Naked and hidden subsets of size `N`: pairs, triples and quads.
//!
//! Naked: `N` cells in a row, column or box whose candidates together are
//! only `N` digits. Those digits go in those cells, so the rest of the house
//! can't have them.
//!
//! Hidden: `N` digits whose spots in a house are only `N` cells. Those cells
//! hold those digits, so their other candidates go.

use crate::digits::Digits;
use crate::grid::{Cell, House};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, choose};

pub struct NakedSubset<const N: usize>;

pub struct HiddenSubset<const N: usize>;

impl<const N: usize> Technique for NakedSubset<N> {
    fn name(&self) -> &'static str {
        match N {
            2 => "Naked pair",
            3 => "Naked triple",
            4 => "Naked quad",
            _ => unreachable!("subsets are pairs, triples or quads"),
        }
    }

    fn level(&self) -> Level {
        level(N)
    }

    fn score(&self) -> u32 {
        match N {
            2 => 60,
            3 => 80,
            _ => 120,
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for house in House::ALL {
            // A cell with one candidate is a naked single, not part of a subset.
            let cells: Vec<Cell> = house
                .cells()
                .into_iter()
                .filter(|&c| (2..=N).contains(&p.candidates(c).len()))
                .collect();
            for subset in choose(&cells, N) {
                let digits = subset
                    .iter()
                    .fold(Digits::NONE, |all, &c| all.union(p.candidates(c)));
                if digits.len() != N {
                    continue;
                }
                let others = house.cells().into_iter().filter(|c| !subset.contains(c));
                steps.extend(Step::removing(
                    &NakedSubset::<N>,
                    p,
                    digits,
                    vec![house],
                    subset.clone(),
                    others.map(|c| (c, digits)),
                ));
            }
        }
        steps
    }
}

impl<const N: usize> Technique for HiddenSubset<N> {
    fn name(&self) -> &'static str {
        match N {
            2 => "Hidden pair",
            3 => "Hidden triple",
            4 => "Hidden quad",
            _ => unreachable!("subsets are pairs, triples or quads"),
        }
    }

    fn level(&self) -> Level {
        level(N)
    }

    fn score(&self) -> u32 {
        match N {
            2 => 70,
            3 => 100,
            _ => 150,
        }
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for house in House::ALL {
            // A digit with one spot is a hidden single, not part of a subset.
            let digits: Vec<u8> = (1..=9)
                .filter(|&d| (2..=N).contains(&p.spots(house, d).len()))
                .collect();
            for subset in choose(&digits, N) {
                let mut cells: Vec<Cell> = subset.iter().flat_map(|&d| p.spots(house, d)).collect();
                cells.sort_unstable();
                cells.dedup();
                if cells.len() != N {
                    continue;
                }
                let keep: Digits = subset.iter().copied().collect();
                let others = Digits::ALL.without(keep);
                steps.extend(Step::removing(
                    &HiddenSubset::<N>,
                    p,
                    keep,
                    vec![house],
                    cells.clone(),
                    cells.iter().map(|&c| (c, others)),
                ));
            }
        }
        steps
    }
}

/// Pairs are Medium; triples and quads are Hard.
fn level(size: usize) -> Level {
    if size == 2 {
        Level::Medium
    } else {
        Level::Hard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{
        HIDDEN_PAIR_1, HIDDEN_PAIR_2, HIDDEN_QUAD_1, HIDDEN_QUAD_2, HIDDEN_TRIPLE_1,
        HIDDEN_TRIPLE_2, NAKED_PAIR_1, NAKED_PAIR_2, NAKED_QUAD_1, NAKED_QUAD_2, NAKED_TRIPLE_1,
        NAKED_TRIPLE_2, check,
    };

    #[test]
    fn naked_pair() {
        check(&NakedSubset::<2>, &NAKED_PAIR_1, "r8c2<>3");
        check(
            &NakedSubset::<2>,
            &NAKED_PAIR_2,
            "r456c4,r56c5,r6c6<>8, r456c4,r56c5,r6c6<>9",
        );
    }

    #[test]
    fn hidden_pair() {
        check(&HiddenSubset::<2>, &HIDDEN_PAIR_1, "r5c9<>6");
        check(
            &HiddenSubset::<2>,
            &HIDDEN_PAIR_2,
            "r1c2<>1, r1c12<>2, r1c12<>3, r1c12<>5, r1c12<>8, r1c1<>9",
        );
    }

    #[test]
    fn naked_triple() {
        check(&NakedSubset::<3>, &NAKED_TRIPLE_1, "r1c2<>6");
        check(
            &NakedSubset::<3>,
            &NAKED_TRIPLE_2,
            "r1c46,r2c456,r3c6<>1, r1c46,r2c456,r3c6<>2, r1c46,r3c6<>6",
        );
    }

    #[test]
    fn hidden_triple() {
        check(&HiddenSubset::<3>, &HIDDEN_TRIPLE_1, "r9c2<>1, r9c3<>6");
        check(
            &HiddenSubset::<3>,
            &HIDDEN_TRIPLE_2,
            "r468c6<>1, r8c6<>3, r468c6<>4, r4c6<>7, r468c6<>8",
        );
    }

    #[test]
    fn naked_quad() {
        check(
            &NakedSubset::<4>,
            &NAKED_QUAD_1,
            "r8c7<>3, r8c7<>8, r8c8<>9",
        );
        check(
            &NakedSubset::<4>,
            &NAKED_QUAD_2,
            "r7c12,r89c1,r9c2<>4, r7c12,r9c12<>6, r7c2<>9",
        );
    }

    #[test]
    fn hidden_quad() {
        check(&HiddenSubset::<4>, &HIDDEN_QUAD_1, "r7c5<>3, r78c5<>6");
        check(
            &HiddenSubset::<4>,
            &HIDDEN_QUAD_2,
            "r238c9<>3, r128c9<>6, r123c9<>7, r18c9<>8",
        );
    }
}
