//! A board as the techniques see it: the placed digits, and the candidates
//! of every empty cell.

use crate::digits::Digits;
use crate::grid::{CELLS, Cell, Grid, House, peers};
use crate::step::Step;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    grid: Grid,
    /// Empty for a cell that holds a digit.
    candidates: [Digits; CELLS],
    unique: bool,
}

impl Position {
    /// Every empty cell gets each digit that no placed digit it sees rules out.
    pub fn new(grid: &Grid) -> Position {
        Position::with_candidates(grid, std::array::from_fn(|c| grid.possible(c)))
    }

    /// Takes the candidates as given, as a player's notes or a lesson set
    /// them, less any digit a placed digit the cell sees rules out: a
    /// technique reading "only two spots" off a stale candidate would place
    /// or remove the wrong digit. A cell that holds a digit gets none.
    pub fn with_candidates(grid: &Grid, candidates: [Digits; CELLS]) -> Position {
        Position {
            grid: *grid,
            candidates: std::array::from_fn(|c| match grid.get(c) {
                Some(_) => Digits::NONE,
                None => candidates[c].intersection(grid.possible(c)),
            }),
            unique: false,
        }
    }

    /// Says the puzzle has exactly one solution. Uniqueness techniques find
    /// nothing until this is said, since on a puzzle with several solutions
    /// they remove true digits.
    pub fn known_unique(mut self) -> Position {
        self.unique = true;
        self
    }

    pub fn is_unique(&self) -> bool {
        self.unique
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn candidates(&self, cell: Cell) -> Digits {
        self.candidates[cell]
    }

    /// The cells of `house` that have `digit` as a candidate, in house order.
    pub fn spots(&self, house: House, digit: u8) -> Vec<Cell> {
        house
            .cells()
            .into_iter()
            .filter(|&c| self.candidates[c].contains(digit))
            .collect()
    }

    /// Every empty cell, in cell order.
    pub fn empty_cells(&self) -> impl Iterator<Item = Cell> + '_ {
        (0..CELLS).filter(|&c| self.grid.get(c).is_none())
    }

    /// Places the step's digit, taking it out of the candidates of every cell
    /// that sees it, and removes the step's candidates.
    pub fn apply(&mut self, step: &Step) {
        if let Some((cell, digit)) = step.placement {
            self.grid.set(cell, Some(digit));
            self.candidates[cell] = Digits::NONE;
            for p in peers(cell) {
                self.candidates[p].remove(digit);
            }
        }
        for &(cell, digits) in &step.removals {
            self.candidates[cell] = self.candidates[cell].without(digits);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::{HiddenSingle, Technique};

    /// r1c1 holds 5 and r1c2 still lists 5, the only other 5 in row 1: read
    /// as given, that is a hidden single placing a second 5 in the row.
    #[test]
    fn candidates_a_placed_digit_rules_out_are_dropped() {
        let mut grid = Grid::EMPTY;
        grid.set(0, Some(5));
        let mut candidates = [Digits::ALL; CELLS];
        for set in &mut candidates[2..9] {
            set.remove(5);
        }
        let p = Position::with_candidates(&grid, candidates);
        assert!(!p.candidates(1).contains(5));
        assert!(
            HiddenSingle
                .all(&p)
                .iter()
                .all(|s| s.placement != Some((1, 5)))
        );
    }
}
