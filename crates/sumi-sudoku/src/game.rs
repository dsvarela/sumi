//! One game in progress: the puzzle and its rating, the player's digits
//! and notes, the cursor, the input mode, the undo history, the hints and
//! reveals used, and the time played.

use std::time::{Duration, Instant};
use sumi_core::digits::Digits;
use sumi_core::grid::{CELLS, Cell, Grid, col, peers, row};
use sumi_core::hint::{self, Hint};
use sumi_core::puzzle::Puzzle;
use sumi_core::step::Step as SolverStep;
use sumi_core::technique::Level;

mod saved;

#[derive(Clone)]
pub struct Game {
    puzzle: Puzzle,
    /// Clues and the player's digits together.
    board: Grid,
    /// `None` is a cell the player has written no candidates in. `Some` of an
    /// empty set is a cell whose candidates were all ruled out by digits placed
    /// around it, which is a contradiction worth seeing rather than hiding.
    candidates: [Option<Digits>; CELLS],
    /// Box marks: "this digit goes in one of these spots in its box".
    marks: [Digits; CELLS],
    hidden: [Hidden; CELLS],
    cursor: Cell,
    mode: Mode,
    candidates_from_marks: CandidatesFromMarks,
    /// The rung a hint opens at, from 1.
    hint_start: usize,
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// Bumped by every change to the board or the notes.
    revision: u64,
    /// The puzzle's rating, when it is known.
    level: Option<Level>,
    /// Hint ladders opened, and cells `r` changed.
    hints_used: u32,
    revealed: u32,
    /// Time played before the clock last started, and when that was, if it
    /// is running.
    banked: Duration,
    running_since: Option<Instant>,
    /// Boards to come back to after a guess, oldest first.
    bookmarks: Vec<Bookmark>,
}

/// Every cell as it was when `b` was pressed, and the cursor.
#[derive(Clone, PartialEq, Eq)]
struct Bookmark {
    cursor: Cell,
    cells: [CellState; CELLS],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Digit,
    Candidate,
    BoxMark,
}

/// What moving from box marks to candidates does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CandidatesFromMarks {
    /// Every empty cell gets all its possible candidates, and the box marks
    /// go.
    #[default]
    Fill,
    /// Each cell's box marks join its candidates.
    Merge,
}

/// The two kinds of note a cell can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Candidates,
    BoxMarks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Everything the player can change about one cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CellState {
    digit: Option<u8>,
    candidates: Option<Digits>,
    marks: Digits,
    hidden: Hidden,
}

/// Notes the player wrote that a placed digit rules out. They stay out of
/// sight until no placed digit rules them out, so erasing digits in any order
/// gives back every note they took. A cell holding the player's digit keeps
/// all its notes here.
///
/// A cell with no candidates written (`None`) has no hidden candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Hidden {
    candidates: Digits,
    marks: Digits,
}

/// One undo step: every cell an action changed, before and after. An action
/// that touches many cells, such as placing a digit and clearing it from the
/// notes around it, is still one step.
#[derive(Clone)]
struct Step {
    /// Where the cursor was when the action ran. Undo and redo put it back
    /// there, which for a one-cell action is the cell that changed.
    cursor: Cell,
    cells: Vec<(Cell, CellState, CellState)>,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Game {
        Game {
            board: *puzzle.clues(),
            puzzle,
            candidates: [None; CELLS],
            marks: [Digits::NONE; CELLS],
            hidden: [Hidden::default(); CELLS],
            cursor: 0,
            mode: Mode::Digit,
            candidates_from_marks: CandidatesFromMarks::default(),
            hint_start: 1,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            level: None,
            hints_used: 0,
            revealed: 0,
            banked: Duration::ZERO,
            running_since: None,
            bookmarks: Vec::new(),
        }
    }

    /// The game with its puzzle's rating.
    pub fn rated(self, level: Level) -> Game {
        Game {
            level: Some(level),
            ..self
        }
    }

    pub fn level(&self) -> Option<Level> {
        self.level
    }

    /// Counts a hint ladder the player opened.
    pub fn count_hint(&mut self) {
        self.hints_used += 1;
    }

    pub fn hints_used(&self) -> u32 {
        self.hints_used
    }

    pub fn revealed(&self) -> u32 {
        self.revealed
    }

    /// Starts the clock at `now`, unless it is running.
    pub fn start_clock(&mut self, now: Instant) {
        self.running_since.get_or_insert(now);
    }

    /// Stops the clock at `now`, keeping the time played so far.
    pub fn stop_clock(&mut self, now: Instant) {
        if let Some(since) = self.running_since.take() {
            self.banked += now.saturating_duration_since(since);
        }
    }

    pub fn clock_is_running(&self) -> bool {
        self.running_since.is_some()
    }

    /// Time played up to `now`.
    pub fn elapsed(&self, now: Instant) -> Duration {
        let running = self
            .running_since
            .map_or(Duration::ZERO, |since| now.saturating_duration_since(since));
        self.banked + running
    }

    pub fn board(&self) -> &Grid {
        &self.board
    }

    pub fn cursor(&self) -> Cell {
        self.cursor
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn candidates(&self, cell: Cell) -> Digits {
        self.candidates[cell].unwrap_or(Digits::NONE)
    }

    pub fn marks(&self, cell: Cell) -> Digits {
        self.marks[cell]
    }

    /// The candidates the filter works from: the player's own, or every
    /// possible digit in a cell where the player has written none. A cell
    /// whose candidates were all ruled out has none.
    pub fn working_candidates(&self, cell: Cell) -> Digits {
        if self.board.get(cell).is_some() {
            return Digits::NONE;
        }
        self.candidates[cell].unwrap_or_else(|| self.board.possible(cell))
    }

    /// Whether a cell that `cell` sees already holds `digit`, so a note for
    /// it in `cell` is wrong. Says nothing about whether such a note exists.
    pub fn note_clashes(&self, cell: Cell, digit: u8) -> bool {
        !self.board.possible(cell).contains(digit)
    }

    /// Nine copies of `digit` are on the board and none of them clash.
    pub fn is_complete(&self, digit: u8) -> bool {
        let clash = self.board.conflicts();
        (0..CELLS)
            .filter(|&c| self.board.get(c) == Some(digit) && !clash[c])
            .count()
            == 9
    }

    pub fn is_clue(&self, cell: Cell) -> bool {
        self.puzzle.clues().get(cell).is_some()
    }

    pub fn is_won(&self) -> bool {
        self.board.is_solved()
    }

    pub fn candidates_from_marks(&self) -> CandidatesFromMarks {
        self.candidates_from_marks
    }

    pub fn set_candidates_from_marks(&mut self, choice: CandidatesFromMarks) {
        self.candidates_from_marks = choice;
    }

    /// The rung a new hint opens at; a ladder shorter than that opens at its
    /// last rung.
    pub fn hint_start(&self) -> usize {
        self.hint_start
    }

    pub fn set_hint_start(&mut self, rung: usize) {
        self.hint_start = rung.clamp(1, 5);
    }

    /// Digit, then candidate, then box mark, then back to digit.
    pub fn next_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Digit => Mode::Candidate,
            Mode::Candidate => Mode::BoxMark,
            Mode::BoxMark => Mode::Digit,
        };
    }

    /// The layer clearing works on: box marks in box-mark mode, otherwise
    /// candidates.
    pub fn current_layer(&self) -> Layer {
        match self.mode {
            Mode::BoxMark => Layer::BoxMarks,
            Mode::Digit | Mode::Candidate => Layer::Candidates,
        }
    }

    /// Moves one cell. Leaving the board on one side comes back on the other
    /// side of the same row or column.
    pub fn step(&mut self, direction: Direction) {
        self.cursor = moved(self.cursor, direction, 1);
    }

    pub fn move_to(&mut self, cell: Cell) {
        self.cursor = cell;
    }

    /// The last move kept in the undo history: where the cursor was, and
    /// the cells whose digit it changed or, for a move of notes only, the
    /// cells whose notes it changed.
    pub fn last_move(&self) -> Option<(Cell, Vec<Cell>)> {
        let step = self.undo.last()?;
        let digits: Vec<Cell> = step
            .cells
            .iter()
            .filter(|(_, before, after)| before.digit != after.digit)
            .map(|&(cell, _, _)| cell)
            .collect();
        let cells = if digits.is_empty() {
            step.cells.iter().map(|&(cell, _, _)| cell).collect()
        } else {
            digits
        };
        Some((step.cursor, cells))
    }

    /// Moves to the same spot in the next box, wrapping around the board.
    pub fn jump(&mut self, direction: Direction) {
        self.cursor = moved(self.cursor, direction, 3);
    }

    /// Places `digit` in digit mode, or toggles it as a note in the other two.
    /// Clues never change, and notes only go in empty cells.
    pub fn press(&mut self, digit: u8) {
        let cell = self.cursor;
        if self.is_clue(cell) {
            return;
        }
        match self.mode {
            Mode::Digit => self.record(|g| g.place(cell, digit)),
            Mode::Candidate if self.board.get(cell).is_none() => self.record(|g| {
                let mut set = g.candidates[cell].unwrap_or(Digits::NONE);
                set.toggle(digit);
                // Taking out your own last candidate means no notes, not a
                // contradiction, unless placed digits hide others.
                let none_left = set.is_empty() && g.hidden[cell].candidates.is_empty();
                g.candidates[cell] = (!none_left).then_some(set);
            }),
            Mode::BoxMark if self.board.get(cell).is_none() => {
                self.record(|g| g.marks[cell].toggle(digit))
            }
            Mode::Candidate | Mode::BoxMark => {}
        }
    }

    /// Removes the player's digit from the cursor cell, or its notes if it
    /// holds no digit.
    pub fn clear(&mut self) {
        let cell = self.cursor;
        if self.is_clue(cell) {
            return;
        }
        self.record(|g| {
            if g.board.get(cell).is_some() {
                g.erase(cell);
            } else {
                g.candidates[cell] = None;
                g.marks[cell] = Digits::NONE;
                g.hidden[cell] = Hidden::default();
            }
        });
    }

    /// Gives every empty cell the player has written no candidates in all its
    /// possible ones. Box marks don't count as candidates here, and a cell
    /// that ran out of candidates keeps showing that it did.
    pub fn fill_candidates(&mut self) {
        self.record(|g| {
            for cell in 0..CELLS {
                if g.board.get(cell).is_none() && g.candidates[cell].is_none() {
                    g.fill(cell);
                }
            }
        });
    }

    /// Replaces the candidates of every empty cell with all its possible ones.
    pub fn refill_candidates(&mut self) {
        self.record(Game::refill);
    }

    /// Moves from box marks to candidates, as `candidates_from_marks` says.
    pub fn marks_to_candidates(&mut self) {
        let choice = self.candidates_from_marks;
        self.record(|g| {
            if choice == CandidatesFromMarks::Fill {
                g.refill();
            }
            for cell in 0..CELLS {
                let hidden = &mut g.hidden[cell];
                if choice == CandidatesFromMarks::Merge
                    && !(g.marks[cell].is_empty() && hidden.marks.is_empty())
                {
                    let set = g.candidates[cell].unwrap_or(Digits::NONE);
                    g.candidates[cell] = Some(set.union(g.marks[cell]));
                    hidden.candidates = hidden.candidates.union(hidden.marks);
                }
                g.marks[cell] = Digits::NONE;
                hidden.marks = Digits::NONE;
            }
        });
    }

    /// Clears one layer of notes from the whole board, or both with `None`.
    pub fn clear_notes(&mut self, layer: Option<Layer>) {
        self.record(|g| {
            for cell in 0..CELLS {
                if layer != Some(Layer::BoxMarks) {
                    g.candidates[cell] = None;
                    g.hidden[cell].candidates = Digits::NONE;
                }
                if layer != Some(Layer::Candidates) {
                    g.marks[cell] = Digits::NONE;
                    g.hidden[cell].marks = Digits::NONE;
                }
            }
        });
    }

    /// The hint for the board as it is, built from the player's candidates.
    pub fn hint(&self) -> Hint {
        hint::find(&self.board, &self.puzzle, &self.candidates)
    }

    /// Does what a hint's step says: places its digit, or removes its
    /// candidates from the player's lists. A cell without a list gets the
    /// digits it can hold, less the removed ones. One undo step.
    pub fn apply_hint(&mut self, step: &SolverStep) {
        self.record(|g| {
            if let Some((cell, digit)) = step.placement {
                g.place(cell, digit);
            }
            for &(cell, digits) in &step.removals {
                // Writing the list out as fill does keeps the notes that
                // the player's own digits rule out ready to come back.
                if g.candidates[cell].is_none() {
                    g.fill(cell);
                }
                let listed = g.working_candidates(cell);
                g.candidates[cell] = Some(listed.without(digits));
            }
        });
    }

    /// The player's digits that differ from the solution, in cell order.
    pub fn wrong_digits(&self) -> Vec<(Cell, u8)> {
        (0..CELLS)
            .filter(|&c| !self.is_clue(c))
            .filter_map(|c| {
                let digit = self.board.get(c)?;
                (self.puzzle.solution().get(c) != Some(digit)).then_some((c, digit))
            })
            .collect()
    }

    /// Writes the solution's digit in the cursor cell. A clue or a right
    /// digit is already it, so nothing changes.
    pub fn reveal(&mut self) {
        let cell = self.cursor;
        if let Some(answer) = self.puzzle.solution().get(cell) {
            let before = self.revision;
            self.record(|g| g.place(cell, answer));
            if self.revision != before {
                self.revealed += 1;
            }
        }
    }

    /// Keeps the board and every note as they are, to come back to with
    /// [`Game::back_to_bookmark`]. A board the same as the newest bookmark's
    /// adds none.
    pub fn bookmark(&mut self) {
        let here = self.here();
        if self.bookmarks.last().map(|b| b.cells) != Some(here.cells) {
            self.bookmarks.push(here);
        }
    }

    /// Goes back to the newest bookmark, keeping it, so another guess can
    /// start from it. On it already, drops it and goes back to the one
    /// before, if there is one. Going back is one undo step. Says whether
    /// anything changed.
    pub fn back_to_bookmark(&mut self) -> bool {
        let at_newest = self
            .bookmarks
            .last()
            .is_some_and(|b| b.cells == self.here().cells);
        if at_newest && self.bookmarks.len() > 1 {
            self.bookmarks.pop();
        }
        match self.bookmarks.last().cloned() {
            Some(bookmark) if bookmark.cells != self.here().cells => {
                self.go_to(&bookmark);
                true
            }
            _ => false,
        }
    }

    /// Whether the board can't be finished as it stands: two digits clash,
    /// or an empty cell has no digit left, by the digits around it or by
    /// the candidates written in it. A guess has gone wrong.
    pub fn at_dead_end(&self) -> bool {
        self.board.conflicts().contains(&true)
            || (0..CELLS).any(|c| {
                self.board.get(c).is_none()
                    && (self.board.possible(c).is_empty()
                        || self.candidates[c].is_some_and(Digits::is_empty))
            })
    }

    /// The cells left to fill at each bookmark, oldest first.
    pub fn bookmarks(&self) -> Vec<usize> {
        self.bookmarks
            .iter()
            .map(|b| b.cells.iter().filter(|s| s.digit.is_none()).count())
            .collect()
    }

    /// Goes back to the bookmark at `index`, oldest first, dropping those
    /// after it.
    pub fn go_to_bookmark(&mut self, index: usize) {
        self.bookmarks.truncate(index + 1);
        if let Some(bookmark) = self.bookmarks.last().cloned() {
            self.go_to(&bookmark);
        }
    }

    fn here(&self) -> Bookmark {
        Bookmark {
            cursor: self.cursor,
            cells: std::array::from_fn(|c| self.state(c)),
        }
    }

    fn go_to(&mut self, bookmark: &Bookmark) {
        self.record(|g| {
            for (cell, &state) in bookmark.cells.iter().enumerate() {
                g.set_state(cell, state);
            }
        });
        self.cursor = bookmark.cursor;
    }

    /// Changes when the board or the notes do, by any action, undo and redo
    /// included; moving and switching mode leave it. Whatever was worked out
    /// from the game at one revision is stale at any other.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Takes back the last step and puts the cursor where it was then.
    pub fn undo(&mut self) {
        if let Some(step) = self.undo.pop() {
            for &(cell, before, _) in &step.cells {
                self.set_state(cell, before);
            }
            self.cursor = step.cursor;
            self.redo.push(step);
            self.revision += 1;
        }
    }

    /// Does the last undone step again, cursor included.
    pub fn redo(&mut self) {
        if let Some(step) = self.redo.pop() {
            for &(cell, _, after) in &step.cells {
                self.set_state(cell, after);
            }
            self.cursor = step.cursor;
            self.undo.push(step);
            self.revision += 1;
        }
    }

    /// Every empty cell gets all its possible candidates.
    fn refill(&mut self) {
        for cell in 0..CELLS {
            if self.board.get(cell).is_none() {
                self.fill(cell);
            }
        }
    }

    /// Writes every candidate the clues allow in `cell`. Those the player's
    /// own digits rule out are written hidden, so they show once those digits
    /// are erased, as if they had been there before the digits were placed.
    fn fill(&mut self, cell: Cell) {
        let possible = self.board.possible(cell);
        self.candidates[cell] = Some(possible);
        self.hidden[cell].candidates = self.puzzle.clues().possible(cell).without(possible);
    }

    /// Writes the digit and hides it in the notes of every cell it sees. The
    /// cell's own notes are hidden too, since a digit covers them. A digit
    /// already there is erased first, unless it is the same digit, which
    /// changes nothing.
    fn place(&mut self, cell: Cell, digit: u8) {
        if self.board.get(cell) == Some(digit) {
            return;
        }
        self.erase(cell);
        let own = &mut self.hidden[cell];
        if let Some(set) = &mut self.candidates[cell] {
            own.candidates = own.candidates.union(std::mem::take(set));
        }
        own.marks = own.marks.union(std::mem::take(&mut self.marks[cell]));
        self.board.set(cell, Some(digit));

        let only: Digits = [digit].into_iter().collect();
        for p in peers(cell) {
            let hidden = &mut self.hidden[p];
            if let Some(set) = &mut self.candidates[p]
                && set.contains(digit)
            {
                set.remove(digit);
                hidden.candidates = hidden.candidates.union(only);
            }
            if self.marks[p].contains(digit) {
                self.marks[p].remove(digit);
                hidden.marks = hidden.marks.union(only);
            }
        }
    }

    /// Takes the player's digit out of `cell`, then shows again every hidden
    /// note, in the cell and the cells it sees, that no placed digit rules
    /// out any more.
    fn erase(&mut self, cell: Cell) {
        if self.board.get(cell).is_none() {
            return;
        }
        self.board.set(cell, None);
        for c in std::iter::once(cell).chain(peers(cell)) {
            if self.board.get(c).is_some() {
                continue;
            }
            let possible = self.board.possible(c);
            let hidden = &mut self.hidden[c];
            let candidates = hidden.candidates.intersection(possible);
            if !candidates.is_empty() {
                let set = self.candidates[c].get_or_insert(Digits::NONE);
                *set = set.union(candidates);
                hidden.candidates = hidden.candidates.without(candidates);
            }
            let marks = hidden.marks.intersection(possible);
            self.marks[c] = self.marks[c].union(marks);
            hidden.marks = hidden.marks.without(marks);
        }
    }

    /// Runs `action` and keeps whatever it changed as one undo step. An action
    /// that changes nothing leaves the history alone.
    fn record(&mut self, action: impl FnOnce(&mut Game)) {
        let before: Vec<CellState> = (0..CELLS).map(|c| self.state(c)).collect();
        action(self);
        let cells: Vec<_> = (0..CELLS)
            .filter_map(|c| {
                let after = self.state(c);
                (before[c] != after).then_some((c, before[c], after))
            })
            .collect();
        if !cells.is_empty() {
            self.undo.push(Step {
                cursor: self.cursor,
                cells,
            });
            self.redo.clear();
            self.revision += 1;
        }
    }

    fn state(&self, cell: Cell) -> CellState {
        CellState {
            digit: self.board.get(cell),
            candidates: self.candidates[cell],
            marks: self.marks[cell],
            hidden: self.hidden[cell],
        }
    }

    fn set_state(&mut self, cell: Cell, state: CellState) {
        self.board.set(cell, state.digit);
        self.candidates[cell] = state.candidates;
        self.marks[cell] = state.marks;
        self.hidden[cell] = state.hidden;
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
    use crate::fixtures::{game, solution};
    use proptest::prelude::*;
    use sumi_core::step::Step as SolverStep;

    /// r1c3 is empty; its possible digits are 1, 2 and 4, and its answer is 4.
    const R1C3: Cell = 2;
    /// r1c8 is empty and sees r1c3; its answer is 1.
    const R1C8: Cell = 7;

    fn at(cell: Cell) -> Game {
        let mut g = game();
        g.cursor = cell;
        g
    }

    fn digits(list: &[u8]) -> Digits {
        list.iter().copied().collect()
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    /// Two digits clashing, a cell no digit fits, or a cell whose
    /// candidates are all struck out is a dead end; a board without them
    /// isn't.
    #[test]
    fn a_dead_end_is_a_clash_or_a_cell_with_nothing_left() {
        let mut g = at(R1C3);
        assert!(!g.at_dead_end());
        g.press(5); // r1c1 is 5
        assert!(g.at_dead_end(), "a clash");
        g.clear();
        g.candidates[R1C3] = Some(Digits::NONE);
        assert!(g.at_dead_end(), "candidates all struck out");
        g.candidates[R1C3] = None;
        // r1c3 can be 1, 2 or 4: put each in another cell of its row,
        // where none of them clashes.
        for (cell, digit) in [(6, 1), (3, 2), (5, 4)] {
            g.move_to(cell);
            g.press(digit);
        }
        assert!(g.board().possible(R1C3).is_empty());
        assert!(!g.board().conflicts().contains(&true), "no clash to find");
        assert!(g.at_dead_end(), "no digit fits");
    }

    /// B goes back to the newest bookmark and keeps it; on it, B drops it
    /// and goes to the one before; on the oldest, B does nothing. Each
    /// going back is one undo step, cursor included.
    #[test]
    fn back_to_a_bookmark_then_walk_the_stack() {
        let mut g = at(R1C3);
        assert!(!g.back_to_bookmark(), "no bookmark");
        g.bookmark();
        g.bookmark();
        assert_eq!(g.bookmarks(), [51], "the same board twice is one bookmark");
        let first = g.here();
        g.press(4);
        g.move_to(R1C8);
        g.press(1);
        g.bookmark();
        let second = g.here();
        g.fill_candidates();

        assert!(g.back_to_bookmark());
        assert!(g.here() == second, "back to the newest");
        g.undo();
        assert!(
            !g.candidates(3).is_empty(),
            "one undo step: before going back"
        );
        g.redo();
        assert!(g.back_to_bookmark());
        assert!(g.here() == first, "on the newest, back to the one before");
        assert_eq!(g.bookmarks(), [51]);
        assert!(!g.back_to_bookmark(), "on the oldest");
        assert!(g.here() == first);
    }

    /// A digit's last move is its own cell, not the notes it hid around
    /// it; a move of notes only is every cell whose notes it changed.
    #[test]
    fn the_last_move_is_the_cells_it_wrote_in() {
        let mut g = at(R1C3);
        assert_eq!(g.last_move(), None);
        g.fill_candidates();
        let (_, filled) = g.last_move().unwrap();
        assert!(filled.len() > 40, "{filled:?}");
        g.move_to(R1C8);
        g.press(1);
        assert_eq!(g.last_move(), Some((R1C8, vec![R1C8])));
    }

    #[test]
    fn the_clock_counts_only_while_it_runs() {
        let mut g = game();
        let t0 = Instant::now();
        g.start_clock(t0);
        g.stop_clock(t0 + secs(10));
        assert_eq!(g.elapsed(t0 + secs(50)), secs(10));
        g.start_clock(t0 + secs(50));
        g.start_clock(t0 + secs(52)); // already running: no change
        assert_eq!(g.elapsed(t0 + secs(55)), secs(15));
        g.stop_clock(t0 + secs(55));
        g.stop_clock(t0 + secs(60)); // already stopped: no change
        assert_eq!(g.elapsed(t0 + secs(70)), secs(15));
    }

    /// A reveal counts when it changes a cell: not on a clue, and not on a
    /// cell that already holds its answer.
    #[test]
    fn reveals_count_when_they_change_a_cell() {
        let mut g = at(R1C3);
        g.reveal();
        assert_eq!(g.revealed(), 1);
        g.reveal();
        assert_eq!(g.revealed(), 1);
        g.cursor = 0;
        g.reveal();
        assert_eq!(g.revealed(), 1);
    }

    #[test]
    fn reveal_places_the_answer_and_undo_takes_it_back() {
        let mut g = at(R1C3);
        g.reveal();
        assert_eq!(g.board().get(R1C3), Some(4));
        g.undo();
        assert_eq!(g.board().get(R1C3), None);
    }

    #[test]
    fn reveal_replaces_a_wrong_digit_and_skips_clues_and_right_digits() {
        let mut g = at(R1C3);
        g.press(1);
        g.reveal();
        assert_eq!(g.board().get(R1C3), Some(4));

        let steps = g.undo.len();
        g.reveal();
        assert_eq!(g.undo.len(), steps, "a right digit is not changed");
        g.cursor = 0;
        g.reveal();
        assert_eq!(g.undo.len(), steps, "a clue is not changed");
    }

    #[test]
    fn wrong_digits_are_those_that_differ_from_the_solution() {
        let mut g = at(R1C3);
        g.press(1); // the answer is 4
        g.cursor = R1C8;
        g.press(1); // the answer is 1
        assert_eq!(g.wrong_digits(), [(R1C3, 1)]);
    }

    fn removal(cell: Cell, digit: u8) -> SolverStep {
        SolverStep {
            technique: &sumi_core::technique::Pointing,
            digits: digits(&[digit]),
            houses: vec![],
            cells: vec![cell],
            removals: vec![(cell, digits(&[digit]))],
            placement: None,
            key_cells: vec![],
        }
    }

    #[test]
    fn a_hint_removal_writes_the_cells_candidates_less_the_digit() {
        let mut g = game();
        g.apply_hint(&removal(R1C3, 1)); // no candidates written: 1, 2, 4 possible
        assert_eq!(g.candidates(R1C3), digits(&[2, 4]));
        g.apply_hint(&removal(R1C3, 2));
        assert_eq!(g.candidates(R1C3), digits(&[4]));
        g.undo();
        assert_eq!(g.candidates(R1C3), digits(&[2, 4]));
    }

    /// Fill writes the candidates a placed digit rules out as hidden notes, so
    /// erasing the digit shows them again. A hint's removal on a cell with no
    /// candidates written must leave the cell the same way.
    #[test]
    fn a_hint_removal_leaves_what_my_digits_rule_out_ready_to_come_back() {
        let mut g = at(R1C8);
        g.press(1); // r1c8 = 1, which r1c3 sees
        g.apply_hint(&removal(R1C3, 2)); // r1c3 had no candidates written
        assert_eq!(g.candidates(R1C3), digits(&[4]));
        g.clear(); // erases the 1
        assert_eq!(g.candidates(R1C3), digits(&[1, 4]));
    }

    #[test]
    fn a_hint_placement_is_one_undo_step_and_cleans_the_notes_around_it() {
        let mut g = game();
        g.fill_candidates();
        let Hint::Step(hint) = g.hint() else {
            panic!("the fixture has a step")
        };
        let (cell, digit) = hint
            .step
            .placement
            .expect("the fixture's first step places");
        let steps = g.undo.len();
        g.apply_hint(&hint.step);
        assert_eq!(g.board().get(cell), Some(digit));
        assert_eq!(g.undo.len(), steps + 1);
        assert!(peers(cell).all(|p| !g.candidates(p).contains(digit)));
        g.undo();
        assert_eq!(g.board().get(cell), None);
    }

    #[test]
    fn the_hint_reads_the_players_candidates() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1); // writes only 1 in r1c3, whose answer is 4
        assert_eq!(g.hint(), Hint::StruckOut(R1C3));
        g.press(4);
        assert_ne!(g.hint(), Hint::StruckOut(R1C3));
    }

    #[test]
    fn a_wrong_digit_is_the_first_hint() {
        let mut g = at(R1C3);
        g.press(1);
        assert_eq!(g.hint(), Hint::Mistake(R1C3));
    }

    proptest! {
        /// From any correct play, following hints keeps every digit right and
        /// every written candidate list holding its cell's answer.
        #[test]
        fn following_hints_never_breaks_the_solution(
            seed in any::<u64>(),
            play in prop::collection::vec((0..CELLS, 1u8..=9, 0u8..3), 0..80),
        ) {
            use rand::SeedableRng;
            let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(seed);
            let (puzzle, _) = sumi_core::generate::generate_solvable(&mut rng);
            let solution = *puzzle.solution();
            let mut g = Game::new(puzzle);
            for (cell, digit, kind) in play {
                g.cursor = cell;
                let answer = solution.get(cell).unwrap();
                match kind {
                    0 => {
                        g.mode = Mode::Digit;
                        g.press(answer);
                    }
                    1 => g.fill_candidates(),
                    _ => {
                        // Strike a wrong candidate, if the cell lists it.
                        g.mode = Mode::Candidate;
                        if digit != answer && g.candidates(cell).contains(digit) {
                            g.press(digit);
                        }
                    }
                }
            }
            for _ in 0..200 {
                match g.hint() {
                    Hint::Step(hint) => g.apply_hint(&hint.step),
                    Hint::Mistake(c) | Hint::StruckOut(c) => {
                        return Err(TestCaseError::fail(format!("mistake at {c}")));
                    }
                    Hint::Solved | Hint::Stuck => break,
                }
                for cell in 0..CELLS {
                    let answer = solution.get(cell).unwrap();
                    prop_assert!(g.board().get(cell).is_none_or(|d| d == answer));
                    if g.board().get(cell).is_none() {
                        prop_assert!(g.candidates[cell].is_none_or(|set| set.contains(answer)));
                    }
                }
            }
        }
    }

    #[test]
    fn the_revision_counts_changes_to_the_board_and_notes_only() {
        let mut g = at(R1C3);
        let start = g.revision();
        g.step(Direction::Right);
        g.next_mode();
        g.step(Direction::Left);
        assert_eq!(
            g.revision(),
            start,
            "moving and changing mode change nothing"
        );
        g.press(1); // a candidate in r1c3
        let one = g.revision();
        assert_ne!(one, start);
        g.undo();
        assert_ne!(g.revision(), one, "undo is a change");
        let undone = g.revision();
        g.redo();
        assert_ne!(g.revision(), undone, "and so is redo");
        let redone = g.revision();
        g.cursor = 0;
        g.press(5); // r1c1 is a clue: nothing changes
        assert_eq!(g.revision(), redone);
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
        let mut g = at(1); // r1c2
        g.jump(Direction::Right);
        assert_eq!(g.cursor(), 4); // r1c5
        g.jump(Direction::Down);
        assert_eq!(g.cursor(), 31); // r4c5
        g.jump(Direction::Left);
        g.jump(Direction::Left);
        assert_eq!(g.cursor(), 34); // wrapped to r4c8
    }

    #[test]
    fn modes_cycle_digit_candidate_box_mark() {
        let mut g = game();
        assert_eq!(g.mode(), Mode::Digit);
        g.next_mode();
        assert_eq!(
            (g.mode(), g.current_layer()),
            (Mode::Candidate, Layer::Candidates)
        );
        g.next_mode();
        assert_eq!(
            (g.mode(), g.current_layer()),
            (Mode::BoxMark, Layer::BoxMarks)
        );
        g.next_mode();
        assert_eq!(g.mode(), Mode::Digit);
    }

    #[test]
    fn clues_cannot_be_changed() {
        let mut g = at(0);
        g.press(1);
        g.clear();
        g.next_mode();
        g.press(1);
        assert_eq!(g.board().get(0), Some(5));
        assert_eq!(g.candidates(0), Digits::NONE);
        assert!(g.undo.is_empty());
    }

    #[test]
    fn undo_and_redo_walk_the_history() {
        let mut g = at(R1C3);
        g.press(4);
        g.press(2);
        g.clear();
        assert_eq!(g.board().get(R1C3), None);

        g.undo();
        assert_eq!(g.board().get(R1C3), Some(2));
        g.undo();
        assert_eq!(g.board().get(R1C3), Some(4));
        g.undo();
        assert_eq!(g.board().get(R1C3), None);
        g.undo(); // nothing left
        assert_eq!(g.board().get(R1C3), None);

        g.redo();
        g.redo();
        assert_eq!(g.board().get(R1C3), Some(2));
    }

    #[test]
    fn a_new_change_drops_the_redo_steps() {
        let mut g = at(R1C3);
        g.press(4);
        g.undo();
        g.press(1);
        g.redo();
        assert_eq!(g.board().get(R1C3), Some(1));
    }

    #[test]
    fn an_action_that_changes_nothing_is_not_a_step() {
        let mut g = at(R1C3);
        g.press(4);
        g.press(4);
        g.undo();
        assert_eq!(g.board().get(R1C3), None);
        assert!(g.undo.is_empty());
    }

    #[test]
    fn undo_moves_the_cursor_to_the_change() {
        let mut g = at(R1C3);
        g.press(4);
        g.step(Direction::Down);
        g.undo();
        assert_eq!(g.cursor(), R1C3);
    }

    #[test]
    fn candidate_and_box_mark_modes_toggle_notes() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.press(4);
        g.press(1);
        assert_eq!(g.candidates(R1C3), digits(&[4]));
        g.next_mode();
        g.press(2);
        assert_eq!(g.marks(R1C3), digits(&[2]));
        assert_eq!(g.board().get(R1C3), None);
        g.undo();
        assert_eq!(g.marks(R1C3), Digits::NONE);
    }

    #[test]
    fn placing_a_digit_clears_it_from_the_notes_around_it() {
        let mut g = game();
        g.fill_candidates();
        g.next_mode();
        g.next_mode();
        g.cursor = 3; // r1c4, in the same row as r1c3
        g.press(4);
        g.mode = Mode::Digit;
        g.cursor = R1C3;
        assert!(g.candidates(R1C3).contains(4));
        g.press(4);

        assert_eq!(g.candidates(R1C3), Digits::NONE);
        assert!(!g.marks(3).contains(4));
        for p in peers(R1C3) {
            assert!(!g.candidates(p).contains(4), "cell {p}");
        }

        g.undo();
        assert_eq!(g.board().get(R1C3), None);
        assert_eq!(g.candidates(R1C3), digits(&[1, 2, 4]));
        assert!(g.marks(3).contains(4));
    }

    #[test]
    fn erasing_a_digit_gives_back_the_notes_its_placement_removed() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.mode = Mode::BoxMark;
        g.cursor = 5; // r1c6
        g.press(4);
        g.mode = Mode::Digit;
        g.cursor = R1C3;
        let before = snapshot(&g);

        g.press(4);
        g.clear();
        assert_eq!(snapshot(&g), before);
    }

    #[test]
    fn overwriting_a_digit_gives_back_the_old_one_and_takes_the_new() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.press(4);
        g.press(2);
        assert!(g.candidates(5).contains(4)); // r1c6
        assert!(!g.candidates(5).contains(2));
        assert_eq!(g.board().get(R1C3), Some(2));
    }

    #[test]
    fn erasing_gives_back_only_notes_that_are_still_possible() {
        let mut g = game();
        g.fill_candidates();
        g.mode = Mode::Candidate;
        g.cursor = R1C8;
        g.press(4); // the player takes 4 out of r1c8 by hand
        g.mode = Mode::Digit;
        g.cursor = R1C3;
        g.press(4);
        g.cursor = 8; // r1c9, which lost its 4 to r1c3
        g.clear(); // the player clears its candidates
        g.cursor = 21; // r3c4, which sees r1c6 but not r1c3 or r1c7
        g.press(4);

        g.cursor = R1C3;
        g.clear();
        assert!(g.candidates(6).contains(4)); // r1c7: given back
        assert!(!g.candidates(5).contains(4)); // r1c6: the 4 at r3c4 rules it out
        assert!(!g.candidates(R1C8).contains(4)); // placing took nothing there
        assert_eq!(g.candidates[8], None);
        assert_eq!(g.candidates(R1C3), digits(&[1, 2, 4]));
    }

    /// r1c3 takes 1 from r1c8, then r1c8 takes a digit itself. Erased in
    /// the order placed, the 1 still comes back to r1c8.
    #[test]
    fn erasing_out_of_order_still_gives_every_note_back() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.press(1);
        g.cursor = R1C8;
        g.press(2);
        g.cursor = R1C3;
        g.clear();
        g.cursor = R1C8;
        g.clear();
        assert_eq!(g.candidates(R1C8), digits(&[1, 2, 4, 9]));
    }

    /// Two clashing 4s both rule 4 out of r1c6; it comes back once both go,
    /// whichever goes first.
    #[test]
    fn a_note_two_digits_rule_out_comes_back_when_both_are_erased() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.press(4);
        g.cursor = 23; // r3c6, which sees r1c6
        g.press(4);
        g.cursor = R1C3;
        g.clear();
        assert!(!g.candidates(5).contains(4));
        g.cursor = 23;
        g.clear();
        assert!(g.candidates(5).contains(4));
    }

    /// Notes cleared after a digit was placed stay cleared when it is erased.
    #[test]
    fn erasing_gives_nothing_back_to_notes_cleared_since() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.mode = Mode::BoxMark;
        g.cursor = 5; // r1c6
        g.press(4);
        g.mode = Mode::Digit;
        g.cursor = R1C3;
        g.press(4);
        g.clear_notes(None);
        g.clear();
        assert_eq!(g.marks(5), Digits::NONE);
        assert_eq!(g.candidates[R1C3], None);
        assert_eq!(g.candidates[5], None);

        let mut g = at(R1C3);
        g.mode = Mode::BoxMark;
        g.cursor = 5;
        g.press(4);
        g.mode = Mode::Digit;
        g.cursor = R1C3;
        g.press(4);
        g.marks_to_candidates(); // clears every box mark
        g.clear();
        assert_eq!(g.marks(5), Digits::NONE);
    }

    /// A wrong 1 in r1c3 keeps 1 out of r1c8 when candidates are filled;
    /// erasing it brings 1 back to r1c8, but not where it was taken out by
    /// hand.
    #[test]
    fn filled_candidates_a_wrong_digit_kept_out_come_back_when_it_goes() {
        for fill in [
            Game::fill_candidates,
            Game::refill_candidates,
            Game::marks_to_candidates,
        ] {
            let mut g = at(R1C3);
            g.press(1);
            fill(&mut g);
            assert_eq!(g.candidates(R1C8), digits(&[2, 4, 9]));
            g.mode = Mode::Candidate;
            g.cursor = 6; // r1c7, possible {4, 8, 9}
            g.press(9);
            g.mode = Mode::Digit;
            g.cursor = R1C3;
            g.clear();
            assert_eq!(g.candidates(R1C8), digits(&[1, 2, 4, 9]));
            assert_eq!(g.candidates(6), digits(&[1, 4, 8]));
        }
    }

    #[test]
    fn fill_only_touches_cells_without_candidates() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.fill_candidates();
        assert_eq!(g.candidates(R1C3), digits(&[1]));
        assert_eq!(g.candidates(3), g.board().possible(3));
        assert_eq!(g.candidates(0), Digits::NONE); // a clue

        g.refill_candidates();
        assert_eq!(g.candidates(R1C3), digits(&[1, 2, 4]));

        g.undo();
        g.undo();
        assert_eq!(g.candidates(R1C3), digits(&[1]));
        assert_eq!(g.candidates(3), Digits::NONE);
    }

    #[test]
    fn moving_from_box_marks_fills_full_candidates_and_clears_the_marks() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.next_mode();
        g.press(2);
        g.cursor = 5; // r1c6, with box marks and no candidates
        g.press(8);
        g.marks_to_candidates();
        assert_eq!(g.candidates(R1C3), digits(&[1, 2, 4]));
        assert_eq!(g.candidates(5), digits(&[2, 4, 6, 8]));
        assert_eq!(g.candidates(3), digits(&[2, 6])); // r1c4, which had no notes
        assert_eq!(g.marks(R1C3), Digits::NONE);
        assert_eq!(g.marks(5), Digits::NONE);
        g.undo();
        assert_eq!(g.candidates(R1C3), digits(&[1]));
        assert_eq!(g.marks(R1C3), digits(&[2]));
    }

    #[test]
    fn box_marks_can_merge_into_candidates_instead() {
        let mut g = at(R1C3);
        g.set_candidates_from_marks(CandidatesFromMarks::Merge);
        g.next_mode();
        g.press(1);
        g.next_mode();
        g.press(2);
        g.marks_to_candidates();
        assert_eq!(g.candidates(R1C3), digits(&[1, 2]));
        assert_eq!(g.candidates(3), Digits::NONE);
        assert_eq!(g.marks(R1C3), Digits::NONE);
        g.undo();
        assert_eq!(g.marks(R1C3), digits(&[2]));
    }

    #[test]
    fn clearing_notes_by_layer_or_all() {
        let mut g = at(R1C3);
        g.fill_candidates();
        g.next_mode();
        g.next_mode();
        g.press(2);

        g.clear_notes(Some(Layer::Candidates));
        assert!(g.candidates(3).is_empty());
        assert_eq!(g.marks(R1C3), digits(&[2]));

        g.undo();
        g.clear_notes(None);
        assert!(g.candidates(3).is_empty());
        assert!(g.marks(R1C3).is_empty());

        g.undo(); // one step brings both layers back
        assert!(!g.candidates(3).is_empty());
        assert_eq!(g.marks(R1C3), digits(&[2]));
    }

    #[test]
    fn clear_takes_the_digit_first_then_the_notes() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.mode = Mode::Digit;
        g.press(4);
        g.clear();
        assert_eq!(g.board().get(R1C3), None);
        // Placing 4 cleared the cell's own notes; undo the clear and the place
        // to see them come back, then clear removes them.
        g.undo();
        g.undo();
        assert_eq!(g.candidates(R1C3), digits(&[1]));
        g.clear();
        assert_eq!(g.candidates(R1C3), Digits::NONE);
    }

    #[test]
    fn working_candidates_fall_back_to_what_is_possible() {
        let mut g = at(R1C3);
        assert_eq!(g.working_candidates(R1C3), digits(&[1, 2, 4]));
        g.next_mode();
        g.press(2);
        assert_eq!(g.working_candidates(R1C3), digits(&[2]));
        assert_eq!(g.working_candidates(0), Digits::NONE);
    }

    #[test]
    fn notes_clash_with_digits_the_cell_sees() {
        let g = game();
        assert!(g.note_clashes(R1C3, 5)); // the 5 at r1c1
        assert!(!g.note_clashes(R1C3, 4));
    }

    #[test]
    fn a_digit_is_complete_once_all_nine_are_placed() {
        let mut g = game();
        assert!(!g.is_complete(5));
        let solution = solution();
        for cell in 0..CELLS {
            if solution.get(cell) == Some(5) {
                g.cursor = cell;
                g.press(5);
            }
        }
        assert!(g.is_complete(5));
        assert!(!g.is_complete(4));
    }

    #[test]
    fn filling_in_the_solution_wins() {
        let mut g = game();
        let solution = solution();
        assert!(!g.is_won());
        for cell in 0..CELLS {
            g.cursor = cell;
            g.press(solution.get(cell).unwrap());
        }
        assert!(g.is_won());
    }

    #[test]
    fn redo_puts_the_cursor_back_too() {
        let mut g = at(R1C3);
        g.press(4);
        g.undo();
        g.step(Direction::Down);
        g.redo();
        assert_eq!(g.cursor(), R1C3);
    }

    #[test]
    fn clear_removes_box_marks_as_well_as_candidates() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.next_mode();
        g.press(2);
        g.clear();
        assert_eq!(g.candidates(R1C3), Digits::NONE);
        assert_eq!(g.marks(R1C3), Digits::NONE);
    }

    #[test]
    fn placing_a_digit_leaves_cells_it_does_not_see_alone() {
        let mut g = at(R1C3);
        g.fill_candidates();
        let far = 15; // r2c7: shares no row, column or box with r1c3
        assert!(g.candidates(far).contains(4));
        g.press(4);
        assert!(g.candidates(far).contains(4));
    }

    #[test]
    fn a_cell_whose_candidates_are_all_ruled_out_shows_none() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1); // r1c3's only candidate is now 1
        g.mode = Mode::Digit;
        g.cursor = R1C8;
        g.press(1); // rules 1 out of r1c3
        assert_eq!(g.working_candidates(R1C3), Digits::NONE);

        // Fill leaves it showing the contradiction; refill starts it over.
        g.fill_candidates();
        assert_eq!(g.working_candidates(R1C3), Digits::NONE);
        g.refill_candidates();
        assert_eq!(g.working_candidates(R1C3), digits(&[2, 4]));
    }

    #[test]
    fn taking_out_your_own_last_candidate_means_no_notes() {
        let mut g = at(R1C3);
        g.next_mode();
        g.press(1);
        g.press(1);
        assert_eq!(g.working_candidates(R1C3), digits(&[1, 2, 4]));
    }

    #[test]
    fn nine_copies_with_a_clash_are_not_complete() {
        let mut g = game();
        let solution = solution();
        // Eight of the missing 5s in their right places, the ninth in r1c3,
        // where it clashes with the clue 5 at r1c1.
        let missing: Vec<Cell> = (0..CELLS)
            .filter(|&c| solution.get(c) == Some(5) && !g.is_clue(c))
            .collect();
        for &cell in &missing[1..] {
            g.cursor = cell;
            g.press(5);
        }
        g.cursor = R1C3;
        g.press(5);
        assert_eq!(
            (0..CELLS).filter(|&c| g.board().get(c) == Some(5)).count(),
            9
        );
        assert!(!g.is_complete(5));
    }

    /// Everything the player can change, for comparing whole games.
    fn snapshot(g: &Game) -> Vec<CellState> {
        (0..CELLS).map(|c| g.state(c)).collect()
    }

    #[derive(Debug, Clone)]
    enum Op {
        Step(Direction),
        Jump(Direction),
        Press(u8),
        Clear,
        NextMode,
        Fill,
        Refill,
        MarksToCandidates,
        ClearLayer,
        ClearAll,
    }

    fn op() -> impl Strategy<Value = Op> {
        let direction = prop_oneof![
            Just(Direction::Up),
            Just(Direction::Down),
            Just(Direction::Left),
            Just(Direction::Right),
        ];
        prop_oneof![
            4 => direction.clone().prop_map(Op::Step),
            1 => direction.prop_map(Op::Jump),
            4 => (1u8..=9).prop_map(Op::Press),
            1 => Just(Op::Clear),
            2 => Just(Op::NextMode),
            1 => Just(Op::Fill),
            1 => Just(Op::Refill),
            1 => Just(Op::MarksToCandidates),
            1 => Just(Op::ClearLayer),
            1 => Just(Op::ClearAll),
        ]
    }

    fn apply(g: &mut Game, op: &Op) {
        match *op {
            Op::Step(d) => g.step(d),
            Op::Jump(d) => g.jump(d),
            Op::Press(d) => g.press(d),
            Op::Clear => g.clear(),
            Op::NextMode => g.next_mode(),
            Op::Fill => g.fill_candidates(),
            Op::Refill => g.refill_candidates(),
            Op::MarksToCandidates => g.marks_to_candidates(),
            Op::ClearLayer => g.clear_notes(Some(g.current_layer())),
            Op::ClearAll => g.clear_notes(None),
        }
    }

    proptest! {
        #[test]
        fn undoing_everything_restores_the_start_and_redo_the_end(
            ops in prop::collection::vec(op(), 0..60)
        ) {
            let mut g = game();
            let start = snapshot(&g);
            for op in &ops {
                apply(&mut g, op);
            }
            let end = snapshot(&g);

            while !g.undo.is_empty() {
                g.undo();
            }
            prop_assert_eq!(snapshot(&g), start);
            while !g.redo.is_empty() {
                g.redo();
            }
            prop_assert_eq!(snapshot(&g), end);
        }

        #[test]
        fn erasing_placed_digits_in_any_order_gives_back_every_note_still_possible(
            ops in prop::collection::vec(op(), 0..60),
            placed in prop::collection::vec((0..CELLS, 1u8..=9), 1..6),
            order in Just((0..6).collect::<Vec<usize>>()).prop_shuffle(),
        ) {
            let mut g = game();
            for op in &ops {
                apply(&mut g, op);
            }
            // Each cell once: clearing a cell a second time clears its notes.
            let mut placed: Vec<(Cell, u8)> = placed
                .into_iter()
                .filter(|&(c, _)| g.board().get(c).is_none())
                .collect();
            placed.sort_by_key(|&(c, _)| c);
            placed.dedup_by_key(|&mut (c, _)| c);
            prop_assume!(!placed.is_empty());
            let before = snapshot(&g);

            g.mode = Mode::Digit;
            for &(cell, digit) in &placed {
                g.cursor = cell;
                g.press(digit);
            }
            for &i in order.iter().filter(|&&i| i < placed.len()) {
                g.cursor = placed[i].0;
                g.clear();
            }
            let after = snapshot(&g);

            for cell in 0..CELLS {
                let (was, now) = (before[cell], after[cell]);
                let ruled_out = Digits::ALL.without(g.board().possible(cell));
                let possible = |set: Digits| set.without(ruled_out);
                prop_assert_eq!(now.digit, was.digit);
                prop_assert_eq!(now.candidates.map(possible), was.candidates.map(possible));
                prop_assert_eq!(possible(now.marks), possible(was.marks));
                // Nothing comes back that wasn't there.
                prop_assert_eq!(now.candidates.is_some(), was.candidates.is_some());
                prop_assert!(now.candidates.unwrap_or_default().without(was.candidates.unwrap_or_default()).is_empty());
                prop_assert!(now.marks.without(was.marks).is_empty());
            }
        }
    }
}
