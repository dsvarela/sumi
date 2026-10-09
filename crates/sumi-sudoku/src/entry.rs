//! Typing in a puzzle: its clues first, then any digits already solved on
//! paper. Drawing is in `ui`.

use sumi_core::grade::grade;
use sumi_core::grid::{CELLS, Cell, Grid};
use sumi_core::puzzle::{NotUnique, Puzzle};
use sumi_core::technique::Level;

use crate::game::{Game, moved};
use crate::input::Action;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub clues: Grid,
    /// The player's digits, in the second stage; never on a clue.
    pub digits: Grid,
    pub cursor: Cell,
    pub stage: Stage,
    /// What the last check, paste or start found.
    pub verdict: Option<Verdict>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Clues,
    /// The clues have one solution, of this level; the player adds the
    /// digits they have.
    Digits(Level),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    NoSolution,
    Several,
    /// Pasted text held this many cells, not 81.
    WrongCount(usize),
    /// Pasted text held this, which is no cell and no border.
    BadCharacter(char),
    /// Pasted digits that differ from a clue.
    ChangesAClue,
    /// The clues and digits fill the board: there is nothing to play.
    AlreadySolved,
}

/// What a key on the entry screen asks of the app.
pub enum Outcome {
    Stay,
    /// Esc from the clues: back to the menu.
    Back,
    /// Enter on the digits: play this game.
    Play(Box<Game>),
}

/// The 81 cells `text` holds, passing over spaces, lines and box borders.
fn read_cells(text: &str) -> Result<Grid, Verdict> {
    let border = |c: char| c.is_whitespace() || "|+-=:".contains(c) || ('─'..='╿').contains(&c);
    let mut cells = String::new();
    for c in text.chars().filter(|&c| !border(c)) {
        match c {
            '0'..='9' | '.' => cells.push(c),
            _ => return Err(Verdict::BadCharacter(c)),
        }
    }
    match cells.len() {
        CELLS => Ok(cells.parse().expect("81 digits and dots")),
        n => Err(Verdict::WrongCount(n)),
    }
}

impl Entry {
    /// The clues and the digits together.
    pub fn board(&self) -> Grid {
        let mut board = self.clues;
        for cell in 0..CELLS {
            if let Some(digit) = self.digits.get(cell) {
                board.set(cell, Some(digit));
            }
        }
        board
    }

    pub fn new() -> Entry {
        Entry {
            clues: Grid::EMPTY,
            digits: Grid::EMPTY,
            cursor: 0,
            stage: Stage::Clues,
            verdict: None,
        }
    }

    pub fn handle(&mut self, action: Action) -> Outcome {
        match action {
            Action::Step(direction) => self.cursor = moved(self.cursor, direction, 1),
            Action::Jump(direction) => self.cursor = moved(self.cursor, direction, 3),
            Action::Digit(digit) => {
                // Typing moves on in reading order, as on paper.
                if self.write(Some(digit)) {
                    self.cursor = (self.cursor + 1) % CELLS;
                }
            }
            Action::Clear => {
                self.write(None);
            }
            Action::Confirm => return self.confirm(),
            Action::Close => match self.stage {
                Stage::Clues => return Outcome::Back,
                Stage::Digits(_) => {
                    self.stage = Stage::Clues;
                    self.verdict = None;
                }
            },
            _ => {}
        }
        Outcome::Stay
    }

    /// Reads pasted text as 81 cells, `0` or `.` for an empty one, as the
    /// clues or, in the second stage, the player's digits; there the clues
    /// may be pasted along with them. Spaces, lines and the borders other
    /// programs draw between boxes are passed over.
    pub fn paste(&mut self, text: &str) {
        let grid = match read_cells(text) {
            Ok(grid) => grid,
            Err(verdict) => {
                self.verdict = Some(verdict);
                return;
            }
        };
        match self.stage {
            Stage::Clues => {
                self.clues = grid;
                self.digits = Grid::EMPTY;
                self.verdict = None;
            }
            Stage::Digits(_) => {
                let clue_changed = (0..CELLS).any(|c| {
                    self.clues.get(c).is_some()
                        && grid.get(c).is_some()
                        && grid.get(c) != self.clues.get(c)
                });
                if clue_changed {
                    self.verdict = Some(Verdict::ChangesAClue);
                    return;
                }
                for c in (0..CELLS).filter(|&c| self.clues.get(c).is_none()) {
                    self.digits.set(c, grid.get(c));
                }
                self.verdict = None;
            }
        }
    }

    /// Writes at the cursor: a clue, which takes the place of any digit
    /// there, or a digit in a cell without a clue. A changed clue makes
    /// the clues unchecked again. Says whether anything was written.
    fn write(&mut self, digit: Option<u8>) -> bool {
        self.verdict = None;
        match self.stage {
            Stage::Clues => {
                self.clues.set(self.cursor, digit);
                self.digits.set(self.cursor, None);
                true
            }
            Stage::Digits(_) if self.clues.get(self.cursor).is_none() => {
                self.digits.set(self.cursor, digit);
                true
            }
            Stage::Digits(_) => false,
        }
    }

    /// Checks the clues, moving on to the digits if they make a puzzle,
    /// or starts the game from the digits.
    fn confirm(&mut self) -> Outcome {
        let puzzle = match Puzzle::new(self.clues) {
            Ok(puzzle) => puzzle,
            Err(err) => {
                self.stage = Stage::Clues;
                self.verdict = Some(match err {
                    NotUnique::NoSolution => Verdict::NoSolution,
                    NotUnique::Several => Verdict::Several,
                });
                return Outcome::Stay;
            }
        };
        match self.stage {
            Stage::Clues => {
                let level = grade(puzzle.clues()).expect("one solution").level;
                self.stage = Stage::Digits(level);
                self.verdict = None;
                Outcome::Stay
            }
            Stage::Digits(_) if (0..CELLS).all(|c| self.board().get(c).is_some()) => {
                self.verdict = Some(Verdict::AlreadySolved);
                Outcome::Stay
            }
            Stage::Digits(level) => Outcome::Play(Box::new(
                Game::new(puzzle).rated(level).starting_with(&self.digits),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{CLUES, SOLUTION};
    use crate::game::Direction;

    fn typed(text: &str) -> Entry {
        let mut entry = Entry::new();
        entry.paste(text);
        entry
    }

    /// Enter checks the clues: none or several solutions say so and stay;
    /// one moves on to the digits, with the level.
    #[test]
    fn enter_checks_the_clues() {
        let mut entry = typed(CLUES);
        entry.clues.set(2, Some(5)); // beside the 5 in r1c1
        assert!(matches!(entry.handle(Action::Confirm), Outcome::Stay));
        assert_eq!(
            (entry.stage, entry.verdict),
            (Stage::Clues, Some(Verdict::NoSolution))
        );
        entry.cursor = 2;
        entry.handle(Action::Clear);
        assert_eq!(entry.verdict, None, "a change clears the verdict");
        let mut entry = typed(&format!("{}{}", ".".repeat(20), &CLUES[20..]));
        entry.handle(Action::Confirm);
        assert_eq!(entry.verdict, Some(Verdict::Several));
        let mut entry = typed(CLUES);
        entry.handle(Action::Confirm);
        assert_eq!(
            (entry.stage, entry.verdict),
            (Stage::Digits(Level::Easy), None)
        );
    }

    /// Keys type at the cursor: clues first, then digits only where no clue
    /// is; Esc goes back from the digits to the clues, and leaves from them.
    #[test]
    fn keys_type_clues_then_digits() {
        let mut entry = Entry::new();
        entry.handle(Action::Digit(5));
        entry.handle(Action::Digit(3));
        assert_eq!(entry.cursor, 2, "typing moves on");
        entry.handle(Action::Clear);
        assert_eq!(entry.cursor, 2, "clearing doesn't");
        entry.handle(Action::Jump(Direction::Down));
        entry.handle(Action::Digit(9));
        let at = |cell| entry.clues.get(cell);
        assert_eq!((at(0), at(1), at(29)), (Some(5), Some(3), Some(9)));
        let mut entry = typed(CLUES);
        entry.handle(Action::Confirm);
        entry.handle(Action::Digit(4)); // r1c1 is a clue
        assert_eq!(entry.digits, Grid::EMPTY);
        entry.handle(Action::Step(Direction::Right));
        entry.handle(Action::Step(Direction::Right));
        entry.handle(Action::Digit(4));
        assert_eq!(entry.digits.get(2), Some(4));
        assert!(matches!(entry.handle(Action::Close), Outcome::Stay));
        assert_eq!(entry.stage, Stage::Clues);
        entry.cursor = 2;
        entry.handle(Action::Digit(7));
        assert_eq!(entry.digits.get(2), None, "a clue takes the digit's place");
        assert!(matches!(entry.handle(Action::Close), Outcome::Back));
    }

    /// Enter on the digits starts the game with them written, as no undo
    /// step.
    #[test]
    fn the_game_starts_with_the_digits_written() {
        let mut entry = typed(CLUES);
        entry.handle(Action::Confirm);
        entry.cursor = 2;
        entry.handle(Action::Digit(1)); // wrong: the answer is 4
        let Outcome::Play(mut game) = entry.handle(Action::Confirm) else {
            panic!("a game")
        };
        assert_eq!(game.level(), Some(Level::Easy));
        assert_eq!(game.board().get(2), Some(1));
        assert!(!game.is_clue(2));
        game.undo();
        assert_eq!(game.board().get(2), Some(1));
        assert_eq!(game.wrong_digits(), [(2, 1)]);
    }

    /// Pasted text is the clues, or in the second stage the digits, with
    /// or without the clues; it must be 81 cells and keep the clues.
    #[test]
    fn pasted_text_fills_the_board() {
        let entry = typed(&format!("{CLUES}\n"));
        assert_eq!(entry.clues.to_string(), CLUES);
        assert_eq!(typed("53..7").verdict, Some(Verdict::WrongCount(5)));
        assert_eq!(
            typed(&CLUES.replacen('.', "x", 1)).verdict,
            Some(Verdict::BadCharacter('x'))
        );
        let bordered: String = CLUES
            .as_bytes()
            .chunks(9)
            .map(|row| {
                let row = std::str::from_utf8(row).unwrap();
                format!("|{}|{}|{}|\n", &row[..3], &row[3..6], &row[6..])
            })
            .collect();
        let pasted = typed(&format!("+---+---+---+\n{bordered}"));
        assert_eq!(pasted.clues.to_string(), CLUES);
        let mut entry = typed(CLUES);
        entry.handle(Action::Confirm);
        entry.paste(SOLUTION);
        assert_eq!(entry.digits.filled(), 81 - entry.clues.filled());
        assert_eq!(entry.digits.get(0), None, "no digit on a clue");
        assert!(matches!(entry.handle(Action::Confirm), Outcome::Stay));
        assert_eq!(
            entry.verdict,
            Some(Verdict::AlreadySolved),
            "nothing to play"
        );
        entry.paste(&format!("4{}", &SOLUTION[1..]));
        assert_eq!(entry.verdict, Some(Verdict::ChangesAClue));
        entry.handle(Action::Close);
        entry.paste(CLUES);
        assert_eq!(entry.digits, Grid::EMPTY, "new clues, no digits");
    }
}
