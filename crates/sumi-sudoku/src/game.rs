//! One game in progress: the puzzle, the player's digits, the cursor, and the
//! undo history.

use sumi_core::generate::Puzzle;
use sumi_core::grid::{Cell, Grid, col, row};

pub struct Game {
    puzzle: Puzzle,
    /// Clues and the player's digits together.
    board: Grid,
    cursor: Cell,
    undo: Vec<Change>,
    redo: Vec<Change>,
}

/// One reversible step: a cell going from one value to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Change {
    cell: Cell,
    before: Option<u8>,
    after: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Game {
        Game {
            board: puzzle.clues,
            puzzle,
            cursor: 0,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn board(&self) -> &Grid {
        &self.board
    }

    pub fn cursor(&self) -> Cell {
        self.cursor
    }

    pub fn is_clue(&self, cell: Cell) -> bool {
        self.puzzle.clues.get(cell).is_some()
    }

    pub fn is_won(&self) -> bool {
        self.board.is_solved()
    }

    /// Moves one cell, wrapping around the edge of the board.
    pub fn step(&mut self, direction: Direction) {
        self.cursor = moved(self.cursor, direction, 1);
    }

    /// Moves to the same spot in the next box, wrapping around the board.
    pub fn jump(&mut self, direction: Direction) {
        self.cursor = moved(self.cursor, direction, 3);
    }

    /// Writes a digit, or clears the cell with `None`. Clues stay as they are,
    /// and writing what is already there records nothing.
    pub fn write(&mut self, digit: Option<u8>) {
        let cell = self.cursor;
        let before = self.board.get(cell);
        if self.is_clue(cell) || before == digit {
            return;
        }
        let change = Change {
            cell,
            before,
            after: digit,
        };
        self.apply(change.cell, change.after);
        self.undo.push(change);
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        if let Some(change) = self.undo.pop() {
            self.apply(change.cell, change.before);
            self.redo.push(change);
        }
    }

    pub fn redo(&mut self) {
        if let Some(change) = self.redo.pop() {
            self.apply(change.cell, change.after);
            self.undo.push(change);
        }
    }

    /// The cursor follows every change, so undo shows where it happened.
    fn apply(&mut self, cell: Cell, value: Option<u8>) {
        self.board.set(cell, value);
        self.cursor = cell;
    }
}

fn moved(cell: Cell, direction: Direction, by: usize) -> Cell {
    let (r, c) = (row(cell), col(cell));
    let (r, c) = match direction {
        Direction::Up => ((r + 9 - by) % 9, c),
        Direction::Down => ((r + by) % 9, c),
        Direction::Left => (r, (c + 9 - by) % 9),
        Direction::Right => (r, (c + by) % 9),
    };
    r * 9 + c
}

#[cfg(test)]
mod tests {
    use super::*;
    use sumi_core::grid::CELLS;
    use sumi_core::solver::{Solutions, solve};

    const CLUES: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    fn game() -> Game {
        let clues: Grid = CLUES.parse().unwrap();
        let Solutions::Unique(solution) = solve(&clues) else {
            panic!("test puzzle has one solution");
        };
        Game::new(Puzzle { clues, solution })
    }

    #[test]
    fn steps_wrap_around_the_edges() {
        let mut g = game();
        g.step(Direction::Left);
        assert_eq!(g.cursor(), 8);
        g.step(Direction::Up);
        assert_eq!(g.cursor(), 80);
        g.step(Direction::Right);
        assert_eq!(g.cursor(), 72);
        g.step(Direction::Down);
        assert_eq!(g.cursor(), 0);
    }

    #[test]
    fn jumps_move_a_box_at_a_time() {
        let mut g = game();
        g.step(Direction::Right); // r1c2
        g.jump(Direction::Right);
        assert_eq!(g.cursor(), 4); // r1c5
        g.jump(Direction::Down);
        assert_eq!(g.cursor(), 31); // r4c5
        g.jump(Direction::Left);
        g.jump(Direction::Left);
        assert_eq!(g.cursor(), 34); // wrapped to r4c8
    }

    #[test]
    fn clues_cannot_be_changed() {
        let mut g = game();
        g.write(Some(1));
        g.write(None);
        assert_eq!(g.board().get(0), Some(5));
        g.undo();
        assert_eq!(g.board().get(0), Some(5));
    }

    #[test]
    fn undo_and_redo_walk_the_history() {
        let mut g = game();
        g.step(Direction::Right);
        g.step(Direction::Right); // r1c3, empty
        g.write(Some(4));
        g.write(Some(2));
        g.write(None);
        assert_eq!(g.board().get(2), None);

        g.undo();
        assert_eq!(g.board().get(2), Some(2));
        g.undo();
        assert_eq!(g.board().get(2), Some(4));
        g.undo();
        assert_eq!(g.board().get(2), None);
        g.undo(); // nothing left
        assert_eq!(g.board().get(2), None);

        g.redo();
        g.redo();
        assert_eq!(g.board().get(2), Some(2));
    }

    #[test]
    fn a_new_change_drops_the_redo_steps() {
        let mut g = game();
        g.step(Direction::Right);
        g.step(Direction::Right);
        g.write(Some(4));
        g.undo();
        g.write(Some(1));
        g.redo();
        assert_eq!(g.board().get(2), Some(1));
    }

    #[test]
    fn writing_the_same_digit_twice_is_one_step() {
        let mut g = game();
        g.step(Direction::Right);
        g.step(Direction::Right);
        g.write(Some(4));
        g.write(Some(4));
        g.undo();
        assert_eq!(g.board().get(2), None);
    }

    #[test]
    fn undo_moves_the_cursor_to_the_change() {
        let mut g = game();
        g.step(Direction::Right);
        g.step(Direction::Right);
        g.write(Some(4));
        g.step(Direction::Down);
        g.undo();
        assert_eq!(g.cursor(), 2);
    }

    #[test]
    fn filling_in_the_solution_wins() {
        let mut g = game();
        let solution = g.puzzle.solution;
        assert!(!g.is_won());
        for cell in 0..CELLS {
            g.cursor = cell;
            g.write(solution.get(cell));
        }
        assert!(g.is_won());
    }
}
