//! How hard a puzzle is for a person: solve it with the techniques alone,
//! always taking the easiest step, and see what it took.

use std::fmt;

use crate::grid::Grid;
use crate::position::Position;
use crate::solver::{Solutions, solve};
use crate::technique::{Level, ORDER, Technique};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grade {
    /// The level of the hardest technique used, or Beyond when the
    /// techniques ran out before the puzzle was solved.
    pub level: Level,
    /// The hardest technique used, last in the order among equals. `None`
    /// when no step was taken.
    pub hardest: Option<&'static dyn Technique>,
    /// How many steps were at the hardest technique's level.
    pub hardest_steps: usize,
    /// The sum of every step's score.
    pub score: u32,
}

/// Uniqueness techniques are used only when the puzzle has exactly one
/// solution.
pub fn grade(clues: &Grid) -> Grade {
    let mut p = Position::new(clues);
    if matches!(solve(clues), Solutions::Unique(_)) {
        p = p.known_unique();
    }
    let mut grade = Grade {
        level: Level::Easy,
        hardest: None,
        hardest_steps: 0,
        score: 0,
    };
    let mut hardest_index = 0;
    while p.grid().filled() < crate::grid::CELLS {
        let Some((index, step)) = ORDER
            .iter()
            .enumerate()
            .find_map(|(i, t)| Some((i, t.find(&p)?)))
        else {
            grade.level = Level::Beyond;
            return grade;
        };
        let technique = ORDER[index];
        grade.score += technique.score();
        match grade.hardest.map(|h| h.level().cmp(&technique.level())) {
            Some(std::cmp::Ordering::Greater) => {}
            Some(std::cmp::Ordering::Equal) => {
                grade.hardest_steps += 1;
                if index > hardest_index {
                    (grade.hardest, hardest_index) = (Some(technique), index);
                }
            }
            Some(std::cmp::Ordering::Less) | None => {
                (grade.hardest, hardest_index) = (Some(technique), index);
                grade.hardest_steps = 1;
                grade.level = technique.level();
            }
        }
        p.apply(&step);
    }
    grade
}

/// `Expert 1240 2 Y-Wing`: level, score, steps at the level, hardest
/// technique. The technique comes last because its name has spaces.
impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} {}", self.level, self.score, self.hardest_steps)?;
        if let Some(t) = self.hardest {
            write!(f, " {}", t.name())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grade_of(puzzle: &str) -> Grade {
        grade(&puzzle.parse().unwrap())
    }

    #[test]
    fn a_singles_puzzle_is_easy() {
        let g = grade_of(
            "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79",
        );
        assert_eq!(g.level, Level::Easy);
    }

    /// Puzzles from grantm's public-domain bank, graded the same by
    /// research/grader_b.py, which shares no code with this module.
    #[test]
    fn the_level_is_the_hardest_technique_used() {
        let cases = [
            (
                ".18.75....954.2....4.........7.1.5...6.2.4.7...1.3.2.........6....5.713....12.74.",
                "Medium 672 3 Claiming",
            ),
            (
                "4..6.9..5..3.4.6.....7.1............7...9...1.81...53..68...95...5...2..1...7...8",
                "Hard 804 1 X-Wing",
            ),
            (
                "9.84.56.21.......8....3....8...2...7.9.....6...23.65...1.....2...3.5.9.....183...",
                "Hard 980 1 Swordfish",
            ),
            (
                ".3.6.9.....2..7..9..98..21.51....9.3....4....2.3....61.96..17..1..3..6.....9.2.3.",
                "Expert 798 1 Skyscraper",
            ),
            (
                "5...6...3.46...87...78.24...6.425.9.9.2...3.4..........7..3..4.4..5.7..9...2.6...",
                "Expert 988 3 Simple colouring",
            ),
            (
                "5.......396..2..15..26.54......8.......1.2....2.....7.27..5..9839.....57..5.9.2..",
                "Expert 762 2 Unique rectangle",
            ),
            (
                "8.......3....3..191..6.548..6...7....7..5..6....3...5..154.2..663..9....7.......8",
                "Beyond 516 1 2-String Kite",
            ),
        ];
        for (puzzle, expected) in cases {
            assert_eq!(grade_of(puzzle).to_string(), expected, "{puzzle}");
        }
    }

    #[test]
    fn a_solved_grid_needs_no_step() {
        let g = grade_of(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        );
        assert_eq!((g.level, g.hardest, g.score), (Level::Easy, None, 0));
    }
}
