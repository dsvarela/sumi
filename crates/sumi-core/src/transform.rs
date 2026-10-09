//! Ways to change how a puzzle looks without changing what it takes to
//! solve: relabel the digits, move rows within their band and bands
//! within the board, the same for columns, and transpose. Each keeps a
//! puzzle's one solution and its rating.
//!
//! sumi's puzzles are symmetric under a half turn, so only moves that keep
//! that are made: a row moved one way has its mirror row moved the
//! mirror way.

use rand::seq::SliceRandom;
use rand::{Rng, RngExt};

use crate::grid::{CELLS, Grid, col, row};
use crate::puzzle::Puzzle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transform {
    /// The digit each digit becomes, at `digit - 1`.
    digits: [u8; 9],
    /// The row each row moves to.
    rows: [usize; 9],
    cols: [usize; 9],
    transpose: bool,
}

impl Transform {
    /// One of the transforms that keep a half-turn symmetric puzzle
    /// symmetric, each as likely as another.
    pub fn random<R: Rng + ?Sized>(rng: &mut R) -> Transform {
        let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(rng);
        Transform {
            digits,
            rows: mirrored_lines(rng),
            cols: mirrored_lines(rng),
            transpose: rng.random(),
        }
    }

    pub fn grid(&self, grid: &Grid) -> Grid {
        let mut out = Grid::EMPTY;
        for cell in 0..CELLS {
            let (r, c) = (self.rows[row(cell)], self.cols[col(cell)]);
            let (r, c) = if self.transpose { (c, r) } else { (r, c) };
            out.set(
                r * 9 + c,
                grid.get(cell).map(|d| self.digits[usize::from(d - 1)]),
            );
        }
        out
    }

    /// The puzzle with its clues and solution transformed alike, so it
    /// keeps the one solution its proof found.
    pub fn puzzle(&self, puzzle: &Puzzle) -> Puzzle {
        Puzzle::proven(self.grid(puzzle.clues()), self.grid(puzzle.solution()))
    }
}

/// Where each of the nine rows (or columns) goes, each line's mirror
/// going to the mirror of its place: the first and last bands swap or not,
/// the first band's lines are shuffled and the last band's follow as their
/// mirror, and the middle band's outer lines swap or not.
fn mirrored_lines<R: Rng + ?Sized>(rng: &mut R) -> [usize; 9] {
    let mut first = [0, 1, 2];
    first.shuffle(rng);
    let swap_bands: bool = rng.random();
    let mut lines = [0; 9];
    for (l, &to) in first.iter().enumerate() {
        lines[l] = if swap_bands { 8 - to } else { to };
        lines[8 - l] = 8 - lines[l];
    }
    let outer = if rng.random() { 5 } else { 3 };
    lines[3] = outer;
    lines[4] = 4;
    lines[5] = 8 - outer;
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grade::grade;
    use crate::solver::{Solutions, solve};
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    /// A symmetric bank puzzle.
    const HARD: &str =
        "..2......37...5..4..67....373.9.6...1...8...9...2.4.515....19..8..3...65......8..";

    fn is_symmetric(grid: &Grid) -> bool {
        (0..CELLS).all(|c| grid.get(c).is_some() == grid.get(CELLS - 1 - c).is_some())
    }

    proptest! {
        /// A transformed puzzle has one solution, the transformed one, the
        /// same rating, and stays symmetric.
        #[test]
        fn a_transform_keeps_the_solution_the_rating_and_the_symmetry(seed in any::<u64>()) {
            let puzzle = Puzzle::new(HARD.parse().unwrap()).unwrap();
            let t = Transform::random(&mut Xoshiro256PlusPlus::seed_from_u64(seed));
            let moved = t.puzzle(&puzzle);
            prop_assert_eq!(solve(moved.clues()), Solutions::Unique(*moved.solution()));
            prop_assert!(is_symmetric(moved.clues()));
            prop_assert_eq!(moved.clues().filled(), puzzle.clues().filled());
            prop_assert_eq!(grade(moved.clues()).map(|g| g.level), grade(puzzle.clues()).map(|g| g.level));
        }
    }

    /// Every part of a transform is used: over a few hundred, each row
    /// and column lands in each place its band allows, every digit takes
    /// every label, and some transpose.
    #[test]
    fn transforms_reach_every_arrangement_they_allow() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
        let ts: Vec<Transform> = (0..300).map(|_| Transform::random(&mut rng)).collect();
        for l in 0..9 {
            let places: std::collections::HashSet<usize> = ts.iter().map(|t| t.rows[l]).collect();
            let expected = if l == 4 {
                1
            } else if (3..6).contains(&l) {
                2
            } else {
                6
            };
            assert_eq!(places.len(), expected, "row {l}");
            let places: std::collections::HashSet<usize> = ts.iter().map(|t| t.cols[l]).collect();
            assert_eq!(places.len(), expected, "column {l}");
        }
        for d in 0..9 {
            let labels: std::collections::HashSet<u8> = ts.iter().map(|t| t.digits[d]).collect();
            assert_eq!(labels.len(), 9);
        }
        assert!(ts.iter().any(|t| t.transpose) && ts.iter().any(|t| !t.transpose));
    }

    /// Each part does what it says to a single digit: 5 at r1c2 becomes 6
    /// at r9c2 with the first and last rows swapped, or at r2c1 transposed.
    #[test]
    fn each_part_moves_a_digit_where_it_says() {
        let identity = Transform {
            digits: [1, 2, 3, 4, 6, 5, 7, 8, 9],
            rows: [0, 1, 2, 3, 4, 5, 6, 7, 8],
            cols: [0, 1, 2, 3, 4, 5, 6, 7, 8],
            transpose: false,
        };
        let mut grid = Grid::EMPTY;
        grid.set(1, Some(5));
        let at = |t: &Transform| {
            let out = t.grid(&grid);
            (0..CELLS).find_map(|c| out.get(c).map(|d| (c, d)))
        };
        assert_eq!(at(&identity), Some((1, 6)));
        let swapped = Transform {
            rows: [8, 1, 2, 3, 4, 5, 6, 7, 0],
            ..identity.clone()
        };
        assert_eq!(at(&swapped), Some((73, 6)));
        let transposed = Transform {
            transpose: true,
            ..identity
        };
        assert_eq!(at(&transposed), Some((9, 6)));
    }
}
