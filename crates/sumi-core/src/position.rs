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
    /// them. A cell that holds a digit gets none.
    pub fn with_candidates(grid: &Grid, candidates: [Digits; CELLS]) -> Position {
        Position {
            grid: *grid,
            candidates: std::array::from_fn(|c| match grid.get(c) {
                Some(_) => Digits::NONE,
                None => candidates[c],
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
