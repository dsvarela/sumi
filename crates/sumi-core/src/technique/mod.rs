//! The solving techniques, and the order they are tried in.

mod claiming;
mod colouring;
mod fish;
mod full_house;
mod hidden_single;
mod kite;
mod naked_single;
mod pointing;
mod skyscraper;
mod subset;
mod unique_rectangle;
mod w_wing;
mod xyz_wing;
mod y_wing;

#[cfg(test)]
pub(crate) mod examples;

pub use claiming::Claiming;
pub use colouring::SimpleColouring;
pub use fish::Fish;
pub use full_house::FullHouse;
pub use hidden_single::HiddenSingle;
pub use kite::Kite;
pub use naked_single::NakedSingle;
pub use pointing::Pointing;
pub use skyscraper::Skyscraper;
pub use subset::{HiddenSubset, NakedSubset};
pub use unique_rectangle::UniqueRectangle;
pub use w_wing::WWing;
pub use xyz_wing::XyzWing;
pub use y_wing::YWing;

use crate::grid::{CELLS, Cell, House, box_of, col, row, sees};
use crate::position::Position;
use crate::step::{Explanation, Step};

pub trait Technique: Sync {
    /// The name players see, such as "Hidden single" or "Y-Wing".
    fn name(&self) -> &'static str;

    fn level(&self) -> Level;

    /// What one step of it adds to a puzzle's score, which sorts puzzles
    /// within a level. The values are HoDoKu's defaults.
    fn score(&self) -> u32;

    /// Every step this technique finds in `p`, in the order it looks. Each
    /// one places a digit or removes at least one candidate. A pattern read
    /// from several rows, columns or boxes may be listed once for each.
    fn search(&self, p: &Position) -> Vec<Step>;

    /// Every distinct step, simplest first: the fewest cells, then the cells
    /// that come first reading from the top left. Steps with the same effect
    /// are one step, kept in their simplest form, so the count is how many
    /// different things the technique can do here.
    fn all(&self, p: &Position) -> Vec<Step> {
        let mut steps = self.search(p);
        steps.sort_by(|a, b| (a.cells.len(), &a.cells).cmp(&(b.cells.len(), &b.cells)));
        let mut distinct: Vec<Step> = Vec::new();
        for step in steps {
            let same = |s: &Step| s.placement == step.placement && s.removals == step.removals;
            if !distinct.iter().any(same) {
                distinct.push(step);
            }
        }
        distinct
    }

    /// What the technique looks for, in one sentence. It depends on the step
    /// only where the technique has a row and a column form.
    fn definition(&self, step: &Step) -> String;

    /// The step's name as the hint says it, which `name` alone can't give
    /// when the name depends on the step, such as a pointing triple.
    fn title(&self, _step: &Step) -> String {
        self.name().to_string()
    }

    /// "Look for a hidden single." for the hint's second rung.
    fn look_for(&self, step: &Step) -> String {
        let title = self.title(step);
        let mut chars = title.chars();
        let first = chars.next().unwrap_or(' ');
        let second = chars.next().unwrap_or(' ');
        // "Hidden single" reads as "a hidden single"; "X-Wing" keeps its
        // capitals.
        let name = if second.is_lowercase() {
            format!("{}{}", first.to_lowercase(), &title[first.len_utf8()..])
        } else {
            title
        };
        let an = name.starts_with(['a', 'e', 'i', 'o', 'X']);
        format!("Look for {} {name}.", if an { "an" } else { "a" })
    }

    /// The wording of a step this technique found, from its fields. Panics
    /// on a step another technique found.
    fn explain(&self, step: &Step) -> Explanation;

    /// The simplest step, so the same position always gives the same one.
    fn find(&self, p: &Position) -> Option<Step> {
        self.all(p).into_iter().next()
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

/// Bumped whenever a puzzle's grade can change: a change to [`ORDER`], to a
/// technique's search or score, or to how `find` chooses. Stored grades must
/// then be redone, and the bank test fails until they are.
pub const ORDER_VERSION: u32 = 1;

/// Easiest first. Grading takes the first technique that finds a step, and
/// hints are to do the same.
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
    &Skyscraper,
    &Kite,
    &WWing,
    &YWing,
    &XyzWing,
    &UniqueRectangle,
    &SimpleColouring,
];

/// The cells a step removes candidates from, in cell order.
fn removed_cells(step: &Step) -> Vec<Cell> {
    step.removals.iter().map(|&(cell, _)| cell).collect()
}

/// A digit's only two spots in a house, so one of them holds it. Houses
/// are taken from `houses` in order.
fn strong_links(p: &Position, digit: u8, houses: &[House]) -> Vec<(Cell, Cell, House)> {
    houses
        .iter()
        .filter_map(|&house| match p.spots(house, digit)[..] {
            [a, b] => Some((a, b, house)),
            _ => None,
        })
        .collect()
}

/// The cells that see every one of `cells`, in cell order.
fn seen_by_all(cells: &[Cell]) -> impl Iterator<Item = Cell> + '_ {
    (0..CELLS).filter(|&c| cells.iter().all(|&x| sees(c, x)))
}

/// Cells with exactly two candidates, in cell order.
fn bivalue_cells(p: &Position) -> Vec<Cell> {
    p.empty_cells()
        .filter(|&c| p.candidates(c).len() == 2)
        .collect()
}

/// A house both cells are in, the box first. Only for cells that see each
/// other.
fn shared_house(a: Cell, b: Cell) -> House {
    if box_of(a) == box_of(b) {
        House::Box(box_of(a))
    } else if row(a) == row(b) {
        House::Row(row(a))
    } else {
        House::Col(col(a))
    }
}

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
    use crate::grid::Grid;
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
            ("Skyscraper", Expert),
            ("2-String Kite", Expert),
            ("W-Wing", Expert),
            ("Y-Wing", Expert),
            ("XYZ-Wing", Expert),
            ("Unique rectangle", Expert),
            ("Simple colouring", Expert),
        ];
        let order: Vec<_> = ORDER.iter().map(|t| (t.name(), t.level())).collect();
        assert_eq!(order, plan);
        // A change to the list above changes grades: bump ORDER_VERSION with
        // it, and rebuild the bank.
        assert_eq!(ORDER_VERSION, 1);
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

    /// Two naked pairs: r4c4-r4c5 on {8,9}, found first since boxes are
    /// read before rows, and r1c1-r1c9 on {1,2}, which is nearer the top
    /// left. Both have 2 cells, so the top left decides.
    #[test]
    fn find_takes_the_fewest_cells_then_the_top_left() {
        let pair = |a: u8, b: u8| -> Digits { [a, b].into_iter().collect() };
        let mut candidates = [Digits::ALL; 81];
        candidates[0] = pair(1, 2);
        candidates[8] = pair(1, 2);
        candidates[30] = pair(8, 9);
        candidates[31] = pair(8, 9);
        let p = Position::with_candidates(&Grid::EMPTY, candidates);

        assert_eq!(NakedSubset::<2>.search(&p)[0].cells, [30, 31]);
        assert_eq!(NakedSubset::<2>.find(&p).unwrap().cells, [0, 8]);
    }

    /// A step read from both its box and its row is listed once.
    #[test]
    fn all_lists_each_effect_once() {
        let examples = [
            &examples::HIDDEN_PAIR_1,
            &examples::HIDDEN_PAIR_2,
            &examples::NAKED_SINGLE_1,
            &examples::W_WING_2,
        ];
        let mut repeats = 0;
        for example in examples {
            let p = example.position().known_unique();
            for technique in ORDER {
                let all = technique.all(&p);
                repeats += technique.search(&p).len() - all.len();
                for (i, a) in all.iter().enumerate() {
                    for b in &all[i + 1..] {
                        assert!(a.placement != b.placement || a.removals != b.removals);
                    }
                }
            }
        }
        assert!(repeats > 0, "the examples have no repeated step to drop");
    }
}
