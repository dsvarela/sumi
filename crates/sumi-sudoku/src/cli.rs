//! The command line: `sumi rate` and `sumi gen`.

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
    command: Command,
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
        Command::Rate { puzzle } => rate(&puzzle),
        Command::Gen { count, seed } => {
            gen_puzzles(count, seed);
            ExitCode::SUCCESS
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

fn gen_puzzles(count: u32, seed: Option<u64>) {
    let mut rng = match seed {
        Some(seed) => Xoshiro256PlusPlus::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    for _ in 0..count {
        println!("{}", generate(&mut rng).clues);
    }
}
