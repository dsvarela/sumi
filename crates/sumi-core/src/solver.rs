//! Brute-force solving by backtracking. Used to check that a puzzle has
//! exactly one solution and to fill a grid at random; the techniques a person
//! would use live elsewhere.

use crate::grid::{CELLS, Grid, box_of, col, row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Solutions {
    None,
    Unique(Grid),
    /// Search stops at the second solution, so this says nothing about how many.
    Multiple,
}

pub fn solve(grid: &Grid) -> Solutions {
    let mut found = None;
    let count = search(grid, 2, &mut found, &mut || IN_ORDER);
    match (count, found) {
        (0, _) => Solutions::None,
        (1, Some(solution)) => Solutions::Unique(solution),
        _ => Solutions::Multiple,
    }
}

const IN_ORDER: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];

/// Counts solutions up to `limit` and stores the first one in `first`.
/// `digit_order` is asked for an order to try digits in at every guess, which
/// is how [`crate::generate`] gets a random full grid out of the same search.
/// A grid whose clues already clash has no solutions.
pub(crate) fn search(
    grid: &Grid,
    limit: usize,
    first: &mut Option<Grid>,
    digit_order: &mut dyn FnMut() -> [u8; 9],
) -> usize {
    let Some(mut state) = State::new(grid) else {
        return 0;
    };
    let mut count = 0;
    state.search(limit, &mut count, first, digit_order);
    count
}

/// The grid plus, for each row, column and box, a 9-bit mask of the digits
/// already in it (bit 0 is digit 1).
struct State {
    grid: Grid,
    rows: [u16; 9],
    cols: [u16; 9],
    boxes: [u16; 9],
}

impl State {
    fn new(grid: &Grid) -> Option<State> {
        let mut state = State {
            grid: *grid,
            rows: [0; 9],
            cols: [0; 9],
            boxes: [0; 9],
        };
        for cell in 0..CELLS {
            if let Some(digit) = grid.get(cell) {
                let bit = 1 << (digit - 1);
                if state.used(cell) & bit != 0 {
                    return None;
                }
                state.toggle(cell, bit);
            }
        }
        Some(state)
    }

    fn used(&self, cell: usize) -> u16 {
        self.rows[row(cell)] | self.cols[col(cell)] | self.boxes[box_of(cell)]
    }

    fn toggle(&mut self, cell: usize, bit: u16) {
        self.rows[row(cell)] ^= bit;
        self.cols[col(cell)] ^= bit;
        self.boxes[box_of(cell)] ^= bit;
    }

    /// Returns true once `limit` solutions have been counted.
    fn search(
        &mut self,
        limit: usize,
        count: &mut usize,
        first: &mut Option<Grid>,
        digit_order: &mut dyn FnMut() -> [u8; 9],
    ) -> bool {
        // Branch on the empty cell with the fewest candidates: it keeps the tree
        // small, and a cell with none ends this branch at once.
        let mut best: Option<(usize, u16)> = None;
        for cell in 0..CELLS {
            if self.grid.get(cell).is_some() {
                continue;
            }
            let free = !self.used(cell) & 0x1FF;
            if best.is_none_or(|(_, b)| free.count_ones() < b.count_ones()) {
                best = Some((cell, free));
                if free.count_ones() <= 1 {
                    break;
                }
            }
        }

        let Some((cell, free)) = best else {
            *count += 1;
            first.get_or_insert(self.grid);
            return *count >= limit;
        };

        for digit in digit_order() {
            let bit = 1 << (digit - 1);
            if free & bit == 0 {
                continue;
            }
            self.grid.set(cell, Some(digit));
            self.toggle(cell, bit);
            let done = self.search(limit, count, first, digit_order);
            self.toggle(cell, bit);
            self.grid.set(cell, None);
            if done {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const PUZZLE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const SOLUTION: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    /// Every row, column and box of a full grid holds each digit once.
    pub(crate) fn is_valid_solution(grid: &Grid) -> bool {
        let mut seen = [[0u16; 9]; 3];
        for cell in 0..CELLS {
            let Some(digit) = grid.get(cell) else {
                return false;
            };
            let bit = 1 << (digit - 1);
            for (house, index) in [row(cell), col(cell), box_of(cell)].into_iter().enumerate() {
                if seen[house][index] & bit != 0 {
                    return false;
                }
                seen[house][index] |= bit;
            }
        }
        true
    }

    #[test]
    fn solves_a_unique_puzzle() {
        let solution: Grid = SOLUTION.parse().unwrap();
        assert!(is_valid_solution(&solution));
        assert_eq!(solve(&PUZZLE.parse().unwrap()), Solutions::Unique(solution));
    }

    #[test]
    fn a_full_grid_is_its_own_solution() {
        let solution: Grid = SOLUTION.parse().unwrap();
        assert_eq!(solve(&solution), Solutions::Unique(solution));
    }

    #[test]
    fn too_few_clues_have_several_solutions() {
        assert_eq!(solve(&Grid::EMPTY), Solutions::Multiple);
    }

    #[test]
    fn an_emptied_deadly_rectangle_has_two_solutions() {
        // Four cells in two rows of one band, holding a b / b a: the digits can
        // swap without breaking any row, column or box, so emptying them leaves
        // exactly two solutions.
        let solution: Grid = SOLUTION.parse().unwrap();
        let at = |r: usize, c: usize| solution.get(r * 9 + c);
        let rectangle = (0..9)
            .flat_map(|r1| (r1 + 1..r1 / 3 * 3 + 3).map(move |r2| (r1, r2)))
            .flat_map(|(r1, r2)| {
                (0..9).flat_map(move |c1| (c1 / 3 * 3 + 3..9).map(move |c2| (r1, r2, c1, c2)))
            })
            .find(|&(r1, r2, c1, c2)| at(r1, c1) == at(r2, c2) && at(r1, c2) == at(r2, c1))
            .expect("this solution has a deadly rectangle");
        let (r1, r2, c1, c2) = rectangle;
        let mut grid = solution;
        for cell in [r1 * 9 + c1, r1 * 9 + c2, r2 * 9 + c1, r2 * 9 + c2] {
            grid.set(cell, None);
        }
        assert_eq!(solve(&grid), Solutions::Multiple);
    }

    #[test]
    fn clashing_clues_have_no_solution() {
        let mut grid: Grid = PUZZLE.parse().unwrap();
        grid.set(2, Some(5)); // a second 5 in row 1
        assert_eq!(solve(&grid), Solutions::None);

        // Two clues and nothing else: only the clash rules out a solution.
        let mut sparse = Grid::EMPTY;
        sparse.set(0, Some(5));
        sparse.set(1, Some(5));
        assert_eq!(solve(&sparse), Solutions::None);
    }

    #[test]
    fn a_dead_end_without_clashing_clues_has_no_solution() {
        // Row 1 holds 1 to 8 and column 9 holds a 9 lower down: the last cell of
        // row 1 has no candidate, though no two clues clash.
        let mut grid = Grid::EMPTY;
        for (cell, digit) in (0..8).zip(1..=8) {
            grid.set(cell, Some(digit));
        }
        grid.set(17, Some(9));
        assert_eq!(solve(&grid), Solutions::None);
    }
}
