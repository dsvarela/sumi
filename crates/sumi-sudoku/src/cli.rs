//! The command line: `sumi` plays, `sumi rate` and `sumi gen` work on puzzles.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rand::SeedableRng;
use rand::rngs::Xoshiro256PlusPlus;
use sumi_core::generate::generate;
use sumi_core::grid::Grid;
use sumi_core::solver::{Solutions, solve};

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
    /// Check that a puzzle has exactly one solution.
    Rate {
        /// 81 cells, row by row: 1 to 9, with 0 or . for an empty cell.
        puzzle: String,
    },
    /// Print new puzzles, one per line.
    Gen {
        /// How many puzzles to print.
        #[arg(short = 'n', long, default_value_t = 1)]
        count: u32,
        /// Start from this seed to get the same puzzles again.
        #[arg(long)]
        seed: Option<u64>,
    },
}

pub fn run() -> ExitCode {
    match Cli::parse().command {
        None => play(),
        Some(Command::Rate { puzzle }) => rate(&puzzle),
        Some(Command::Gen { count, seed }) => match gen_puzzles(count, seed) {
            Ok(()) => ExitCode::SUCCESS,
            // A reader that stops early, like `head`, closes the pipe: that
            // ends the output, it isn't a failure.
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("sumi: {err}");
                ExitCode::FAILURE
            }
        },
    }
}

fn play() -> ExitCode {
    match crate::app::run() {
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
    let verdict = match solve(&grid) {
        Solutions::None => "no solution",
        Solutions::Unique(_) => "unique",
        Solutions::Multiple => "more than one solution",
    };
    println!("{verdict}, {} clues", grid.filled());
    ExitCode::SUCCESS
}

fn gen_puzzles(count: u32, seed: Option<u64>) -> io::Result<()> {
    let mut rng = match seed {
        Some(seed) => Xoshiro256PlusPlus::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    let mut out = io::stdout().lock();
    for _ in 0..count {
        writeln!(out, "{}", generate(&mut rng).clues)?;
    }
    out.flush()
}
