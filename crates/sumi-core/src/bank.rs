//! Graded Hard and Expert puzzles built into the program. Hard ones are
//! about 1 in 300 random puzzles, too rare to generate while a player
//! waits, so they are generated only once the bank's are all played.
//! Expert ones generate in tens of milliseconds on a laptop; they come
//! from here too, chosen so they spread over the Expert techniques, which
//! generation does not do. `data/README.md` says where the puzzles come
//! from and the rule they are chosen by.

use std::collections::HashSet;

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::grade::Grade;
use crate::grid::{Grid, House};
use crate::position::Position;
use crate::puzzle::Puzzle;
use crate::step::Step;
use crate::technique::{HiddenSingle, Level, Technique};

const BANK: &str = include_str!("../data/bank.txt");

/// A random bank puzzle of `level` whose clues aren't in `played`, or
/// `None` once every one is, or for a level the bank doesn't hold.
pub fn pick<R: Rng + ?Sized>(rng: &mut R, level: Level, played: &HashSet<Grid>) -> Option<Puzzle> {
    let puzzles: Vec<Grid> = lines()
        .filter(|(clues, l, _)| *l == level && !played.contains(clues))
        .map(|(clues, _, _)| clues)
        .collect();
    let clues = *puzzles.choose(rng)?;
    Some(Puzzle::new(clues).expect("the bank test checks every puzzle has one solution"))
}

/// Every bank puzzle's clues, with the name of its hardest technique.
pub fn puzzles() -> Vec<(Grid, &'static str)> {
    lines()
        .map(|(clues, _, grade)| (clues, grade.splitn(4, ' ').nth(3).unwrap_or("")))
        .collect()
}

/// How many positions along the solving path `steps` have a hidden single
/// in a row or column and none in a box: where a player who scans only
/// boxes is stuck. Every bank puzzle has one.
pub fn needs_lines(clues: &Grid, steps: &[Step]) -> usize {
    let mut p = Position::new(clues).known_unique();
    let mut positions = 0;
    for step in steps {
        let singles = HiddenSingle.search(&p);
        if !singles.is_empty() && !singles.iter().any(|s| matches!(s.houses[0], House::Box(_))) {
            positions += 1;
        }
        p.apply(step);
    }
    positions
}

/// The house a step's pattern lies along, for the techniques whose pattern
/// lies along one: pairs, triples and quads in a row, column or box,
/// X-Wings, Swordfish and Skyscrapers along rows or columns, and a W-Wing's
/// link in any. The bank keeps each technique's steps spread over these.
pub fn read_along(step: &Step) -> Option<House> {
    match step.technique.name() {
        "Naked pair" | "Hidden pair" | "Naked triple" | "Hidden triple" | "Naked quad"
        | "Hidden quad" | "X-Wing" | "Swordfish" | "Skyscraper" | "W-Wing" => Some(step.houses[0]),
        _ => None,
    }
}

/// The techniques of the grade's level that its solving path takes besides
/// the hardest, in name order. The bank varies these among puzzles with the
/// same hardest technique.
pub fn beside_hardest(grade: &Grade) -> Vec<&'static str> {
    let mut others: Vec<&str> = grade
        .steps
        .iter()
        .map(|s| s.technique)
        .filter(|&t| t.level() == grade.level && Some(t) != grade.hardest)
        .map(|t| t.name())
        .collect();
    others.sort_unstable();
    others.dedup();
    others
}

/// Each puzzle with its level and the grade text after the puzzle.
fn lines() -> impl Iterator<Item = (Grid, Level, &'static str)> {
    BANK.lines().filter(|l| !l.starts_with('#')).map(|line| {
        let (puzzle, grade) = line.split_once(' ').expect("a puzzle and its grade");
        let level = match grade.split(' ').next() {
            Some("Hard") => Level::Hard,
            Some("Expert") => Level::Expert,
            other => unreachable!("the bank holds Hard and Expert, not {other:?}"),
        };
        (puzzle.parse().expect("an 81-cell puzzle"), level, grade)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grade::grade;
    use crate::grid::CELLS;
    use crate::solver::{Solutions, solve};
    use crate::technique::{ORDER, ORDER_VERSION};
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn every_puzzle_grades_as_its_line_says_with_the_current_order() {
        assert_eq!(
            BANK.lines().next(),
            Some(format!("# sumi puzzle bank, technique order {ORDER_VERSION}").as_str())
        );
        for (clues, _, line) in lines() {
            assert_eq!(grade(&clues).unwrap().to_string(), line, "{clues}");
        }
    }

    #[test]
    fn every_puzzle_is_symmetric_unique_and_hard_enough() {
        let mut count = [0; 2];
        let mut solutions = HashSet::new();
        for (clues, level, _) in lines() {
            assert!(
                (0..CELLS).all(|c| clues.get(c).is_some() == clues.get(CELLS - 1 - c).is_some()),
                "{clues}"
            );
            let Solutions::Unique(solution) = solve(&clues) else {
                panic!("{clues}")
            };
            assert!(solutions.insert(solution), "{clues} shares its solution");
            let grade = grade(&clues).unwrap();
            if level == Level::Expert {
                assert!(grade.hardest_steps >= 2, "{clues}");
            }
            assert!(needs_lines(&clues, &grade.steps) > 0, "{clues}");
            count[(level == Level::Expert) as usize] += 1;
        }
        assert_eq!(count, [500, 500]);
    }

    /// Each level's techniques are the hardest step of an even share of its
    /// puzzles, give or take one. Hidden quad is too rare to fill its share,
    /// so the others share what it lacks.
    #[test]
    fn each_level_spreads_over_its_hardest_techniques() {
        for level in [Level::Hard, Level::Expert] {
            let mut counts: HashMap<&str, usize> = ORDER
                .iter()
                .filter(|t| t.level() == level)
                .map(|t| (t.name(), 0))
                .collect();
            for (clues, l, _) in lines().filter(|(_, l, _)| *l == level) {
                let hardest = grade(&clues).unwrap().hardest.unwrap().name();
                *counts
                    .get_mut(hardest)
                    .unwrap_or_else(|| panic!("{clues} {l:?}")) += 1;
            }
            let full: Vec<usize> = counts
                .iter()
                .filter(|(t, _)| **t != "Hidden quad")
                .map(|(_, &n)| n)
                .collect();
            let (min, max) = (full.iter().min().unwrap(), full.iter().max().unwrap());
            assert!(max - min <= 1, "{level:?}: {counts:?}");
            assert!(
                counts.get("Hidden quad").is_none_or(|n| n <= min),
                "{counts:?}"
            );
        }
    }

    /// The steps of each technique that lies along a house are spread over
    /// the kinds it can lie along: none holds more than half of a technique's
    /// steps or less than a fifth, or with rows and columns only, none holds
    /// more than 60%. Techniques with fewer than 50 steps are left out: over
    /// 30 steps, a third swings by 8 points either way by chance alone.
    #[test]
    fn each_technique_spreads_over_rows_columns_and_boxes() {
        for level in [Level::Hard, Level::Expert] {
            let mut kinds: HashMap<&str, [usize; 3]> = HashMap::new();
            for (clues, _, _) in lines().filter(|(_, l, _)| *l == level) {
                for step in grade(&clues).unwrap().steps {
                    if let Some(house) = read_along(&step) {
                        let i = match house {
                            House::Box(_) => 0,
                            House::Row(_) => 1,
                            House::Col(_) => 2,
                        };
                        kinds.entry(step.technique.name()).or_default()[i] += 1;
                    }
                }
            }
            for (technique, [boxes, rows, cols]) in kinds {
                let total = (boxes + rows + cols) as f64;
                if total < 50.0 {
                    continue;
                }
                let shares = [boxes, rows, cols].map(|n| n as f64 / total);
                let ok = if boxes == 0 {
                    shares[1..].iter().all(|&s| s <= 0.6)
                } else {
                    shares.iter().all(|&s| (0.2..=0.5).contains(&s))
                };
                assert!(
                    ok,
                    "{level:?} {technique}: {boxes} box, {rows} row, {cols} column"
                );
            }
        }
    }

    /// Expert puzzles with the same hardest technique vary in which other
    /// Expert techniques they take: none of those, nor taking none, is in
    /// more than 60% of them. Skyscraper comes first in the order, so its
    /// puzzles take no other.
    #[test]
    fn expert_puzzles_vary_in_the_techniques_beside_their_hardest() {
        let mut with: HashMap<&str, (usize, HashMap<&str, usize>)> = HashMap::new();
        for (clues, _, _) in lines().filter(|(_, l, _)| *l == Level::Expert) {
            let grade = grade(&clues).unwrap();
            let mut others = beside_hardest(&grade);
            if others.is_empty() {
                others.push("nothing else");
            }
            let (puzzles, companions) = with.entry(grade.hardest.unwrap().name()).or_default();
            *puzzles += 1;
            for other in others {
                *companions.entry(other).or_default() += 1;
            }
        }
        for (hardest, (puzzles, companions)) in with {
            if hardest == "Skyscraper" {
                continue;
            }
            for (other, n) in &companions {
                assert!(
                    *n * 10 <= puzzles * 6,
                    "{hardest} with {other}: {companions:?}"
                );
            }
        }
    }

    /// Every step every technique finds, along the way to solving every
    /// tenth bank puzzle, keeps the solution. These positions reach the
    /// Expert techniques far more often than generated ones do.
    #[test]
    fn no_technique_removes_the_true_digit_in_bank_puzzles() {
        for (clues, _, _) in lines().step_by(10) {
            let Solutions::Unique(solution) = solve(&clues) else {
                unreachable!()
            };
            let mut p = Position::new(&clues).known_unique();
            while let Some(step) = ORDER.iter().find_map(|t| t.find(&p)) {
                for t in ORDER {
                    for s in t.all(&p) {
                        for &(cell, digits) in &s.removals {
                            assert!(!digits.contains(solution.get(cell).unwrap()), "{s:?}");
                        }
                        if let Some((cell, digit)) = s.placement {
                            assert_eq!(solution.get(cell), Some(digit), "{s:?}");
                        }
                    }
                }
                p.apply(&step);
            }
        }
    }

    /// The first puzzle's solving path has two positions where boxes run
    /// dry and a row has a single; transposed, its path has one where a
    /// column has.
    #[test]
    fn counts_the_positions_where_only_a_row_or_column_has_a_single() {
        for (puzzle, positions) in [
            (
                "...1.26......5.29.9....6..33..6..9..1...3...8..7..8..54..5....2.26.4......52.3...",
                2,
            ),
            (
                "..931.4.........2......7.651..6..5.2.5..3..4.2.6..8..362.9......9.........3.852..",
                1,
            ),
            (
                "...6..9.7......3..7.43..2...4.5.76...6..8..1...14.6.2...3..28.6..6......1.2..8...",
                0,
            ),
        ] {
            let clues: Grid = puzzle.parse().unwrap();
            let steps = grade(&clues).unwrap().steps;
            assert_eq!(needs_lines(&clues, &steps), positions, "{puzzle}");
        }
    }

    /// Every puzzle comes with its hardest technique, named as its grade
    /// names it, 2-String Kite included; checked on every 97th.
    #[test]
    fn puzzles_name_their_hardest_technique() {
        let all = puzzles();
        assert_eq!(all.len(), 1000);
        for (clues, hardest) in all.iter().step_by(97) {
            assert_eq!(grade(clues).unwrap().hardest.unwrap().name(), *hardest);
        }
        assert!(all.iter().any(|(_, h)| *h == "2-String Kite"));
    }

    #[test]
    fn pick_gives_a_puzzle_of_the_level_asked() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
        let none = HashSet::new();
        for level in [Level::Hard, Level::Expert] {
            let puzzle = pick(&mut rng, level, &none).unwrap();
            assert_eq!(grade(puzzle.clues()).unwrap().level, level);
            assert_eq!(solve(puzzle.clues()), Solutions::Unique(*puzzle.solution()));
        }
        assert_eq!(pick(&mut rng, Level::Easy, &none), None);
    }

    /// Played puzzles aren't picked again; with all but one played, that
    /// one comes, and with all played, none.
    #[test]
    fn pick_leaves_out_the_puzzles_played() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
        let mut played: HashSet<Grid> = lines()
            .filter(|(_, l, _)| *l == Level::Expert)
            .map(|(clues, _, _)| clues)
            .collect();
        let last = *played.iter().next().unwrap();
        played.remove(&last);
        assert_eq!(
            pick(&mut rng, Level::Expert, &played).map(|p| *p.clues()),
            Some(last)
        );
        played.insert(last);
        assert_eq!(pick(&mut rng, Level::Expert, &played), None);
        assert!(pick(&mut rng, Level::Hard, &played).is_some());
    }
}
