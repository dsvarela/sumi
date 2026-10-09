//! Hints: the easiest step on the board, told one rung at a time.
//!
//! A wrong digit comes first, then a candidate the player struck out that is
//! the cell's answer, then the first technique in [`ORDER`] that finds a step
//! from the player's candidates. A mistake is only ever located, as a box
//! and then a cell; a step's digit shows on its last rung, once the
//! technique has been named and its cells marked.

use crate::digits::Digits;
use crate::grid::{CELLS, Cell, Grid, House, box_of};
use crate::position::Position;
use crate::puzzle::Puzzle;
use crate::step::Step;
use crate::technique::{Level, ORDER};
use crate::words;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hint {
    /// Nothing left to hint at.
    Solved,
    /// A placed digit is wrong.
    Mistake(Cell),
    /// A cell's candidates leave out its answer.
    StruckOut(Cell),
    Step(Box<StepHint>),
    /// No technique finds a step.
    Stuck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepHint {
    pub step: Step,
    /// The level of the step's technique.
    pub level: Level,
    /// How many different steps there are at that level, this one included.
    pub count: usize,
}

/// What one rung says, and what the board shows at it. Each rung shows
/// everything the rungs before it do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rung {
    pub text: String,
    /// Rows, columns and boxes to shade faintly.
    pub areas: Vec<House>,
    /// The pattern's cells.
    pub cells: Vec<Cell>,
    /// Pattern cells drawn apart from `cells`, as colouring's second group.
    pub second_group: Vec<Cell>,
    pub removals: Vec<(Cell, Digits)>,
    pub placement: Option<(Cell, u8)>,
}

/// The hint for a board played from `puzzle`. `notes` are the player's
/// candidates for each cell, or `None` for a cell with none written, which
/// holds every digit it can.
pub fn find(board: &Grid, puzzle: &Puzzle, notes: &[Option<Digits>; CELLS]) -> Hint {
    let solution = puzzle.solution();
    if let Some(cell) =
        (0..CELLS).find(|&c| board.get(c).is_some() && board.get(c) != solution.get(c))
    {
        return Hint::Mistake(cell);
    }
    if board.is_solved() {
        return Hint::Solved;
    }
    let answer_missing = |&c: &Cell| match (board.get(c), notes[c], solution.get(c)) {
        (None, Some(set), Some(answer)) => !set.contains(answer),
        _ => false,
    };
    if let Some(cell) = (0..CELLS).find(answer_missing) {
        return Hint::StruckOut(cell);
    }
    let position = Position::with_candidates(
        board,
        std::array::from_fn(|c| notes[c].unwrap_or_else(|| board.possible(c))),
    )
    .known_unique();
    let Some(step) = ORDER.iter().find_map(|t| t.find(&position)) else {
        return Hint::Stuck;
    };
    let step = picked_form(&position, step);
    let level = step.technique.level();
    let count = steps_at(&position, level);
    Hint::Step(Box::new(StepHint { step, level, count }))
}

/// The form of `step` the hint names. A step that reads the same in several
/// houses, as a hidden single in its box, row and column, takes the kind its
/// cells and digits pick, so hints name boxes, rows and columns alike rather
/// than the box `find` keeps, and a position always gives the same hint.
fn picked_form(p: &Position, step: Step) -> Step {
    let digits: usize = step.digits.iter().map(usize::from).sum();
    let first = (step.cells.iter().sum::<usize>() + digits) % 3;
    let same = |s: &Step| {
        s.cells == step.cells && s.placement == step.placement && s.removals == step.removals
    };
    step.technique
        .search(p)
        .into_iter()
        .filter(same)
        .filter(|s| !s.houses.is_empty())
        .min_by_key(|s| (kind(s.houses[0]) + 3 - first) % 3)
        .unwrap_or(step)
}

/// Box, row and column as 0, 1 and 2.
fn kind(house: House) -> usize {
    match house {
        House::Box(_) => 0,
        House::Row(_) => 1,
        House::Col(_) => 2,
    }
}

/// What a step does: the digit it places, or the candidates it removes.
type Effect = (Option<(Cell, u8)>, Vec<(Cell, Digits)>);

/// How many different things can be done at `level`. A digit that two
/// techniques can place counts once, and so does a step whose removals another
/// step also makes: once that one is applied, nothing is left of it.
fn steps_at(p: &Position, level: Level) -> usize {
    let effects = ORDER
        .iter()
        .filter(|t| t.level() == level)
        .flat_map(|t| t.all(p))
        .map(|step| (step.placement, step.removals))
        .collect();
    count_distinct(effects)
}

fn count_distinct(effects: Vec<Effect>) -> usize {
    let contains = |big: &Effect, small: &Effect| {
        big.0 == small.0
            && small.1.iter().all(|&(cell, digits)| {
                big.1
                    .iter()
                    .any(|&(c, d)| c == cell && digits.without(d).is_empty())
            })
    };
    let mut distinct: Vec<&Effect> = Vec::new();
    for effect in &effects {
        if !distinct.contains(&effect) {
            distinct.push(effect);
        }
    }
    distinct
        .iter()
        .filter(|small| {
            !distinct
                .iter()
                .any(|big| big != *small && contains(big, small))
        })
        .count()
}

impl Hint {
    /// How many rungs the ladder has: three to find a mistake, five to
    /// explain a step, one for the rest.
    pub fn rungs(&self) -> usize {
        match self {
            Hint::Solved | Hint::Stuck => 1,
            Hint::Mistake(_) | Hint::StruckOut(_) => 3,
            Hint::Step(_) => 5,
        }
    }

    /// The step to apply once the last rung is showing, if the hint has one.
    pub fn step(&self) -> Option<&Step> {
        match self {
            Hint::Step(hint) => Some(&hint.step),
            _ => None,
        }
    }

    /// Rung `n` from 1; a rung past the last is the last.
    pub fn rung(&self, n: usize) -> Rung {
        let n = n.clamp(1, self.rungs());
        let mut rung = Rung {
            text: String::new(),
            areas: Vec::new(),
            cells: Vec::new(),
            second_group: Vec::new(),
            removals: Vec::new(),
            placement: None,
        };
        match self {
            Hint::Solved => rung.text = "The puzzle is solved.".into(),
            Hint::Stuck => rung.text = "No logical step found with sumi's techniques.".into(),
            Hint::Mistake(cell) | Hint::StruckOut(cell) => {
                let wrong_digit = matches!(self, Hint::Mistake(_));
                rung.text = match (n, wrong_digit) {
                    (1, true) => "There's a mistake on the board.".into(),
                    (1, false) => "There's a mistake in your candidates.".into(),
                    (2, _) => format!("The mistake is in box {}.", box_of(*cell) + 1),
                    (_, true) => format!("The mistake is in {}.", words::cell(*cell)),
                    (_, false) => format!(
                        "The mistake is in {}: it no longer lists its answer.",
                        words::cell(*cell)
                    ),
                };
                if n >= 2 {
                    rung.areas = vec![House::Box(box_of(*cell))];
                }
                if n >= 3 {
                    rung.cells = vec![*cell];
                }
            }
            Hint::Step(hint) => hint.fill(n, &mut rung),
        }
        rung
    }
}

impl StepHint {
    fn fill(&self, n: usize, rung: &mut Rung) {
        let step = &self.step;
        let technique = step.technique;
        let explanation = technique.explain(step);
        rung.text = match n {
            1 => self.difficulty(),
            2 => format!(
                "{} {}",
                technique.look_for(step),
                technique.definition(step)
            ),
            3 => explanation.area.clone(),
            4 => cells_line(step, &explanation.second_group),
            _ => format!("{} {}", explanation.effect, explanation.reason),
        };
        if n >= 3 {
            rung.areas = explanation.area_houses;
        }
        if n >= 4 {
            rung.cells = step
                .cells
                .iter()
                .copied()
                .filter(|c| !explanation.second_group.contains(c))
                .collect();
            rung.second_group = explanation.second_group;
        }
        if n >= 5 {
            rung.removals = step.removals.clone();
            rung.placement = step.placement;
        }
    }

    /// "A Hard step is available (3 steps at that level). Nothing easier."
    fn difficulty(&self) -> String {
        let level = format!("{:?}", self.level);
        let article = if level.starts_with(['A', 'E', 'I', 'O', 'U']) {
            "An"
        } else {
            "A"
        };
        let plural = if self.count == 1 { "" } else { "s" };
        let nothing_easier = if self.level == Level::Easy {
            ""
        } else {
            " Nothing easier."
        };
        format!(
            "{article} {level} step is available ({} step{plural} at that level).{nothing_easier}",
            self.count
        )
    }
}

/// The most cells the fourth rung names before it only counts them.
const MOST_NAMED: usize = 6;

/// The pattern's cells, as the fourth rung names them.
fn cells_line(step: &Step, second_group: &[Cell]) -> String {
    if second_group.is_empty() {
        return format!("Cells: {}.", words::cells_up_to(&step.cells, MOST_NAMED));
    }
    let first: Vec<Cell> = step
        .cells
        .iter()
        .copied()
        .filter(|c| !second_group.contains(c))
        .collect();
    format!(
        "Group A, brighter: {}. Group B: {}.",
        words::cells_up_to(&first, MOST_NAMED),
        words::cells_up_to(second_group, MOST_NAMED)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::generate;
    use crate::technique::examples::*;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    /// HoDoKu's examples, each with the hint sumi gives on it. A naked
    /// triple's example has a hidden pair that is easier, and a naked
    /// single's has a hidden single.
    const EXAMPLES: &[(&Example, &str)] = &[
        (&KITE_1, "2-String Kite"),
        (&KITE_2, "2-String Kite"),
        (&X_WING_1, "X-Wing"),
        (&X_WING_2, "X-Wing"),
        (&SWORDFISH_1, "Swordfish"),
        (&SWORDFISH_2, "Swordfish"),
        (&FULL_HOUSE_1, "Full house"),
        (&HIDDEN_SINGLE_1, "Hidden single"),
        (&HIDDEN_SINGLE_2, "Hidden single"),
        (&HIDDEN_PAIR_1, "Hidden pair"),
        (&HIDDEN_PAIR_2, "Hidden pair"),
        (&HIDDEN_TRIPLE_1, "Hidden triple"),
        (&HIDDEN_TRIPLE_2, "Hidden triple"),
        (&HIDDEN_QUAD_1, "Hidden quad"),
        (&HIDDEN_QUAD_2, "Hidden quad"),
        (&POINTING_1, "Pointing pair"),
        (&POINTING_2, "Pointing pair"),
        (&CLAIMING_1, "Claiming"),
        (&CLAIMING_2, "Claiming"),
        (&NAKED_SINGLE_1, "Hidden single"),
        (&NAKED_PAIR_1, "Naked pair"),
        (&NAKED_PAIR_2, "Naked pair"),
        (&NAKED_TRIPLE_1, "Hidden pair"),
        (&NAKED_TRIPLE_2, "Hidden pair"),
        (&NAKED_QUAD_1, "Naked quad"),
        (&NAKED_QUAD_2, "Naked quad"),
        (&SIMPLE_COLOURING_1, "Simple colouring"),
        (&SIMPLE_COLOURING_2, "Simple colouring"),
        (&SKYSCRAPER_1, "Skyscraper"),
        (&SKYSCRAPER_2, "Skyscraper"),
        (&UNIQUE_RECTANGLE_1, "Unique rectangle"),
        (&UNIQUE_RECTANGLE_2, "Unique rectangle"),
        (&W_WING_1, "W-Wing"),
        (&W_WING_2, "W-Wing"),
        (&Y_WING_1, "Y-Wing"),
        (&Y_WING_2, "Y-Wing"),
        (&XYZ_WING_1, "XYZ-Wing"),
        (&XYZ_WING_2, "XYZ-Wing"),
    ];

    /// The panel's text is 30 columns wide: `PANEL_WIDTH` in the ui is 34,
    /// less two border columns and two of padding.
    const WIDTH: usize = 30;

    fn wrapped_lines(text: &str) -> usize {
        let mut lines = 1;
        let mut used = 0;
        for word in text.split_whitespace() {
            let len = word.chars().count();
            if used == 0 {
                used = len;
            } else if used + 1 + len <= WIDTH {
                used += 1 + len;
            } else {
                lines += 1;
                used = len;
            }
        }
        lines
    }

    /// The notes a player would have written from `p`: its candidates in
    /// every empty cell.
    fn notes_of(p: &Position) -> [Option<Digits>; CELLS] {
        std::array::from_fn(|c| p.grid().get(c).is_none().then(|| p.candidates(c)))
    }

    fn puzzle_of(grid: &Grid) -> Puzzle {
        Puzzle::new(*grid).expect("one solution")
    }

    fn hint_for(example: &Example) -> Hint {
        let p = example.position();
        find(p.grid(), &puzzle_of(p.grid()), &notes_of(&p))
    }

    fn step_of(hint: &Hint) -> &StepHint {
        match hint {
            Hint::Step(hint) => hint,
            other => panic!("not a step: {other:?}"),
        }
    }

    /// The five rungs of the first step each technique finds on each of the
    /// plan's examples, shown with the same wording the board uses.
    #[test]
    fn every_rung_of_every_technique_fits_four_lines_and_avoids_banned_words() {
        let banned = [
            "note", "pencil", "unit", "region", "block", "obvious", "simply", "just", "easy ",
        ];
        let mut seen = std::collections::HashSet::new();
        for (example, _) in EXAMPLES {
            let p = example.position().known_unique();
            for technique in ORDER {
                for step in technique.all(&p) {
                    seen.insert(technique.name());
                    let hint = StepHint {
                        level: technique.level(),
                        count: 12,
                        step,
                    };
                    for n in 1..=5 {
                        let mut rung = Rung {
                            text: String::new(),
                            areas: vec![],
                            cells: vec![],
                            second_group: vec![],
                            removals: vec![],
                            placement: None,
                        };
                        hint.fill(n, &mut rung);
                        let text = &rung.text;
                        assert!(
                            wrapped_lines(text) <= 4,
                            "{}, rung {n}: {text}",
                            technique.name()
                        );
                        // Rung 1 names the level, which can be "Easy".
                        for word in banned.iter().filter(|_| n > 1) {
                            assert!(
                                !text.to_lowercase().contains(word),
                                "{}, rung {n}: {text}",
                                technique.name()
                            );
                        }
                        if n == 5 {
                            let verb = if hint.step.placement.is_some() {
                                "Place"
                            } else {
                                "Remove"
                            };
                            assert!(text.starts_with(verb), "{text}");
                        }
                    }
                }
            }
        }
        assert_eq!(seen.len(), ORDER.len(), "an example is missing: {seen:?}");
    }

    #[test]
    fn a_pointing_pair_is_worded_from_its_fields() {
        let p = POINTING_1.position().known_unique();
        let step = ORDER[3]
            .all(&p)
            .into_iter()
            .find(|s| s.removals == [(24, [5].into_iter().collect())])
            .unwrap();
        assert_eq!(step.technique.name(), "Pointing pair");
        assert_eq!(step.technique.look_for(&step), "Look for a pointing pair.");
        let explanation = step.technique.explain(&step);
        assert_eq!(explanation.area, "On digit 5, box 1 and row 3.");
        assert_eq!(explanation.effect, "Remove 5 from r3c7.");
        assert_eq!(
            explanation.reason,
            "Box 1 must put its 5 in row 3, so the rest of row 3 can't."
        );
    }

    #[test]
    fn a_fish_names_its_rows_and_the_columns_that_cannot_hold_the_digit() {
        let p = X_WING_1.position().known_unique();
        let step = ORDER[9].find(&p).unwrap();
        assert_eq!(step.technique.name(), "X-Wing");
        assert_eq!(step.technique.look_for(&step), "Look for an X-Wing.");
        let explanation = step.technique.explain(&step);
        assert_eq!(explanation.area, "On digit 5, rows 2 and 5.");
        assert_eq!(explanation.effect, "Remove 5 from r4c5.");
        assert_eq!(
            explanation.reason,
            "Rows 2 and 5 put their 5s in columns 5 and 8, so the rest of those columns can't."
        );
        assert_eq!(explanation.area_houses, [House::Row(1), House::Row(4)]);
    }

    #[test]
    fn a_wing_names_its_pivot_and_the_digit() {
        let p = Y_WING_1.position().known_unique();
        let step = ORDER[16].find(&p).unwrap();
        assert_eq!(step.technique.name(), "Y-Wing");
        let explanation = step.technique.explain(&step);
        assert_eq!(explanation.effect, "Remove 2 from r2c6.");
        assert!(
            explanation.area.starts_with("Middle cell r"),
            "{}",
            explanation.area
        );
        let pivot = words::cell(step.key_cells[0]);
        assert!(explanation.area.contains(&pivot));
        assert!(
            explanation
                .reason
                .starts_with(&format!("Whatever {pivot} is, ")),
            "{}",
            explanation.reason
        );
    }

    /// The second rung's opening for each technique, articles included.
    #[test]
    fn each_technique_is_named_with_the_right_article() {
        let expected = [
            ("Full house", "Look for a full house."),
            ("Hidden single", "Look for a hidden single."),
            ("Naked single", "Look for a naked single."),
            ("Pointing pair", "Look for a pointing pair."),
            ("Claiming", "Look for claiming."),
            ("Naked pair", "Look for a naked pair."),
            ("Hidden pair", "Look for a hidden pair."),
            ("Naked triple", "Look for a naked triple."),
            ("Hidden triple", "Look for a hidden triple."),
            ("X-Wing", "Look for an X-Wing."),
            ("Naked quad", "Look for a naked quad."),
            ("Swordfish", "Look for a swordfish."),
            ("Hidden quad", "Look for a hidden quad."),
            ("Skyscraper", "Look for a skyscraper."),
            ("2-String Kite", "Look for a 2-String Kite."),
            ("W-Wing", "Look for a W-Wing."),
            ("Y-Wing", "Look for a Y-Wing."),
            ("XYZ-Wing", "Look for an XYZ-Wing."),
            ("Unique rectangle", "Look for a unique rectangle."),
            ("Simple colouring", "Look for simple colouring."),
        ];
        let mut seen = 0;
        for (name, sentence) in expected {
            let technique = ORDER.iter().find(|t| t.name() == name).unwrap();
            let step = EXAMPLES
                .iter()
                .find_map(|(e, _)| technique.find(&e.position().known_unique()))
                .unwrap_or_else(|| panic!("no example step for {name}"));
            if step.cells.len() != 3 || name != "Pointing pair" {
                assert_eq!(technique.look_for(&step), sentence);
                seen += 1;
            }
        }
        assert_eq!(seen, ORDER.len());
    }

    /// An X-Wing's four cells are named on the fourth rung, not counted.
    #[test]
    fn the_cell_rung_names_up_to_six_cells() {
        let p = X_WING_1.position().known_unique();
        let step = ORDER[9].find(&p).unwrap();
        assert_eq!(step.cells.len(), 4);
        let hint = Hint::Step(Box::new(StepHint {
            step,
            level: Level::Hard,
            count: 1,
        }));
        assert_eq!(hint.rung(4).text, "Cells: r2c5, r2c8, r5c5 and r5c8.");
    }

    #[test]
    fn three_spots_in_a_box_make_a_pointing_triple() {
        let mut candidates = [Digits::ALL; CELLS];
        for cell in [9, 10, 11, 18, 19, 20] {
            candidates[cell].remove(1);
        }
        let p = Position::with_candidates(&Grid::EMPTY, candidates);
        let step = ORDER[3].find(&p).unwrap();
        assert_eq!(step.cells, [0, 1, 2]);
        assert_eq!(
            step.technique.look_for(&step),
            "Look for a pointing triple."
        );
    }

    #[test]
    fn the_hint_goes_to_a_wrong_digit_before_anything_else() {
        let p = FULL_HOUSE_1.position();
        let puzzle = puzzle_of(p.grid());
        let solution = *puzzle.solution();
        let mut board = *p.grid();
        let cell = (0..CELLS).find(|&c| board.get(c).is_none()).unwrap();
        let wrong = (1..=9).find(|&d| Some(d) != solution.get(cell)).unwrap();
        board.set(cell, Some(wrong));
        // A struck-out answer elsewhere comes second.
        let mut notes = [None; CELLS];
        let other = (cell + 1..CELLS).find(|&c| board.get(c).is_none()).unwrap();
        notes[other] = Some(Digits::NONE);
        assert_eq!(find(&board, &puzzle, &notes), Hint::Mistake(cell));
    }

    #[test]
    fn the_hint_goes_to_a_struck_out_answer_before_any_technique() {
        let p = FULL_HOUSE_1.position();
        let puzzle = puzzle_of(p.grid());
        let solution = *puzzle.solution();
        let mut notes = notes_of(&p);
        let cell = (0..CELLS).find(|&c| notes[c].is_some()).unwrap();
        let answer = solution.get(cell).unwrap();
        notes[cell] = notes[cell].map(|set| set.without([answer].into_iter().collect()));
        assert_eq!(find(p.grid(), &puzzle, &notes), Hint::StruckOut(cell));
    }

    #[test]
    fn a_cell_with_no_notes_holds_every_digit_it_can() {
        let p = FULL_HOUSE_1.position();
        let puzzle = puzzle_of(p.grid());
        let solution = *puzzle.solution();
        let hint = find(p.grid(), &puzzle, &[None; CELLS]);
        let step = &step_of(&hint).step;
        assert_eq!(step.technique.name(), "Full house");
        let (cell, digit) = step.placement.unwrap();
        assert_eq!(solution.get(cell), Some(digit));
    }

    #[test]
    fn a_board_the_techniques_cannot_move_has_no_step() {
        // Every technique fails on an empty grid.
        let hint = find(&Grid::EMPTY, &puzzle_of_any(), &[None; CELLS]);
        assert_eq!(
            hint.rung(1).text,
            "No logical step found with sumi's techniques."
        );
        assert_eq!(hint.rungs(), 1);
    }

    fn puzzle_of_any() -> Puzzle {
        generate(&mut Xoshiro256PlusPlus::seed_from_u64(1))
    }

    #[test]
    fn a_solved_board_has_nothing_to_hint() {
        let puzzle = puzzle_of_any();
        let solution = *puzzle.solution();
        assert_eq!(find(&solution, &puzzle, &[None; CELLS]), Hint::Solved);
    }

    /// One empty cell left is a full house, a hidden single and a naked
    /// single, all placing the same digit: one step, not three.
    #[test]
    fn a_placement_three_techniques_make_counts_once() {
        let puzzle = puzzle_of_any();
        let solution = *puzzle.solution();
        let mut board = solution;
        board.set(40, None);
        let p = Position::new(&board).known_unique();
        let by_each: usize = ORDER
            .iter()
            .filter(|t| t.level() == Level::Easy)
            .map(|t| t.all(&p).len())
            .sum();
        assert_eq!(by_each, 3);
        assert_eq!(steps_at(&p, Level::Easy), 1);
        let hint = find(&board, &puzzle, &[None; CELLS]);
        assert_eq!(step_of(&hint).count, 1);
    }

    #[test]
    fn rung_one_gives_the_level_and_the_count() {
        let hint = StepHint {
            step: ORDER[0].find(&FULL_HOUSE_1.position()).unwrap(),
            level: Level::Easy,
            count: 3,
        };
        assert_eq!(
            hint.difficulty(),
            "An Easy step is available (3 steps at that level)."
        );
        let hard = StepHint {
            level: Level::Hard,
            count: 1,
            ..hint
        };
        assert_eq!(
            hard.difficulty(),
            "A Hard step is available (1 step at that level). Nothing easier."
        );
    }

    #[test]
    fn a_step_another_contains_is_not_counted_apart() {
        let one: Digits = [1].into_iter().collect();
        let one_seven: Digits = [1, 7].into_iter().collect();
        let small: Effect = (None, vec![(5, one), (9, one)]);
        let big: Effect = (None, vec![(5, one_seven), (9, one)]);
        let apart: Effect = (None, vec![(30, one)]);
        assert_eq!(count_distinct(vec![small.clone(), big.clone()]), 1);
        assert_eq!(count_distinct(vec![big.clone(), small.clone()]), 1);
        assert_eq!(count_distinct(vec![small.clone(), big, apart.clone()]), 2);
        assert_eq!(count_distinct(vec![small.clone(), small]), 1);
        // Two cells losing 1 is not inside one cell losing 1 and 7, nor the
        // other way round: they are two steps.
        let wider: Effect = (None, vec![(5, one_seven)]);
        let longer: Effect = (None, vec![(5, one), (9, one)]);
        assert_eq!(count_distinct(vec![wider, longer]), 2);
        let place: Effect = (Some((4, 2)), vec![]);
        assert_eq!(count_distinct(vec![place.clone(), apart, place]), 2);
    }

    /// Every digit the effect line names is one the step removes. Naked
    /// subsets once named all the subset's digits, though a digit that is a
    /// candidate nowhere else in the house is not removed.
    #[test]
    fn the_effect_line_names_only_digits_the_step_removes() {
        let mut checked = 0;
        for seed in 0..12 {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
            let level = if seed % 2 == 0 {
                Level::Hard
            } else {
                Level::Expert
            };
            let puzzle = crate::bank::pick(&mut rng, level).unwrap();
            for step in &solving_steps(puzzle.clues()) {
                let effect = step.technique.explain(step).effect;
                // "Remove 2, 7 and 9 elsewhere in row 1." or "Remove 4 from
                // r5c3 and r9c8.": the digits come before the place.
                if !effect.starts_with("Remove ") || effect.contains("candidates") {
                    continue;
                }
                let head = effect.split(" from ").next().unwrap();
                let head = head.split(" elsewhere").next().unwrap();
                let removed = step
                    .removals
                    .iter()
                    .fold(Digits::NONE, |all, &(_, d)| all.union(d));
                for digit in head
                    .split(|c: char| !c.is_ascii_digit())
                    .filter_map(|t| t.parse::<u8>().ok())
                {
                    assert!(removed.contains(digit), "{effect}: removes {removed:?}");
                }
                checked += 1;
            }
        }
        assert!(checked > 40, "{checked}");
    }

    /// The steps hints would give from the clues to the solution, each
    /// applied to the candidates in full.
    fn solving_steps(clues: &Grid) -> Vec<Step> {
        let mut p = Position::new(clues).known_unique();
        let mut steps = Vec::new();
        while let Some(step) = ORDER.iter().find_map(|t| t.find(&p)) {
            p.apply(&step);
            steps.push(step);
        }
        steps
    }

    #[test]
    fn a_naked_single_does_not_claim_the_grid_alone_rules_out_the_rest() {
        let mut candidates = [Digits::ALL; CELLS];
        candidates[40] = [7].into_iter().collect(); // the player's notes, not the grid
        let p = Position::with_candidates(&Grid::EMPTY, candidates);
        let step = ORDER[2].find(&p).unwrap();
        assert_eq!(step.technique.name(), "Naked single");
        let reason = step.technique.explain(&step).reason;
        assert!(!reason.contains("row, column and box"), "{reason}");
    }

    #[test]
    fn a_struck_out_answer_is_found_in_three_rungs_without_the_digit() {
        let hint = Hint::StruckOut(40);
        assert_eq!(hint.rungs(), 3);
        assert_eq!(hint.rung(1).text, "There's a mistake in your candidates.");
        assert_eq!(hint.rung(2).text, "The mistake is in box 5.");
        assert_eq!(
            hint.rung(3).text,
            "The mistake is in r5c5: it no longer lists its answer."
        );
        assert_eq!(hint.rung(0), hint.rung(1));
        assert_eq!(hint.rung(9), hint.rung(3));
        assert_eq!(Hint::Mistake(40).rung(9), Hint::Mistake(40).rung(3));
        assert_eq!(Hint::Stuck.rung(5), Hint::Stuck.rung(1));
    }

    #[test]
    fn each_rung_shows_more_of_the_board() {
        let hint = hint_for(&FULL_HOUSE_1);
        let shown: Vec<_> = (1..=5).map(|n| hint.rung(n)).collect();
        assert!(shown[1].areas.is_empty() && shown[1].cells.is_empty());
        assert!(!shown[2].areas.is_empty() && shown[2].cells.is_empty());
        assert!(!shown[3].cells.is_empty() && shown[3].placement.is_none());
        assert!(shown[4].placement.is_some());
        assert_eq!(hint.rung(9), shown[4]);
    }

    #[test]
    fn a_mistake_is_found_in_three_rungs_without_the_answer() {
        let hint = Hint::Mistake(40);
        assert_eq!(hint.rung(1).text, "There's a mistake on the board.");
        assert_eq!(hint.rung(2).text, "The mistake is in box 5.");
        assert_eq!(hint.rung(2).areas, [House::Box(4)]);
        assert_eq!(hint.rung(3).text, "The mistake is in r5c5.");
        assert_eq!(hint.rung(3).cells, [40]);
        assert!(hint.step().is_none());
    }

    proptest! {
        /// Whatever correct digits and struck-out wrong candidates a player
        /// has, following hints never makes a wrong move, and never lands on
        /// a mistake of its own.
        #[test]
        fn following_hints_from_correct_play_never_breaks_the_solution(
            seed in any::<u64>(),
            play in prop::collection::vec((0..CELLS, 1u8..=9, any::<bool>()), 0..120),
        ) {
            let puzzle = generate(&mut Xoshiro256PlusPlus::seed_from_u64(seed));
            let solution = *puzzle.solution();
            let mut board = *puzzle.clues();
            let mut notes: [Option<Digits>; CELLS] = [None; CELLS];
            for (cell, digit, place) in play {
                let answer = solution.get(cell).unwrap();
                if board.get(cell).is_some() {
                    continue;
                }
                if place {
                    board.set(cell, Some(answer));
                    notes[cell] = None;
                } else if digit != answer {
                    let set = notes[cell].unwrap_or_else(|| board.possible(cell));
                    notes[cell] = Some(set.without([digit].into_iter().collect()));
                }
            }
            for _ in 0..200 {
                match find(&board, &puzzle, &notes) {
                    Hint::Solved => {
                        prop_assert_eq!(board, solution);
                        break;
                    }
                    Hint::Stuck => break,
                    Hint::Mistake(c) | Hint::StruckOut(c) => {
                        return Err(TestCaseError::fail(format!("mistake at {c}")));
                    }
                    hint @ Hint::Step(_) => {
                        for n in 1..=5 {
                            prop_assert!(!hint.rung(n).text.is_empty());
                        }
                        let step = hint.step().unwrap().clone();
                        if let Some((cell, digit)) = step.placement {
                            prop_assert_eq!(solution.get(cell), Some(digit));
                            board.set(cell, Some(digit));
                            notes[cell] = None;
                            for peer in crate::grid::peers(cell) {
                                if let Some(set) = &mut notes[peer] {
                                    set.remove(digit);
                                }
                            }
                        }
                        for &(cell, digits) in &step.removals {
                            prop_assert!(!digits.contains(solution.get(cell).unwrap()));
                            let set = notes[cell].unwrap_or_else(|| board.possible(cell));
                            notes[cell] = Some(set.without(digits));
                        }
                    }
                }
            }
        }
    }

    /// A hidden single is often the only spot in its box, row and column at
    /// once. Hints name each kind about a third of the time along the bank's
    /// solving paths, and name the step the grader takes.
    #[test]
    fn hidden_singles_are_named_in_boxes_rows_and_columns_alike() {
        let bank = include_str!("../data/bank.txt");
        let mut named = [0usize; 3];
        for line in bank.lines().filter(|l| !l.starts_with('#')).take(20) {
            let clues: Grid = line.split(' ').next().unwrap().parse().unwrap();
            let puzzle = puzzle_of(&clues);
            let mut p = Position::new(&clues).known_unique();
            for graded in solving_steps(&clues) {
                let hint = find(p.grid(), &puzzle, &notes_of(&p));
                let step = &step_of(&hint).step;
                assert_eq!(
                    (&step.cells, step.placement, &step.removals),
                    (&graded.cells, graded.placement, &graded.removals)
                );
                if step.technique.name() == "Hidden single" {
                    named[kind(step.houses[0])] += 1;
                }
                p.apply(&graded);
            }
        }
        let total: usize = named.iter().sum();
        for n in named {
            assert!((total / 4..=total * 5 / 12).contains(&n), "{named:?}");
        }
    }

    #[test]
    fn each_example_gets_the_hint_for_its_technique() {
        for (example, expected) in EXAMPLES {
            let hint = hint_for(example);
            assert_eq!(step_of(&hint).step.technique.name(), *expected);
        }
    }

    /// Candidates the player struck out are used: without them, a pointing
    /// pair comes before the hidden quad or colouring the notes allow.
    #[test]
    fn the_players_notes_decide_the_hint() {
        for example in [&HIDDEN_QUAD_1, &SIMPLE_COLOURING_1] {
            let p = example.position();
            let puzzle = puzzle_of(p.grid());
            let hint = find(p.grid(), &puzzle, &[None; CELLS]);
            assert_eq!(step_of(&hint).step.technique.name(), "Pointing pair");
        }
    }
}
