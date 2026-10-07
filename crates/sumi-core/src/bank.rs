//! Graded Hard and Expert puzzles built into the program, since they are
//! too rare to generate while a player waits. `data/README.md` says where
//! they come from.

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::generate::Puzzle;
use crate::grid::Grid;
use crate::solver::{Solutions, solve};
use crate::technique::Level;

const BANK: &str = include_str!("../data/bank.txt");

/// A random bank puzzle of `level`, or `None` for a level the bank doesn't
/// hold.
pub fn pick<R: Rng + ?Sized>(rng: &mut R, level: Level) -> Option<Puzzle> {
    let puzzles: Vec<Grid> = lines()
        .filter(|(_, l, _)| *l == level)
        .map(|(clues, _, _)| clues)
        .collect();
    let clues = *puzzles.choose(rng)?;
    let Solutions::Unique(solution) = solve(&clues) else {
        unreachable!("the bank test checks every puzzle has one solution")
    };
    Some(Puzzle { clues, solution })
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
    use crate::position::Position;
    use crate::technique::{ORDER, ORDER_VERSION};
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

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
        for (clues, level, _) in lines() {
            assert!(
                (0..CELLS).all(|c| clues.get(c).is_some() == clues.get(CELLS - 1 - c).is_some()),
                "{clues}"
            );
            assert!(matches!(solve(&clues), Solutions::Unique(_)), "{clues}");
            if level == Level::Expert {
                assert!(grade(&clues).unwrap().hardest_steps >= 2, "{clues}");
            }
            count[(level == Level::Expert) as usize] += 1;
        }
        assert_eq!(count, [500, 500]);
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

    #[test]
    fn pick_gives_a_puzzle_of_the_level_asked() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(1);
        for level in [Level::Hard, Level::Expert] {
            let puzzle = pick(&mut rng, level).unwrap();
            assert_eq!(grade(&puzzle.clues).unwrap().level, level);
            assert_eq!(solve(&puzzle.clues), Solutions::Unique(puzzle.solution));
        }
        assert_eq!(pick(&mut rng, Level::Easy), None);
    }
}
