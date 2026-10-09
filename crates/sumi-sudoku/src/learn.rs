//! Learn: a lesson for each Easy and Medium technique, read from
//! `lessons/*.toml`, and drills for every technique on bank boards stopped
//! just before it. The answer is the move: a placing step's digit placed,
//! or an eliminating step's candidates struck and Enter. Any instance of
//! the technique on the board counts. Drawing is in `ui`, through
//! [`Panel`].

use std::sync::OnceLock;

use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};
use serde::Deserialize;
use sumi_core::bank;
use sumi_core::digits::Digits;
use sumi_core::grade::grade;
use sumi_core::grid::{CELLS, Cell, Grid, House, box_of, col, peers, row};
use sumi_core::hint::{Hint, Rung, StepHint};
use sumi_core::position::Position;
use sumi_core::puzzle::Puzzle;
use sumi_core::step::Step;
use sumi_core::technique::{Level, ORDER, Technique};
use sumi_core::words;

use crate::game::Game;
use crate::input::Action;

/// The lesson files, in the techniques' order.
const LESSON_FILES: [&str; 7] = [
    include_str!("../lessons/full-house.toml"),
    include_str!("../lessons/hidden-single.toml"),
    include_str!("../lessons/naked-single.toml"),
    include_str!("../lessons/pointing-pair.toml"),
    include_str!("../lessons/claiming.toml"),
    include_str!("../lessons/naked-pair.toml"),
    include_str!("../lessons/hidden-pair.toml"),
];

/// Boards in a drill.
pub const DRILL_BOARDS: usize = 5;

pub struct Lesson {
    pub technique: &'static dyn Technique,
    /// What to do on a Finish it or Find it board, in one line.
    goal: String,
    /// The line the lesson ends on.
    close: String,
    see: Board,
    frames: Vec<Frame>,
    finish: Board,
    find: Vec<Board>,
}

/// One step of See it: a sentence, and what of the technique's step the
/// board shows with it.
struct Frame {
    text: String,
    show: Show,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Show {
    Nothing,
    /// The rows, columns or boxes to look in, as the hint's third rung.
    Area,
    /// The pattern's cells too, as the fourth.
    Pattern,
    /// What the step places or removes, as the fifth.
    Result,
}

/// A board to answer on: a puzzle, the digits already placed, and the
/// candidates shown, or none for a placing technique, which is found from
/// the digits.
#[derive(Clone)]
pub struct Board {
    puzzle: Puzzle,
    digits: Grid,
    notes: Option<[Digits; CELLS]>,
}

impl Board {
    fn game(&self) -> Game {
        let game = Game::new(self.puzzle.clone()).starting_with(&self.digits);
        match &self.notes {
            Some(notes) => game.with_notes(notes),
            None => game,
        }
    }

    /// What the techniques read: the candidates shown, or with none shown,
    /// every candidate the digits allow.
    fn position(&self) -> Position {
        let mut grid = *self.puzzle.clues();
        for cell in 0..CELLS {
            if let Some(digit) = self.digits.get(cell) {
                grid.set(cell, Some(digit));
            }
        }
        let candidates = self
            .notes
            .unwrap_or(std::array::from_fn(|c| grid.possible(c)));
        Position::with_candidates(&grid, candidates).known_unique()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LessonFile {
    technique: String,
    goal: String,
    close: String,
    see: SeeFile,
    finish: BoardFile,
    find: Vec<BoardFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SeeFile {
    board: String,
    frames: Vec<FrameFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameFile {
    text: String,
    show: Show,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoardFile {
    board: String,
}

/// The lessons, in the techniques' order.
pub fn lessons() -> &'static [Lesson] {
    static LESSONS: OnceLock<Vec<Lesson>> = OnceLock::new();
    LESSONS.get_or_init(|| {
        LESSON_FILES
            .iter()
            .map(|text| read_lesson(text).expect("the lesson test reads every file"))
            .collect()
    })
}

/// The lesson for `technique`, if it has one.
pub fn lesson(technique: &dyn Technique) -> Option<&'static Lesson> {
    lessons()
        .iter()
        .find(|l| l.technique.name() == technique.name())
}

fn read_lesson(text: &str) -> Result<Lesson, String> {
    let file: LessonFile = toml::from_str(text).map_err(|err| err.message().to_string())?;
    let technique = *ORDER
        .iter()
        .find(|t| t.name() == file.technique)
        .ok_or_else(|| format!("no technique is called {:?}", file.technique))?;
    // Lesson boards hold every candidate the digits allow, except for
    // the placing techniques, whose boards show none.
    let board = |text: &str| -> Result<Board, String> {
        let grid: Grid = text.parse().map_err(|_| format!("{text:?} isn't a grid"))?;
        let puzzle = Puzzle::new(grid).map_err(|_| format!("{text} hasn't one solution"))?;
        let notes = (!places(technique)).then(|| std::array::from_fn(|c| grid.possible(c)));
        Ok(Board {
            puzzle,
            digits: Grid::EMPTY,
            notes,
        })
    };
    Ok(Lesson {
        technique,
        goal: file.goal,
        close: file.close,
        see: board(&file.see.board)?,
        frames: file
            .see
            .frames
            .into_iter()
            .map(|f| Frame {
                text: f.text,
                show: f.show,
            })
            .collect(),
        finish: board(&file.finish.board)?,
        find: file
            .find
            .iter()
            .map(|f| board(&f.board))
            .collect::<Result<_, _>>()?,
    })
}

/// Whether `technique` places a digit, rather than removing candidates.
fn places(technique: &dyn Technique) -> bool {
    technique.level() == Level::Easy
}

/// Up to `n` boards from different bank puzzles, each stopped just before
/// a step of `technique`, the easiest step there. Hard and Expert boards
/// come from puzzles whose hardest step it is; Easy and Medium ones from
/// any puzzle. A placing technique's board shows no candidates, so it must
/// be the easiest step on the digits alone.
pub fn drill<R: Rng + ?Sized>(
    rng: &mut R,
    technique: &'static dyn Technique,
    n: usize,
) -> Vec<Board> {
    let mut puzzles: Vec<Grid> = bank::puzzles()
        .into_iter()
        .filter(|(_, hardest)| technique.level() <= Level::Medium || *hardest == technique.name())
        .map(|(clues, _)| clues)
        .collect();
    puzzles.shuffle(rng);
    let mut boards = Vec::new();
    for clues in puzzles {
        if boards.len() == n {
            break;
        }
        let graded = grade(&clues).expect("bank puzzles have one solution");
        let mut p = Position::new(&clues).known_unique();
        let mut moments = Vec::new();
        for step in &graded.steps {
            if step.technique.name() == technique.name() {
                let board = Board {
                    puzzle: Puzzle::new(clues).expect("one solution"),
                    digits: placed(&clues, p.grid()),
                    notes: (!places(technique)).then(|| std::array::from_fn(|c| p.candidates(c))),
                };
                if easiest(&board.position()) == Some(technique.name()) {
                    moments.push(board);
                }
            }
            p.apply(step);
        }
        if let Some(board) = moments.choose(rng) {
            boards.push(board.clone());
        }
    }
    boards
}

/// The digits on `board` that aren't `clues`.
fn placed(clues: &Grid, board: &Grid) -> Grid {
    let mut digits = Grid::EMPTY;
    for cell in (0..CELLS).filter(|&c| clues.get(c).is_none()) {
        digits.set(cell, board.get(cell));
    }
    digits
}

/// The name of the first technique, in sumi's order, that finds a step.
fn easiest(p: &Position) -> Option<&'static str> {
    ORDER
        .iter()
        .find(|t| !t.all(p).is_empty())
        .map(|t| t.name())
}

/// What the panel shows during a lesson or a drill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    pub title: &'static str,
    /// Where in the lesson or drill this is, such as "Find it · 2 of 3".
    pub heading: String,
    pub text: String,
    /// What the board shows, as a hint's rung would.
    pub highlight: Option<Rung>,
    /// The keys the key bar offers, besides the key list and quit.
    pub keys: Vec<(Action, &'static str)>,
}

/// A lesson or a drill under way.
pub struct Learning {
    technique: &'static dyn Technique,
    course: Course,
    at: At,
    /// The board as it was given, which the answer is checked against.
    start: Position,
    /// Wrong answers on this board.
    misses: u8,
    /// The instance the player's answer matched, once it is right.
    found: Option<Step>,
    /// What the last answer drew.
    feedback: Option<String>,
}

enum Course {
    Lesson(&'static Lesson),
    Drill(Vec<Board>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    See(usize),
    Finish,
    Find(usize),
    Drill(usize),
    Close,
}

/// What Enter leads to.
pub enum Next {
    Stay,
    /// Play this board next.
    Board(Box<Game>),
    /// The lesson or drill is over.
    Done,
}

impl Learning {
    pub fn lesson(lesson: &'static Lesson) -> (Learning, Game) {
        Learning::at(lesson.technique, Course::Lesson(lesson), At::See(0))
    }

    /// A drill on `boards`, which must not be empty.
    pub fn drill(technique: &'static dyn Technique, boards: Vec<Board>) -> (Learning, Game) {
        Learning::at(technique, Course::Drill(boards), At::Drill(0))
    }

    fn at(technique: &'static dyn Technique, course: Course, at: At) -> (Learning, Game) {
        let mut learning = Learning {
            technique,
            course,
            at,
            start: Position::new(&Grid::EMPTY),
            misses: 0,
            found: None,
            feedback: None,
        };
        let game = learning.begin();
        (learning, game)
    }

    fn board(&self) -> Option<&Board> {
        match (&self.course, self.at) {
            (Course::Lesson(lesson), At::See(_) | At::Close) => Some(&lesson.see),
            (Course::Lesson(lesson), At::Finish) => Some(&lesson.finish),
            (Course::Lesson(lesson), At::Find(i)) => lesson.find.get(i),
            (Course::Drill(boards), At::Drill(i)) => boards.get(i),
            _ => None,
        }
    }

    /// The game for the board at `self.at`, everything about the last
    /// board forgotten. An eliminating technique's board starts in
    /// candidate mode, where striking is done.
    fn begin(&mut self) -> Game {
        let board = self.board().expect("every stage has a board").clone();
        self.start = board.position();
        self.misses = 0;
        self.found = None;
        self.feedback = None;
        let mut game = board.game();
        if !places(self.technique) {
            game.next_mode();
        }
        game
    }

    /// The simplest right answer on the board showing.
    #[cfg(test)]
    pub fn answer(&self) -> Step {
        self.instances().remove(0)
    }

    /// Every instance of the technique on the board as given.
    fn instances(&self) -> Vec<Step> {
        self.technique.all(&self.start)
    }

    /// Whether `action` reaches the game. While a lesson shows its frames
    /// or its last line, and once an answer is right, only looking around
    /// does; Enter and Esc are the lesson's own.
    pub fn lets(&self, action: Action) -> bool {
        let looking = matches!(
            action,
            Action::Step(_) | Action::Jump(_) | Action::Help | Action::Pause | Action::Quit
        );
        match action {
            Action::Confirm | Action::Close => false,
            _ if matches!(self.at, At::See(_) | At::Close) || self.found.is_some() => looking,
            _ => true,
        }
    }

    /// Enter: the next frame or board once an answer is right, or the
    /// check of struck candidates.
    pub fn confirm(&mut self, game: &mut Game) -> Next {
        let frames = match &self.course {
            Course::Lesson(lesson) => lesson.frames.len(),
            Course::Drill(_) => 0,
        };
        match self.at {
            At::See(i) if i + 1 < frames => {
                self.at = At::See(i + 1);
                Next::Stay
            }
            At::Close => Next::Done,
            _ if self.found.is_some() || matches!(self.at, At::See(_)) => self.next_board(),
            _ if places(self.technique) => {
                self.feedback = Some(self.goal());
                Next::Stay
            }
            _ => {
                self.check_strikes(game);
                Next::Stay
            }
        }
    }

    fn next_board(&mut self) -> Next {
        let finds = match &self.course {
            Course::Lesson(lesson) => lesson.find.len(),
            Course::Drill(boards) => boards.len(),
        };
        self.at = match self.at {
            At::See(_) => At::Finish,
            At::Finish => At::Find(0),
            At::Find(i) if i + 1 < finds => At::Find(i + 1),
            At::Drill(i) if i + 1 < finds => At::Drill(i + 1),
            _ => At::Close,
        };
        if self.at == At::Close {
            self.found = None;
            self.feedback = None;
            return Next::Stay;
        }
        Next::Board(Box::new(self.begin()))
    }

    /// After a key reached the game: a digit placed is the answer to a
    /// placing technique, and a slip in an eliminating one. A placed digit
    /// that isn't right is taken back.
    pub fn after(&mut self, game: &mut Game) {
        let Some(cell) = (0..CELLS)
            .find(|&c| self.start.grid().get(c).is_none() && game.board().get(c).is_some())
        else {
            return;
        };
        let digit = game.board().get(cell).expect("just placed");
        if !places(self.technique) {
            game.undo();
            self.miss(format!(
                "{} strikes candidates: strike them, then Enter.",
                self.technique.name()
            ));
            return;
        }
        let instance = self
            .instances()
            .into_iter()
            .find(|s| s.placement == Some((cell, digit)));
        if let Some(step) = instance {
            self.right(step);
            return;
        }
        game.undo();
        let why = if game.solution_digit(cell) == Some(digit) {
            format!(
                "{digit} is right in {}, but nothing on the board shows it yet.",
                words::cell(cell)
            )
        } else if let Some(house) = clash(self.start.grid(), cell, digit) {
            format!("There's already a {digit} in {}.", words::house(house))
        } else {
            format!("{digit} doesn't go in {}.", words::cell(cell))
        };
        self.miss(why);
    }

    /// Checks the candidates struck since the board was given: right if
    /// each is one an instance removes, and then the matching instance is
    /// shown whole.
    fn check_strikes(&mut self, game: &Game) {
        let struck: Vec<(Cell, u8)> = self
            .start
            .empty_cells()
            .flat_map(|c| {
                let gone = self.start.candidates(c).without(game.candidates(c));
                gone.iter().map(move |d| (c, d))
            })
            .collect();
        if struck.is_empty() {
            self.feedback = Some(self.goal());
            return;
        }
        let instances = self.instances();
        let removes = |s: &Step, (cell, digit): (Cell, u8)| {
            s.removals
                .iter()
                .any(|&(c, d)| c == cell && d.contains(digit))
        };
        if let Some(&(cell, digit)) = struck
            .iter()
            .find(|&&strike| !instances.iter().any(|s| removes(s, strike)))
        {
            self.miss(format!(
                "Nothing here rules out {digit} in {}: u takes it back.",
                words::cell(cell)
            ));
            return;
        }
        let best = instances
            .into_iter()
            .max_by_key(|s| struck.iter().filter(|&&strike| removes(s, strike)).count())
            .expect("a strike an instance removes");
        self.right(best);
    }

    fn right(&mut self, step: Step) {
        self.feedback = Some(format!("Right. {}", rung(&step, 5).text));
        self.found = Some(step);
    }

    fn miss(&mut self, why: String) {
        self.misses += 1;
        self.feedback = Some(if self.misses >= 2 {
            format!("{why} ? gives a hint.")
        } else {
            why
        });
    }

    /// What to do on the board: the lesson's goal, or the drill's. Enter
    /// too soon says it again.
    fn goal(&self) -> String {
        let name = self.technique.name();
        if let Course::Lesson(lesson) = self.course {
            lesson.goal.clone()
        } else if places(self.technique) {
            format!("{name}: place the digit it gives.")
        } else {
            format!("{name}: strike what it rules out, then Enter.")
        }
    }

    pub fn panel(&self) -> Panel {
        let title = match self.course {
            Course::Lesson(_) => "Lesson",
            Course::Drill(_) => "Drill",
        };
        let first = self.instances().into_iter().next();
        let (heading, text, highlight) = match (&self.course, self.at) {
            (Course::Lesson(lesson), At::See(i)) => {
                let frame = &lesson.frames[i];
                let step = first.expect("the lesson test finds the technique");
                let (text, highlight) = frame_text(frame, &step);
                let heading = format!("See it · {} of {}", i + 1, lesson.frames.len());
                (heading, text, highlight)
            }
            (Course::Lesson(lesson), At::Close) => ("Done".to_string(), lesson.close.clone(), None),
            (Course::Drill(boards), At::Close) => (
                "Done".to_string(),
                format!("That's all {} boards.", boards.len()),
                None,
            ),
            (course, at) => {
                let heading = match (course, at) {
                    (_, At::Finish) => "Finish it".to_string(),
                    (Course::Lesson(lesson), At::Find(i)) => {
                        format!("Find it · {} of {}", i + 1, lesson.find.len())
                    }
                    (Course::Drill(boards), At::Drill(i)) => {
                        format!("{} · {} of {}", self.technique.name(), i + 1, boards.len())
                    }
                    _ => String::new(),
                };
                let text = self.feedback.clone().unwrap_or_else(|| self.goal());
                // Finish it shows the pattern; once right, the answer shows.
                let highlight = match (&self.found, at) {
                    (Some(step), _) => Some(rung(step, 5)),
                    (None, At::Finish) => first.map(|s| rung(&s, 4)),
                    _ => None,
                };
                (heading, text, highlight)
            }
        };
        Panel {
            title,
            heading,
            text,
            highlight,
            keys: self.keys(),
        }
    }

    fn keys(&self) -> Vec<(Action, &'static str)> {
        let mut keys = match self.at {
            At::See(_) => vec![(Action::Confirm, "next")],
            At::Close => vec![(Action::Confirm, "done")],
            _ if self.found.is_some() => vec![(Action::Confirm, "next")],
            _ if places(self.technique) => vec![
                (Action::Digit(1), "place"),
                (Action::Undo, "undo"),
                (Action::Hint, "hint"),
            ],
            _ => vec![
                (Action::Digit(1), "strike"),
                (Action::Confirm, "check"),
                (Action::Undo, "undo"),
                (Action::Hint, "hint"),
            ],
        };
        keys.push((Action::Close, "leave"));
        keys
    }
}

/// A rung of the hint for `step`, which says what it says about it at
/// that height.
fn rung(step: &Step, n: usize) -> Rung {
    let hint = Hint::Step(Box::new(StepHint {
        step: step.clone(),
        level: step.technique.level(),
        count: 1,
    }));
    hint.rung(n)
}

/// A frame's sentence, with `{area}`, `{cells}`, `{result}` and
/// `{definition}` filled in from `step`, and what the board shows.
fn frame_text(frame: &Frame, step: &Step) -> (String, Option<Rung>) {
    let text = frame
        .text
        .replace("{area}", &rung(step, 3).text)
        .replace("{cells}", &rung(step, 4).text)
        .replace("{result}", &rung(step, 5).text)
        .replace("{definition}", &step.technique.definition(step));
    let highlight = match frame.show {
        Show::Nothing => None,
        Show::Area => Some(rung(step, 3)),
        Show::Pattern => Some(rung(step, 4)),
        Show::Result => Some(rung(step, 5)),
    };
    (text, highlight)
}

/// The row, column or box of `cell` that already holds `digit` on `grid`.
fn clash(grid: &Grid, cell: Cell, digit: u8) -> Option<House> {
    let holder = peers(cell).find(|&p| grid.get(p) == Some(digit))?;
    Some(if row(holder) == row(cell) {
        House::Row(row(cell))
    } else if col(holder) == col(cell) {
        House::Col(col(cell))
    } else {
        House::Box(box_of(cell))
    })
}

/// Every text a lesson's panel can show from its file and its boards:
/// the frames, the goal, the last line, and each instance's answer.
#[cfg(test)]
pub fn every_lesson_text() -> Vec<String> {
    let mut texts = Vec::new();
    for lesson in lessons() {
        let boards = [&lesson.see, &lesson.finish]
            .into_iter()
            .chain(&lesson.find);
        for board in boards {
            for step in lesson.technique.all(&board.position()) {
                texts.push(format!("Right. {}", rung(&step, 5).text));
            }
        }
        let step = &lesson.technique.all(&lesson.see.position())[0];
        texts.extend(lesson.frames.iter().map(|f| frame_text(f, step).0));
        texts.extend([lesson.goal.clone(), lesson.close.clone()]);
    }
    texts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every lesson file reads, its technique is the easiest step on every
    /// board with the candidates it shows, See it has frames, and Find it
    /// has 3 boards.
    #[test]
    fn every_lesson_holds_its_technique() {
        let lessons = lessons();
        assert_eq!(lessons.len(), 7);
        for (lesson, technique) in lessons.iter().zip(ORDER) {
            let name = technique.name();
            assert_eq!(lesson.technique.name(), name, "in the techniques' order");
            assert!(!lesson.frames.is_empty(), "{name}");
            assert_eq!(lesson.find.len(), 3, "{name}");
            let boards = [&lesson.see, &lesson.finish]
                .into_iter()
                .chain(&lesson.find);
            for board in boards {
                let p = board.position();
                assert_eq!(easiest(&p), Some(name), "{name}: {}", p.grid());
            }
        }
    }

    /// Every technique has a drill of 5 boards from different puzzles,
    /// each with the technique as its easiest step.
    #[test]
    fn every_technique_has_a_drill() {
        let mut rng = <rand::rngs::Xoshiro256PlusPlus as rand::SeedableRng>::seed_from_u64(1);
        for &technique in ORDER {
            let name = technique.name();
            let boards = drill(&mut rng, technique, DRILL_BOARDS);
            assert_eq!(boards.len(), DRILL_BOARDS, "{name}");
            for board in &boards {
                assert_eq!(easiest(&board.position()), Some(name));
                assert_eq!(board.notes.is_none(), places(technique));
            }
            let puzzles: std::collections::HashSet<String> = boards
                .iter()
                .map(|b| b.puzzle.clues().to_string())
                .collect();
            assert_eq!(puzzles.len(), DRILL_BOARDS, "{name}");
        }
    }

    fn lesson_named(name: &str) -> &'static Lesson {
        lessons()
            .iter()
            .find(|l| l.technique.name() == name)
            .unwrap()
    }

    /// Enter through the frames; the last one leads to Finish it's board.
    fn to_finish(lesson: &'static Lesson) -> (Learning, Game) {
        let (mut learning, mut game) = Learning::lesson(lesson);
        for _ in 1..lesson.frames.len() {
            assert!(matches!(learning.confirm(&mut game), Next::Stay));
        }
        match learning.confirm(&mut game) {
            Next::Board(game) => (learning, *game),
            _ => panic!("Finish it's board"),
        }
    }

    /// A placing technique's answer is its digit: right shows the step and
    /// waits for Enter; a clash names the house, a right digit nothing
    /// shows yet says so, and both are taken back; a second miss offers a
    /// hint.
    #[test]
    fn a_placing_step_is_answered_by_its_digit() {
        let (mut learning, mut game) = to_finish(lesson_named("Hidden single"));
        assert_eq!(learning.at, At::Finish);
        assert!(
            learning.panel().highlight.is_some(),
            "Finish it shows the pattern"
        );
        let step = learning.instances()[0].clone();
        let (cell, digit) = step.placement.unwrap();
        let other = (0..CELLS)
            .find(|&c| {
                c != cell
                    && learning.start.grid().get(c).is_none()
                    && learning
                        .instances()
                        .iter()
                        .all(|s| s.placement.unwrap().0 != c)
            })
            .unwrap();
        let answer = game.solution_digit(other).unwrap();
        game.move_to(other);
        game.press(answer);
        learning.after(&mut game);
        assert_eq!(game.board().get(other), None, "taken back");
        assert!(
            learning
                .feedback
                .as_ref()
                .unwrap()
                .contains("nothing on the board shows")
        );
        let clue = peers(other)
            .find(|&p| learning.start.grid().get(p).is_some())
            .unwrap();
        let taken = learning.start.grid().get(clue).unwrap();
        game.press(taken);
        learning.after(&mut game);
        let said = learning.feedback.clone().unwrap();
        assert!(
            said.starts_with(&format!("There's already a {taken} in")),
            "{said}"
        );
        assert!(said.ends_with("? gives a hint."), "{said}");
        game.move_to(cell);
        game.press(digit);
        learning.after(&mut game);
        assert!(learning.found.is_some());
        assert!(learning.panel().text.starts_with("Right."));
        assert!(
            !learning.lets(Action::Digit(1)),
            "the board waits for Enter"
        );
        assert!(matches!(learning.confirm(&mut game), Next::Board(_)));
        assert_eq!(learning.at, At::Find(0));
    }

    /// An eliminating technique's answer is its strikes and Enter: part of
    /// an instance is right; a candidate nothing rules out is named; Enter
    /// with nothing struck says what to do; a placed digit is taken back.
    #[test]
    fn an_eliminating_step_is_answered_by_its_strikes() {
        let (mut learning, mut game) = to_finish(lesson_named("Pointing pair"));
        assert_eq!(game.mode(), crate::game::Mode::Candidate);
        learning.confirm(&mut game);
        let goal = &lesson_named("Pointing pair").goal;
        assert_eq!(
            learning.feedback.as_ref(),
            Some(goal),
            "Enter too soon says the goal"
        );
        let step = learning.instances()[0].clone();
        let (cell, digits) = step.removals[0];
        let empty = learning.start.empty_cells().find(|&c| {
            !learning
                .instances()
                .iter()
                .any(|s| s.removals.iter().any(|&(r, _)| r == c))
        });
        let empty = empty.unwrap();
        let kept = learning.start.candidates(empty).iter().next().unwrap();
        game.move_to(empty);
        game.press(kept);
        learning.confirm(&mut game);
        assert!(
            learning
                .feedback
                .as_ref()
                .unwrap()
                .starts_with("Nothing here rules out")
        );
        game.undo();
        game.next_mode();
        game.next_mode();
        game.press(game.solution_digit(empty).unwrap());
        learning.after(&mut game);
        assert_eq!(game.board().get(empty), None, "taken back");
        game.next_mode();
        game.move_to(cell);
        game.press(digits.iter().next().unwrap());
        learning.confirm(&mut game);
        assert_eq!(learning.found.as_ref(), Some(&step));
    }

    /// A drill's boards come one after another, then a last line, then
    /// it's over.
    #[test]
    fn a_drill_goes_board_by_board() {
        let mut rng = <rand::rngs::Xoshiro256PlusPlus as rand::SeedableRng>::seed_from_u64(2);
        let technique = *ORDER.iter().find(|t| t.name() == "Full house").unwrap();
        let (mut learning, mut game) = Learning::drill(technique, drill(&mut rng, technique, 2));
        for board in 0..2 {
            assert_eq!(learning.at, At::Drill(board));
            let (cell, digit) = learning.instances()[0].placement.unwrap();
            game.move_to(cell);
            game.press(digit);
            learning.after(&mut game);
            if let Next::Board(next) = learning.confirm(&mut game) {
                game = *next;
            }
        }
        assert_eq!(learning.at, At::Close);
        assert!(matches!(learning.confirm(&mut game), Next::Done));
    }
}
