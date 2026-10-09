//! New puzzles: a random full grid, then clues taken away in symmetric pairs
//! for as long as the puzzle keeps exactly one solution.

use std::collections::{HashMap, HashSet};

use rand::seq::{IndexedRandom, SliceRandom};
use rand::{Rng, RngExt};

use crate::grade::{Grade, grade};
use crate::grid::{CELLS, Grid};
use crate::puzzle::Puzzle;
use crate::solver::{Solutions, search, solve};
use crate::technique::Level;

/// A puzzle of any level sumi's techniques can finish, with its grade. A
/// Beyond puzzle is never offered, since no hint could help with it; about
/// 1 in 10 random puzzles is one.
pub fn generate_solvable<R: Rng + ?Sized>(rng: &mut R) -> (Puzzle, Grade) {
    loop {
        let puzzle = generate(rng);
        let grade = grade(puzzle.clues()).expect("generated puzzles have one solution");
        if grade.level != Level::Beyond {
            return (puzzle, grade);
        }
    }
}

/// The same generator state always gives the same puzzle. No symmetric pair
/// of clues can be removed from the result without a second solution
/// appearing. Its level is anything, Beyond included.
pub(crate) fn generate<R: Rng + ?Sized>(rng: &mut R) -> Puzzle {
    let solution = random_full_grid(rng);
    Puzzle::proven(minimise(rng, solution, &solution), solution)
}

/// Makes puzzles of one level, all different.
///
/// - Easy, Medium and Expert ones are generated until one grades at the
///   level. An Expert one must also need two Expert steps or more.
/// - Hard ones are about 1 in 300 at random. Every Hard puzzle found becomes
///   a seed, and the next are made by moving a pair of a seed's clues, which
///   gives a Hard puzzle far more often. Moving clues keeps the solution,
///   and a seed's neighbours look alike, so at most 3 puzzles share a
///   solution and a seed is retired after 50 tries or once its solution
///   has its 3. With no seed left, or when a move leaves several
///   solutions, a fresh random puzzle is tried instead.
#[derive(Clone)]
pub struct LevelGenerator {
    level: Level,
    /// Hard puzzles not yet retired, each with how often it has been tried.
    seeds: Vec<(Puzzle, u32)>,
    made: HashSet<Grid>,
    /// How many puzzles made have each solution.
    solutions: HashMap<Grid, usize>,
}

/// Tries per seed, since the puzzles one seed leads to look alike.
const TRIES_PER_SEED: u32 = 50;

/// Puzzles made with one solution, since those from one seed differ by a
/// few clues.
const PER_SOLUTION: usize = 3;

impl LevelGenerator {
    /// Panics on Beyond, which is never generated.
    pub fn new(level: Level) -> LevelGenerator {
        assert!(level != Level::Beyond, "Beyond puzzles are never generated");
        LevelGenerator {
            level,
            seeds: Vec::new(),
            made: HashSet::new(),
            solutions: HashMap::new(),
        }
    }

    /// A generator in the same state, given an rng in the same state, gives
    /// the same puzzle.
    pub fn next<R: Rng + ?Sized>(&mut self, rng: &mut R) -> (Puzzle, Grade) {
        loop {
            let puzzle = match self.level {
                Level::Hard => self.moved_seed(rng).unwrap_or_else(|| generate(rng)),
                _ => generate(rng),
            };
            if self.made_with(puzzle.solution()) >= PER_SOLUTION
                || self.made.contains(puzzle.clues())
            {
                continue;
            }
            let grade = grade(puzzle.clues()).expect("generated puzzles have one solution");
            let wanted = grade.level == self.level
                && (self.level != Level::Expert || grade.hardest_steps >= 2);
            if wanted {
                self.made.insert(*puzzle.clues());
                *self.solutions.entry(*puzzle.solution()).or_default() += 1;
                if self.level == Level::Hard {
                    self.seeds.push((puzzle.clone(), 0));
                }
                return (puzzle, grade);
            }
        }
    }

    fn made_with(&self, solution: &Grid) -> usize {
        self.solutions.get(solution).copied().unwrap_or(0)
    }

    /// A seed with one pair of clues moved, or `None` when there are no seeds
    /// left to try or the move gave more than one solution.
    fn moved_seed<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Puzzle> {
        let solutions = &self.solutions;
        self.seeds.retain(|(seed, tries)| {
            *tries < TRIES_PER_SEED
                && solutions.get(seed.solution()).copied().unwrap_or(0) < PER_SOLUTION
        });
        let i = rng.random_range(0..self.seeds.len().max(1));
        let (seed, tries) = self.seeds.get_mut(i)?;
        *tries += 1;
        move_clue_pair(rng, seed)
    }
}

/// Takes one symmetric pair of clues away and adds another from the
/// solution, then minimises, which may take away any pairs, the new one
/// included.
fn move_clue_pair<R: Rng + ?Sized>(rng: &mut R, puzzle: &Puzzle) -> Option<Puzzle> {
    let halves = 0..=CELLS / 2;
    let given: Vec<usize> = halves
        .clone()
        .filter(|&c| puzzle.clues().get(c).is_some())
        .collect();
    let empty: Vec<usize> = halves
        .filter(|&c| puzzle.clues().get(c).is_none())
        .collect();
    let (&from, &to) = (given.choose(rng)?, empty.choose(rng)?);
    let mut clues = *puzzle.clues();
    for cell in [from, CELLS - 1 - from] {
        clues.set(cell, None);
    }
    for cell in [to, CELLS - 1 - to] {
        clues.set(cell, puzzle.solution().get(cell));
    }
    if !matches!(solve(&clues), Solutions::Unique(_)) {
        return None;
    }
    let solution = *puzzle.solution();
    Some(Puzzle::proven(minimise(rng, clues, &solution), solution))
}

/// Tries taking away every symmetric pair of clues once, in random order,
/// keeping each removal that leaves one solution. `clues` must have exactly
/// that one solution.
fn minimise<R: Rng + ?Sized>(rng: &mut R, mut clues: Grid, solution: &Grid) -> Grid {
    // Cell i pairs with 80 - i, its mirror through the centre; the centre cell
    // pairs with itself.
    let mut pairs: Vec<usize> = (0..=CELLS / 2).collect();
    pairs.shuffle(rng);
    for cell in pairs {
        let mirror = CELLS - 1 - cell;
        if clues.get(cell).is_none() {
            continue;
        }
        clues.set(cell, None);
        clues.set(mirror, None);
        if !matches!(solve(&clues), Solutions::Unique(_)) {
            clues.set(cell, solution.get(cell));
            clues.set(mirror, solution.get(mirror));
        }
    }
    clues
}

fn random_full_grid<R: Rng + ?Sized>(rng: &mut R) -> Grid {
    let mut first = None;
    let mut order = || {
        let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(rng);
        digits
    };
    search(&Grid::EMPTY, 1, &mut first, &mut order);
    first.expect("an empty grid always has a solution")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::tests::is_valid_solution;
    use rand::SeedableRng;
    use rand::rngs::Xoshiro256PlusPlus;

    #[test]
    fn a_thousand_puzzles_each_have_exactly_one_solution() {
        for seed in 0..1000 {
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
            let puzzle = generate(&mut rng);
            let (clues, solution) = (*puzzle.clues(), *puzzle.solution());

            assert!(is_valid_solution(&solution), "seed {seed}");
            assert_eq!(solve(&clues), Solutions::Unique(solution), "seed {seed}");
            for cell in 0..CELLS {
                let clue = clues.get(cell);
                assert!(clue.is_none() || clue == solution.get(cell), "seed {seed}");
                assert_eq!(
                    clue.is_some(),
                    clues.get(CELLS - 1 - cell).is_some(),
                    "seed {seed}: cell {cell} is not symmetric"
                );
            }
        }
    }

    fn is_symmetric(clues: &Grid) -> bool {
        (0..CELLS).all(|c| clues.get(c).is_some() == clues.get(CELLS - 1 - c).is_some())
    }

    #[test]
    fn each_level_generates_puzzles_of_that_level() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(3);
        for level in [Level::Easy, Level::Medium, Level::Expert] {
            let mut generator = LevelGenerator::new(level);
            let (a, grade) = generator.next(&mut rng);
            assert_eq!(grade.level, level);
            assert_eq!(Some(grade.clone()), crate::grade::grade(a.clues()));
            let (b, _) = generator.next(&mut rng);
            assert_ne!(a, b);
            if level == Level::Expert {
                assert!(grade.hardest_steps >= 2);
            }
        }
    }

    #[test]
    fn a_puzzle_already_made_is_not_made_again() {
        let mut generator = LevelGenerator::new(Level::Medium);
        let rng = Xoshiro256PlusPlus::seed_from_u64(11);
        let (first, _) = generator.clone().next(&mut rng.clone());
        generator.made.insert(*first.clues());
        let (next, _) = generator.next(&mut rng.clone());
        assert_ne!(next, first);
    }

    /// A solution with 3 puzzles gets no more, and its seeds retire.
    #[test]
    fn at_most_three_puzzles_share_a_solution() {
        let rng = Xoshiro256PlusPlus::seed_from_u64(11);
        let mut generator = LevelGenerator::new(Level::Medium);
        let mut counted = generator.clone();
        let (first, _) = counted.next(&mut rng.clone());
        assert_eq!(counted.made_with(first.solution()), 1);
        generator.solutions.insert(*first.solution(), 2);
        let (again, _) = generator.clone().next(&mut rng.clone());
        assert_eq!(again, first, "two made, so a third may be");
        generator.solutions.insert(*first.solution(), 3);
        let (next, _) = generator.next(&mut rng.clone());
        assert_ne!(next.solution(), first.solution());

        let mut hard = LevelGenerator::new(Level::Hard);
        hard.seeds.push((first.clone(), 0));
        hard.solutions.insert(*first.solution(), 3);
        assert_eq!(hard.moved_seed(&mut rng.clone()), None);
        assert!(hard.seeds.is_empty());
    }

    /// Starting from a Hard puzzle in sumi's bank whose solution 50 other
    /// bank puzzles share, moved clues give Hard puzzles that are still
    /// symmetric and have one solution.
    #[test]
    fn hard_puzzles_come_from_moving_clues() {
        let clues: Grid =
            "..2......37...5..4..67....373.9.6...1...8...9...2.4.515....19..8..3...65......8.."
                .parse()
                .unwrap();
        let Solutions::Unique(solution) = solve(&clues) else {
            panic!("the seed has one solution")
        };
        let seed = Puzzle::proven(clues, solution);
        let mut generator = LevelGenerator::new(Level::Hard);
        generator.seeds.push((seed.clone(), 0));
        generator.made.insert(clues);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(5);
        let mut moved = 0;
        for _ in 0..3 {
            let (puzzle, grade) = generator.next(&mut rng);
            assert_eq!(grade.level, Level::Hard);
            assert!(is_symmetric(puzzle.clues()));
            assert_eq!(solve(puzzle.clues()), Solutions::Unique(*puzzle.solution()));
            // Moving clues keeps the solution; a fresh puzzle wouldn't.
            moved += usize::from(*puzzle.solution() == *seed.solution());
        }
        assert!(moved > 0);
    }

    #[test]
    fn solvable_puzzles_are_never_beyond() {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(4);
        for _ in 0..100 {
            let (puzzle, grade) = generate_solvable(&mut rng);
            assert_ne!(grade.level, Level::Beyond, "{}", puzzle.clues());
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_puzzle() {
        let a = generate(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        let b = generate(&mut Xoshiro256PlusPlus::seed_from_u64(7));
        let c = generate(&mut Xoshiro256PlusPlus::seed_from_u64(8));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
