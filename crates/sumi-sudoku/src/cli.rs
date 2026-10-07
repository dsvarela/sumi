//! The command line: `sumi` plays, `sumi rate` and `sumi gen` work on puzzles.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use rand::SeedableRng;
use rand::rngs::Xoshiro256PlusPlus;
use sumi_core::generate::{LevelGenerator, generate};
use sumi_core::grade::grade;
use sumi_core::grid::Grid;
use sumi_core::solver::{Solutions, solve};
use sumi_core::technique::Level;

#[derive(Parser)]
#[command(
    name = "sumi",
    version,
    about = "A calm terminal Sudoku game that teaches"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Play puzzles of one level.
    New {
        #[arg(value_enum)]
        level: LevelArg,
    },
    /// Say whether a puzzle has one solution, and how hard it is. With no
    /// puzzle, rate each line of standard input, one per output line.
    Rate {
        /// 81 cells, row by row: 1 to 9, with 0 or . for an empty cell.
        puzzle: Option<String>,
    },
    /// Print new puzzles, one per line.
    Gen {
        /// How many puzzles to print.
        #[arg(short = 'n', long, default_value_t = 1)]
        count: u32,
        /// Start from this seed to get the same puzzles again.
        #[arg(long)]
        seed: Option<u64>,
        /// Only puzzles of this level. Expert ones need two Expert steps or
        /// more.
        #[arg(long, value_enum)]
        level: Option<LevelArg>,
    },
}

/// The levels a puzzle can be asked for. Beyond is never generated.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum LevelArg {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl From<LevelArg> for Level {
    fn from(level: LevelArg) -> Level {
        match level {
            LevelArg::Easy => Level::Easy,
            LevelArg::Medium => Level::Medium,
            LevelArg::Hard => Level::Hard,
            LevelArg::Expert => Level::Expert,
        }
    }
}

pub fn run() -> ExitCode {
    match Cli::parse().command {
        None => play(None),
        Some(Command::New { level }) => play(Some(level.into())),
        Some(Command::Rate {
            puzzle: Some(puzzle),
        }) => rate(&puzzle),
        Some(Command::Rate { puzzle: None }) => {
            piped(rate_lines(io::stdin().lock(), &mut io::stdout().lock()))
        }
        Some(Command::Gen { count, seed, level }) => {
            piped(gen_puzzles(count, seed, level.map(Level::from)))
        }
    }
}

/// The exit code of a command that writes to standard output.
fn piped(result: io::Result<bool>) -> ExitCode {
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        // A reader that stops early, like `head`, closes the pipe: that ends
        // the output, it isn't a failure.
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("sumi: {err}");
            ExitCode::FAILURE
        }
    }
}

fn play(level: Option<Level>) -> ExitCode {
    match crate::app::run(level) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("sumi: {err}");
            ExitCode::FAILURE
        }
    }
}

/// A puzzle without exactly one solution is a valid answer, not an error;
/// only input that isn't a grid fails.
fn rate(input: &str) -> ExitCode {
    let grid: Grid = match input.parse() {
        Ok(grid) => grid,
        Err(err) => {
            eprintln!("sumi: {err}");
            return ExitCode::FAILURE;
        }
    };
    let grade = grade(&grid);
    let verdict = match solve(&grid) {
        Solutions::None => "no solution",
        Solutions::Unique(_) => "unique",
        Solutions::Multiple => "more than one solution",
    };
    let hardest = grade.hardest.map_or("no step", |t| t.name());
    let steps = match grade.hardest_steps {
        1 => "1 step".to_string(),
        n => format!("{n} steps"),
    };
    println!(
        "{verdict}, {} clues, {:?}: {hardest} ({steps} at that level, score {})",
        grid.filled(),
        grade.level,
        grade.score
    );
    ExitCode::SUCCESS
}

/// Rates the first word of each line, printing the puzzle and its grade, or
/// why it has none. A line that isn't a puzzle is reported on standard error,
/// and the rest are still rated. Returns whether every line was a puzzle.
fn rate_lines(input: impl BufRead, out: &mut impl Write) -> io::Result<bool> {
    let mut all_read = true;
    for (number, line) in input.lines().enumerate() {
        let line = line?;
        let Some(word) = line.split_whitespace().next() else {
            continue;
        };
        let grid: Grid = match word.parse() {
            Ok(grid) => grid,
            Err(err) => {
                eprintln!("sumi: line {}: {err}", number + 1);
                all_read = false;
                continue;
            }
        };
        match solve(&grid) {
            Solutions::Unique(_) => writeln!(out, "{grid} {}", grade(&grid))?,
            Solutions::None => writeln!(out, "{grid} no solution")?,
            Solutions::Multiple => writeln!(out, "{grid} more than one solution")?,
        }
    }
    out.flush()?;
    Ok(all_read)
}

/// Prints the clues only; `sumi rate` adds the grade.
fn gen_puzzles(count: u32, seed: Option<u64>, level: Option<Level>) -> io::Result<bool> {
    let mut rng = match seed {
        Some(seed) => Xoshiro256PlusPlus::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    let mut out = io::stdout().lock();
    let mut generator = level.map(LevelGenerator::new);
    for _ in 0..count {
        let puzzle = match &mut generator {
            Some(generator) => generator.next(&mut rng).0,
            None => generate(&mut rng),
        };
        writeln!(out, "{}", puzzle.clues)?;
    }
    out.flush()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_reads_a_list_and_skips_lines_that_are_not_puzzles() {
        let input = "\
53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79 easy
not a puzzle

.................................................................................
";
        let mut out = Vec::new();
        let all_read = rate_lines(input.as_bytes(), &mut out).unwrap();
        assert!(!all_read);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "\
53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79 Easy 504 51 Hidden single
................................................................................. more than one solution
"
        );
    }
}
