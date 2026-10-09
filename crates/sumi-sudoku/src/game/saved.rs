//! A game as JSON, for the autosave: everything the player can't get back
//! by playing, the undo history and hidden notes included. Settings are
//! left out, since config.toml holds them, and so is the clock's state: a
//! saved game comes back paused.
//!
//! A saved game is read as untrusted text. Its clues must prove one
//! solution again, and no cell, digit or step may break what a game holds,
//! such as a step that changes a clue.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sumi_core::digits::Digits;
use sumi_core::grid::{CELLS, Cell, Grid};
use sumi_core::puzzle::{NotUnique, Puzzle};

use super::{Bookmark, CellState, Game, Hidden, Mode, Step};
use crate::store;

/// Bumped when a saved game an older sumi wrote can't be read as it is.
const VERSION: u32 = 1;

impl Game {
    /// The game as it is at `now`, as JSON.
    pub fn to_json(&self, now: Instant) -> String {
        let saved = Saved {
            version: VERSION,
            clues: self.puzzle.clues().to_string(),
            level: self.level.map(store::level_key).map(str::to_string),
            board: self.board.to_string(),
            candidates: self.candidates.iter().map(|c| c.map(text)).collect(),
            marks: self.marks.iter().map(|&m| text(m)).collect(),
            hidden: self
                .hidden
                .iter()
                .map(|h| (text(h.candidates), text(h.marks)))
                .collect(),
            cursor: self.cursor,
            mode: mode_name(self.mode).to_string(),
            undo: self.undo.iter().map(SavedStep::from).collect(),
            redo: self.redo.iter().map(SavedStep::from).collect(),
            bookmarks: self
                .bookmarks
                .iter()
                .map(|b| SavedBookmark {
                    cursor: b.cursor,
                    cells: b.cells.iter().map(saved_cell).collect(),
                })
                .collect(),
            hints_used: self.hints_used,
            hint_techniques: self.hint_techniques.clone(),
            revealed: self.revealed,
            played_ms: self.elapsed(now).as_millis() as u64,
        };
        serde_json::to_string(&saved).expect("a game always makes JSON")
    }

    /// The game `json` holds, with its clock stopped, or what's wrong with
    /// it. Settings are the defaults until the caller applies its own.
    pub fn from_json(json: &str) -> Result<Game, String> {
        // The version first: a newer sumi's save may hold fields this one
        // doesn't know, and should say so rather than name a field.
        let value: serde_json::Value = serde_json::from_str(json).map_err(|err| err.to_string())?;
        let version = value.get("version").and_then(serde_json::Value::as_u64);
        if version != Some(u64::from(VERSION)) {
            return Err(format!(
                "saved by another version of sumi ({})",
                version.map_or("none".to_string(), |v| v.to_string())
            ));
        }
        let saved: Saved = serde_json::from_value(value).map_err(|err| err.to_string())?;
        let clues: Grid = saved.clues.parse().map_err(|_| "the clues aren't a grid")?;
        let puzzle = Puzzle::new(clues).map_err(|err| match err {
            NotUnique::NoSolution => "the clues have no solution",
            NotUnique::Several => "the clues have more than one solution",
        })?;
        let mut game = Game::new(puzzle);
        game.level = match saved.level {
            Some(name) => Some(store::level_from_key(&name).ok_or("an unknown level")?),
            None => None,
        };
        game.board = saved.board.parse().map_err(|_| "the board isn't a grid")?;
        if (0..CELLS).any(|c| clues.get(c).is_some() && game.board.get(c) != clues.get(c)) {
            return Err("the board changes a clue".to_string());
        }
        game.candidates = cells(saved.candidates, |c| c.as_deref().map(digits).transpose())?;
        game.marks = cells(saved.marks, |m| digits(m))?;
        game.hidden = cells(saved.hidden, |(c, m)| {
            Ok(Hidden {
                candidates: digits(c)?,
                marks: digits(m)?,
            })
        })?;
        game.cursor = cell(saved.cursor)?;
        game.mode = mode_named(&saved.mode).ok_or("an unknown mode")?;
        let steps = |saved: Vec<SavedStep>| -> Result<Vec<Step>, String> {
            saved.into_iter().map(|s| s.into_step(&clues)).collect()
        };
        game.undo = steps(saved.undo)?;
        game.redo = steps(saved.redo)?;
        game.bookmarks = saved
            .bookmarks
            .into_iter()
            .map(|b| b.into_bookmark(&clues))
            .collect::<Result<_, _>>()?;
        game.hints_used = saved.hints_used;
        game.hint_techniques = saved.hint_techniques;
        game.revealed = saved.revealed;
        game.banked = Duration::from_millis(saved.played_ms);
        if game.is_won() {
            return Err("the game is already won".to_string());
        }
        Ok(game)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    version: u32,
    clues: String,
    level: Option<String>,
    board: String,
    /// Per cell; `None` is a cell with no candidates written.
    candidates: Vec<Option<String>>,
    marks: Vec<String>,
    /// Per cell: hidden candidates, then hidden box marks.
    hidden: Vec<(String, String)>,
    cursor: Cell,
    mode: String,
    undo: Vec<SavedStep>,
    redo: Vec<SavedStep>,
    #[serde(default)]
    bookmarks: Vec<SavedBookmark>,
    hints_used: u32,
    #[serde(default)]
    hint_techniques: BTreeMap<String, u32>,
    revealed: u32,
    played_ms: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedStep {
    cursor: Cell,
    /// Each cell the step changed, then its state before and after.
    cells: Vec<(Cell, SavedCell, SavedCell)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedBookmark {
    cursor: Cell,
    /// Every cell, in order.
    cells: Vec<SavedCell>,
}

/// A cell's digit (0 for none), candidates (`None` for none written),
/// box marks, hidden candidates and hidden box marks.
type SavedCell = (u8, Option<String>, String, String, String);

fn saved_cell(s: &CellState) -> SavedCell {
    (
        s.digit.unwrap_or(0),
        s.candidates.map(text),
        text(s.marks),
        text(s.hidden.candidates),
        text(s.hidden.marks),
    )
}

fn cell_state(
    (digit, candidates, marks, hidden, hidden_marks): &SavedCell,
) -> Result<CellState, String> {
    Ok(CellState {
        digit: match *digit {
            0 => None,
            1..=9 => Some(*digit),
            _ => return Err(format!("{digit} isn't a digit")),
        },
        candidates: candidates.as_deref().map(digits).transpose()?,
        marks: digits(marks)?,
        hidden: Hidden {
            candidates: digits(hidden)?,
            marks: digits(hidden_marks)?,
        },
    })
}

impl From<&Step> for SavedStep {
    fn from(step: &Step) -> SavedStep {
        SavedStep {
            cursor: step.cursor,
            cells: step
                .cells
                .iter()
                .map(|(c, before, after)| (*c, saved_cell(before), saved_cell(after)))
                .collect(),
        }
    }
}

impl SavedBookmark {
    fn into_bookmark(self, clues: &Grid) -> Result<Bookmark, String> {
        let cells = cells(self.cells, cell_state)?;
        if (0..CELLS).any(|c| clues.get(c).is_some() && cells[c].digit != clues.get(c)) {
            return Err("a bookmark changes a clue".to_string());
        }
        Ok(Bookmark {
            cursor: cell(self.cursor)?,
            cells,
        })
    }
}

impl SavedStep {
    fn into_step(self, clues: &Grid) -> Result<Step, String> {
        let mut cells = Vec::new();
        for (c, before, after) in self.cells {
            let c = cell(c)?;
            if clues.get(c).is_some() {
                return Err("a step changes a clue".to_string());
            }
            cells.push((c, cell_state(&before)?, cell_state(&after)?));
        }
        Ok(Step {
            cursor: cell(self.cursor)?,
            cells,
        })
    }
}

/// One value per cell, each read by `read`.
fn cells<T: Copy + Default, S>(
    saved: Vec<S>,
    read: impl Fn(&S) -> Result<T, String>,
) -> Result<[T; CELLS], String> {
    if saved.len() != CELLS {
        return Err(format!("{} cells, not {CELLS}", saved.len()));
    }
    let mut out = [T::default(); CELLS];
    for (slot, s) in out.iter_mut().zip(&saved) {
        *slot = read(s)?;
    }
    Ok(out)
}

fn cell(c: Cell) -> Result<Cell, String> {
    (c < CELLS)
        .then_some(c)
        .ok_or_else(|| format!("{c} isn't a cell"))
}

/// The digits in a set, smallest first, as "147".
fn text(set: Digits) -> String {
    set.iter().map(|d| char::from(b'0' + d)).collect()
}

fn digits(text: &str) -> Result<Digits, String> {
    text.chars()
        .map(|c| match c {
            '1'..='9' => Ok(c as u8 - b'0'),
            _ => Err(format!("{c:?} isn't a digit")),
        })
        .collect()
}

const MODES: [(Mode, &str); 3] = [
    (Mode::Digit, "digit"),
    (Mode::Candidate, "candidate"),
    (Mode::BoxMark, "box mark"),
];

fn mode_name(mode: Mode) -> &'static str {
    MODES
        .iter()
        .find(|(m, _)| *m == mode)
        .expect("every mode")
        .1
}

fn mode_named(name: &str) -> Option<Mode> {
    MODES.iter().find(|(_, n)| *n == name).map(|(m, _)| *m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{CLUES, game};
    use proptest::prelude::*;
    use sumi_core::hint::Hint;
    use sumi_core::technique::Level;

    /// Everything a saved game keeps, as one comparable value.
    fn kept(g: &Game) -> impl PartialEq + std::fmt::Debug {
        let cells: Vec<_> = (0..CELLS).map(|c| g.state(c)).collect();
        let steps = |steps: &[Step]| -> Vec<_> {
            steps.iter().map(|s| (s.cursor, s.cells.clone())).collect()
        };
        (
            cells,
            (g.cursor, g.mode, g.level),
            steps(&g.undo),
            steps(&g.redo),
            g.bookmarks
                .iter()
                .map(|b| (b.cursor, b.cells.to_vec()))
                .collect::<Vec<_>>(),
            (g.hints_used, g.revealed, g.banked),
            g.hint_techniques.clone(),
        )
    }

    #[derive(Debug, Clone)]
    enum Move {
        To(Cell),
        Press(u8),
        Mode,
        Clear,
        Fill,
        Marks,
        Hint,
        Reveal,
        Undo,
        Redo,
        Bookmark,
        Back,
    }

    fn a_move() -> impl Strategy<Value = Move> {
        prop_oneof![
            (0..CELLS).prop_map(Move::To),
            (1u8..=9).prop_map(Move::Press),
            Just(Move::Mode),
            Just(Move::Clear),
            Just(Move::Fill),
            Just(Move::Marks),
            Just(Move::Hint),
            Just(Move::Reveal),
            Just(Move::Undo),
            Just(Move::Redo),
            Just(Move::Bookmark),
            Just(Move::Back),
        ]
    }

    fn play(g: &mut Game, m: &Move) {
        match *m {
            Move::To(cell) => g.move_to(cell),
            Move::Press(digit) => g.press(digit),
            Move::Mode => g.next_mode(),
            Move::Clear => g.clear(),
            Move::Fill => g.fill_candidates(),
            Move::Marks => g.marks_to_candidates(),
            Move::Hint => {
                if let Hint::Step(hint) = g.hint() {
                    g.count_hint(&Hint::Step(hint.clone()));
                    g.apply_hint(&hint.step);
                }
            }
            Move::Reveal => g.reveal(),
            Move::Undo => g.undo(),
            Move::Redo => g.redo(),
            Move::Bookmark => g.bookmark(),
            Move::Back => {
                g.back_to_bookmark();
            }
        }
    }

    proptest! {
        /// Whatever was played, the game read back is the same game, and
        /// undoing and redoing it goes through the same states.
        #[test]
        fn a_saved_game_reads_back_the_same(moves in prop::collection::vec(a_move(), 0..60)) {
            let mut g = game().rated(Level::Medium);
            for m in &moves {
                play(&mut g, m);
            }
            let now = Instant::now();
            g.stop_clock(now);
            let mut back = Game::from_json(&g.to_json(now)).unwrap();
            prop_assert_eq!(kept(&back), kept(&g));
            for _ in 0..g.undo.len() {
                g.undo();
                back.undo();
                prop_assert_eq!(kept(&back), kept(&g));
            }
            for _ in 0..g.redo.len() {
                g.redo();
                back.redo();
                prop_assert_eq!(kept(&back), kept(&g));
            }
        }
    }

    #[test]
    fn the_time_played_is_kept_and_the_clock_comes_back_stopped() {
        let mut g = game();
        let t0 = Instant::now();
        g.start_clock(t0);
        let back = Game::from_json(&g.to_json(t0 + Duration::from_secs(90))).unwrap();
        assert!(!back.clock_is_running());
        assert_eq!(back.elapsed(t0), Duration::from_secs(90));
    }

    /// Each edit breaks one thing a game holds, and is refused for it.
    #[test]
    fn a_save_that_breaks_the_game_is_refused() {
        let json = game().to_json(Instant::now());
        let several = format!("{}{}", ".".repeat(20), &CLUES[20..]);
        let board = |grid: &str| format!(r#""board":"{grid}""#);
        let clue_changed = format!("4{}", &CLUES[1..]);
        let step_on_clue =
            r#""undo":[{"cursor":0,"cells":[[0,[0,null,"","",""],[0,null,"","",""]]]}]"#;
        let broken = [
            (json.replace(CLUES, &several), "more than one solution"),
            (
                json.replace(&board(CLUES), &board(&clue_changed)),
                "changes a clue",
            ),
            (
                json.replace(r#""undo":[]"#, step_on_clue),
                "step changes a clue",
            ),
            (
                json.replace(r#""cursor":0"#, r#""cursor":81"#),
                "81 isn't a cell",
            ),
            (json.replacen(r#""""#, r#""0""#, 1), "'0' isn't a digit"),
            (
                json.replace(r#""version":1"#, r#""version":2"#),
                "another version",
            ),
            (
                json.replace(r#""mode":"digit""#, r#""mode":"pen""#),
                "unknown mode",
            ),
        ];
        let mut marked = game();
        marked.bookmark();
        let marked = marked.to_json(Instant::now());
        let bookmark_on_clue = marked.replace("[[5,null", "[[4,null");
        assert_ne!(bookmark_on_clue, marked);
        let broken = broken
            .into_iter()
            .chain([(bookmark_on_clue, "bookmark changes a clue")]);
        let mut stepped = game();
        stepped.move_to(2);
        stepped.press(4);
        let stepped = stepped.to_json(Instant::now());
        let step_cursor = stepped.replacen(r#""undo":[{"cursor":2"#, r#""undo":[{"cursor":200"#, 1);
        assert_ne!(step_cursor, stepped);
        let bookmark_cursor = marked.replace(
            r#""bookmarks":[{"cursor":0"#,
            r#""bookmarks":[{"cursor":99"#,
        );
        assert_ne!(bookmark_cursor, marked);
        let short = json.replacen(r#""marks":["","#, r#""marks":["#, 1);
        let newer = json.replace(r#""version":1,"#, r#""version":2,"new":0,"#);
        let mut won = game();
        for cell in 0..CELLS {
            if let Some(d) = won.solution_digit(cell) {
                won.move_to(cell);
                won.press(d);
            }
        }
        let won = won.to_json(Instant::now());
        let broken = broken.into_iter().chain([
            (step_cursor, "200 isn't a cell"),
            (bookmark_cursor, "99 isn't a cell"),
            (short, "80 cells, not 81"),
            (newer, "another version of sumi (2)"),
            (won, "already won"),
        ]);
        for (text, why) in broken {
            let err = Game::from_json(&text).err().unwrap_or_default();
            assert!(err.contains(why), "{why:?} in {err:?}");
        }
        assert!(Game::from_json(&json).is_ok());
    }
}
