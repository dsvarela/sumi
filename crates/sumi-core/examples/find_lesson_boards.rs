//! Finds boards for the lessons: positions on the way to solving a puzzle
//! where, with every candidate the digits allow, the easiest step is one of
//! the 7 Easy and Medium techniques. Prints, for each technique, the
//! boards with one instance of it and with two or three, the most empty
//! cells first:
//!
//! `cargo run --release -p sumi-core --example find_lesson_boards`
//!
//! A lesson file takes its boards from here; its test checks them again.

use std::collections::BTreeMap;

use rand::SeedableRng;
use rand::rngs::Xoshiro256PlusPlus;
use sumi_core::generate::generate_solvable;
use sumi_core::grade::grade;
use sumi_core::grid::Grid;
use sumi_core::position::Position;
use sumi_core::technique::{Level, ORDER};

/// Generated puzzles to search, after the bank's.
const GENERATED: u64 = 3000;
/// Boards printed per technique and instance count.
const SHOWN: usize = 6;

fn main() {
    let bank = include_str!("../data/bank.txt");
    let mut puzzles: Vec<Grid> = bank
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| l.split(' ').next()?.parse().ok())
        .collect();
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
    puzzles.extend((0..GENERATED).map(|_| *generate_solvable(&mut rng).0.clues()));

    // technique -> (one instance?, empty cells, board)
    let mut found: BTreeMap<&str, Vec<(bool, usize, String)>> = BTreeMap::new();
    for clues in &puzzles {
        let Some(graded) = grade(clues) else { continue };
        let mut p = Position::new(clues).known_unique();
        for step in &graded.steps {
            let fresh = Position::new(p.grid()).known_unique();
            let easiest = ORDER.iter().find(|t| !t.all(&fresh).is_empty());
            if let Some(t) = easiest.filter(|t| t.level() <= Level::Medium) {
                let instances = t.all(&fresh).len();
                if instances <= 3 {
                    let empty = 81 - p.grid().filled();
                    let entry = (instances == 1, empty, p.grid().to_string());
                    found.entry(t.name()).or_default().push(entry);
                }
            }
            p.apply(step);
        }
    }
    for (name, mut boards) in found {
        boards.sort_by_key(|b| std::cmp::Reverse(b.1));
        boards.dedup_by(|a, b| a.2 == b.2);
        for one in [true, false] {
            let label = if one { "one instance" } else { "two or three" };
            println!("# {name}, {label}");
            for (_, empty, board) in boards.iter().filter(|b| b.0 == one).take(SHOWN) {
                println!("{board} {empty} empty");
            }
        }
    }
}
