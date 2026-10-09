//! A puzzle: clues with exactly one solution. Only a proof builds one, so
//! whatever holds a `Puzzle` may use the techniques that rely on there being
//! one solution, as unique rectangle does.

use crate::grid::Grid;
use crate::solver::{Solutions, solve};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    clues: Grid,
    solution: Grid,
}

/// Why clues aren't a puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotUnique {
    /// The clues clash, or no grid completes them.
    NoSolution,
    Several,
}

impl Puzzle {
    /// The puzzle `clues` make, once the solver has shown they have one
    /// solution.
    pub fn new(clues: Grid) -> Result<Puzzle, NotUnique> {
        match solve(&clues) {
            Solutions::Unique(solution) => Ok(Puzzle { clues, solution }),
            Solutions::None => Err(NotUnique::NoSolution),
            Solutions::Multiple => Err(NotUnique::Several),
        }
    }

    /// For the generator, which only ever keeps clues the solver has shown
    /// complete to `solution` alone, and for transforms, which move a
    /// puzzle's clues and solution alike.
    pub(crate) fn proven(clues: Grid, solution: Grid) -> Puzzle {
        debug_assert_eq!(solve(&clues), Solutions::Unique(solution));
        Puzzle { clues, solution }
    }

    pub fn clues(&self) -> &Grid {
        &self.clues
    }

    pub fn solution(&self) -> &Grid {
        &self.solution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLUES: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    #[test]
    fn clues_with_one_solution_make_a_puzzle() {
        let clues: Grid = CLUES.parse().unwrap();
        let puzzle = Puzzle::new(clues).unwrap();
        assert_eq!(puzzle.clues(), &clues);
        assert_eq!(Solutions::Unique(*puzzle.solution()), solve(&clues));
    }

    #[test]
    fn clues_with_several_solutions_or_none_do_not() {
        let mut few: Grid = CLUES.parse().unwrap();
        for cell in 0..20 {
            few.set(cell, None);
        }
        assert_eq!(Puzzle::new(few), Err(NotUnique::Several));
        let mut clash: Grid = CLUES.parse().unwrap();
        clash.set(2, Some(5));
        assert_eq!(Puzzle::new(clash), Err(NotUnique::NoSolution));
    }
}
