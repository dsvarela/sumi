//! New puzzles: a random full grid, then clues taken away in symmetric pairs
//! for as long as the puzzle keeps exactly one solution.

use rand::Rng;
use rand::seq::SliceRandom;

use crate::grid::{CELLS, Grid};
use crate::solver::{Solutions, search, solve};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub clues: Grid,
    pub solution: Grid,
}

/// The same generator state always gives the same puzzle. No symmetric pair
/// of clues can be removed from the result without a second solution
/// appearing.
pub fn generate<R: Rng + ?Sized>(rng: &mut R) -> Puzzle {
    let solution = random_full_grid(rng);

    // Cell i pairs with 80 - i, its mirror through the centre; the centre cell
    // pairs with itself.
    let mut pairs: Vec<usize> = (0..=CELLS / 2).collect();
    pairs.shuffle(rng);

    let mut clues = solution;
    for cell in pairs {
        let mirror = CELLS - 1 - cell;
        clues.set(cell, None);
        clues.set(mirror, None);
        if !matches!(solve(&clues), Solutions::Unique(_)) {
            clues.set(cell, solution.get(cell));
            clues.set(mirror, solution.get(mirror));
        }
    }
    Puzzle { clues, solution }
}

fn random_full_grid<R: Rng + ?Sized>(rng: &mut R) -> Grid {
    let mut first = None;
    let mut order = || {
        let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(rng);
        digits
    };
    search(&Grid::EMPTY, 1, &mut first, &mut order);
    first.expect("an empty grid always has a solution")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::tests::is_valid_solution;
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    #[test]
    fn a_thousand_puzzles_each_have_exactly_one_solution() {
        for seed in 0..1000 {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
            let Puzzle { clues, solution } = generate(&mut rng);

            assert!(is_valid_solution(&solution), "seed {seed}");
            assert_eq!(solve(&clues), Solutions::Unique(solution), "seed {seed}");
            for cell in 0..CELLS {
                let clue = clues.get(cell);
                assert!(clue.is_none() || clue == solution.get(cell), "seed {seed}");
                assert_eq!(
                    clue.is_some(),
                    clues.get(CELLS - 1 - cell).is_some(),
                    "seed {seed}: cell {cell} is not symmetric"
                );
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_puzzle() {
        let a = generate(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        let b = generate(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        let c = generate(&mut Xoshiro256PlusPlus::seed_from_u64(8));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
