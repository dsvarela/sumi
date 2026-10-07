//! The solving techniques, and the order hints and grading try them in.

mod claiming;
mod fish;
mod full_house;
mod hidden_single;
mod naked_single;
mod pointing;
mod subset;

#[cfg(test)]
pub(crate) mod examples;

pub use claiming::Claiming;
pub use fish::Fish;
pub use full_house::FullHouse;
pub use hidden_single::HiddenSingle;
pub use naked_single::NakedSingle;
pub use pointing::Pointing;
pub use subset::{HiddenSubset, NakedSubset};

use crate::position::Position;
use crate::step::Step;

pub trait Technique: Sync {
    /// The name players see, such as "Hidden single" or "Y-Wing".
    fn name(&self) -> &'static str;

    fn level(&self) -> Level;

    /// Every step this technique finds in `p`. Each one places a digit or
    /// removes at least one candidate. The same pattern may be listed once
    /// per row, column or box it can be read from.
    fn all(&self, p: &Position) -> Vec<Step>;

    /// The step a hint shows: the one with the fewest cells, then the one
    /// whose cells come first reading from the top left.
    fn find(&self, p: &Position) -> Option<Step> {
        self.all(p)
            .into_iter()
            .min_by(|a, b| (a.cells.len(), &a.cells).cmp(&(b.cells.len(), &b.cells)))
    }
}

/// How hard a technique is, and so how hard a puzzle is: the level of the
/// hardest technique it needs, or Beyond when sumi's techniques can't finish
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Easy,
    Medium,
    Hard,
    Expert,
    Beyond,
}

/// Easiest first. Hints and grading take the first technique that finds a
/// step.
pub static ORDER: &[&dyn Technique] = &[
    &FullHouse,
    &HiddenSingle,
    &NakedSingle,
    &Pointing,
    &Claiming,
    &NakedSubset::<2>,
    &HiddenSubset::<2>,
    &NakedSubset::<3>,
    &HiddenSubset::<3>,
    &Fish::<2>,
    &NakedSubset::<4>,
    &Fish::<3>,
    &HiddenSubset::<4>,
];

/// Every way to choose `k` of `items`, keeping their order.
fn choose<T: Copy>(items: &[T], k: usize) -> Vec<Vec<T>> {
    if k == 0 {
        return vec![Vec::new()];
    }
    let mut all = Vec::new();
    for (i, &first) in items.iter().enumerate() {
        for mut rest in choose(&items[i + 1..], k - 1) {
            rest.insert(0, first);
            all.push(rest);
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digits::Digits;
    use crate::generate::generate;
    use crate::grid::{CELLS, Grid};
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    proptest! {
        /// From a generated puzzle with some wrong candidates already
        /// removed, as a player might have, every step every technique finds
        /// keeps the solution, and so does every step after it while solving.
        #[test]
        fn no_technique_ever_removes_the_true_digit(
            seed in any::<u64>(),
            removals in prop::collection::vec((0..CELLS, 1u8..=9), 0..150),
        ) {
            let puzzle = generate(&mut Xoshiro256PlusPlus::seed_from_u64(seed));
            let mut candidates: [Digits; CELLS] =
                std::array::from_fn(|c| puzzle.clues.possible(c));
            for (cell, digit) in removals {
                if puzzle.solution.get(cell) != Some(digit) {
                    candidates[cell].remove(digit);
                }
            }
            let mut p = Position::with_candidates(&puzzle.clues, candidates).known_unique();
            loop {
                let mut first = None;
                for technique in ORDER {
                    for step in technique.all(&p) {
                        for &(cell, digits) in &step.removals {
                            let answer = puzzle.solution.get(cell).unwrap();
                            prop_assert!(!digits.contains(answer), "{:?} in {:?}", step, p);
                        }
                        if let Some((cell, digit)) = step.placement {
                            prop_assert_eq!(puzzle.solution.get(cell), Some(digit), "{:?}", step);
                        }
                        first.get_or_insert(step);
                    }
                    if first.is_some() {
                        break;
                    }
                }
                match first {
                    Some(step) => p.apply(&step),
                    None => break,
                }
            }
        }
    }

    #[test]
    fn the_order_never_gets_easier_and_names_are_distinct() {
        for pair in ORDER.windows(2) {
            assert!(pair[0].level() <= pair[1].level(), "{pair:?}");
        }
        assert!(ORDER.iter().all(|t| t.level() < Level::Beyond));
        for (i, a) in ORDER.iter().enumerate() {
            assert!(ORDER[i + 1..].iter().all(|b| a.name() != b.name()));
        }
    }

    /// The plan's table of levels, in search order.
    #[test]
    fn the_order_is_the_plans() {
        use Level::*;
        let plan = [
            ("Full house", Easy),
            ("Hidden single", Easy),
            ("Naked single", Easy),
            ("Pointing pair", Medium),
            ("Claiming", Medium),
            ("Naked pair", Medium),
            ("Hidden pair", Medium),
            ("Naked triple", Hard),
            ("Hidden triple", Hard),
            ("X-Wing", Hard),
            ("Naked quad", Hard),
            ("Swordfish", Hard),
            ("Hidden quad", Hard),
        ];
        let order: Vec<_> = ORDER.iter().map(|t| (t.name(), t.level())).collect();
        assert_eq!(order, plan);
    }

    #[test]
    fn choose_keeps_order() {
        assert_eq!(
            choose(&[1, 2, 3, 4], 2),
            vec![
                vec![1, 2],
                vec![1, 3],
                vec![1, 4],
                vec![2, 3],
                vec![2, 4],
                vec![3, 4]
            ]
        );
        assert_eq!(choose(&[1, 2], 3), Vec::<Vec<i32>>::new());
    }

    /// A pointing triple on 1 in box 1 and a pointing pair on 2 in box 9:
    /// the pair has fewer cells, so it wins though the triple is further up.
    #[test]
    fn find_takes_the_fewest_cells_first() {
        let mut candidates = [Digits::ALL; 81];
        for cell in [9, 10, 11, 18, 19, 20] {
            candidates[cell].remove(1); // box 1 keeps 1 only in row 1
        }
        for cell in [60, 61, 62, 69, 70, 71, 78] {
            candidates[cell].remove(2); // box 9 keeps 2 only in r9c8 and r9c9
        }
        let p = Position::with_candidates(&Grid::EMPTY, candidates);

        assert!(Pointing.all(&p).iter().any(|s| s.cells == [0, 1, 2]));
        assert_eq!(Pointing.find(&p).unwrap().cells, [79, 80]);
    }

    /// Two naked pairs in row 1: r1c1-r1c2 on {1,2} and r1c8-r1c9 on {8,9}.
    /// The first comes first from the top left, though both have 2 cells.
    #[test]
    fn find_takes_the_fewest_cells_then_the_top_left() {
        let pair = |a: u8, b: u8| -> Digits { [a, b].into_iter().collect() };
        let mut candidates = [Digits::ALL; 81];
        candidates[0] = pair(1, 2);
        candidates[1] = pair(1, 2);
        candidates[7] = pair(8, 9);
        candidates[8] = pair(8, 9);
        let p = Position::with_candidates(&Grid::EMPTY, candidates);

        let all = NakedSubset::<2>.all(&p);
        assert!(all.iter().any(|s| s.cells == [7, 8]));
        assert_eq!(NakedSubset::<2>.find(&p).unwrap().cells, [0, 1]);
    }
}
