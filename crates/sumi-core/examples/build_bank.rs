//! Builds `data/bank.txt` from candidate puzzle files, choosing 500 Hard and
//! 500 Expert puzzles by the spread rule in `data/README.md`. Prints the
//! bank on standard output and its counts on standard error:
//!
//! `cargo run --release -p sumi-core --example build_bank -- FILE...`
//!
//! Each line of a file may hold a puzzle as an 81-character word anywhere
//! in it, so grantm's files are read as they come.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::{env, fs, thread};

use rand::SeedableRng;
use rand::rngs::Xoshiro256PlusPlus;
use rand::seq::SliceRandom;
use sumi_core::bank::{beside_hardest, needs_lines, read_along};
use sumi_core::grade::{Grade, grade};
use sumi_core::grid::{CELLS, Grid, House};
use sumi_core::solver::{Solutions, solve};
use sumi_core::technique::{Level, ORDER, ORDER_VERSION};

const PER_LEVEL: usize = 500;

struct Candidate {
    clues: Grid,
    solution: Grid,
    grade: Grade,
    source: String,
    /// What the puzzle adds to the bank's spread: (family, member) pairs,
    /// such as ("X-Wing", "row") for an X-Wing along rows, or ("W-Wing
    /// with", "Skyscraper") for a puzzle whose hardest step is a W-Wing and
    /// which takes a Skyscraper step too.
    marks: Vec<(String, String)>,
}

fn main() {
    let mut puzzles: Vec<(Grid, String)> = Vec::new();
    let mut seen = HashSet::new();
    for path in env::args().skip(1) {
        let source = path.rsplit('/').next().unwrap().to_string();
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        for word in text.split_whitespace().filter(|w| w.len() == CELLS) {
            let Ok(clues) = word.parse::<Grid>() else {
                continue;
            };
            if symmetric(&clues) && seen.insert(clues) {
                puzzles.push((clues, source.clone()));
            }
        }
    }
    eprintln!("{} distinct symmetric puzzles", puzzles.len());

    let mut candidates = rate_all(puzzles);
    candidates.shuffle(&mut Xoshiro256PlusPlus::seed_from_u64(1));
    let mut chosen = Vec::new();
    let mut report = String::new();
    for level in [Level::Hard, Level::Expert] {
        let pool: Vec<&Candidate> = candidates
            .iter()
            .filter(|c| c.grade.level == level)
            .collect();
        chosen.extend(choose(level, &pool, &mut report));
    }

    chosen.sort_by_key(|c| (c.grade.level, c.grade.score, c.clues.to_string()));
    println!("# sumi puzzle bank, technique order {ORDER_VERSION}");
    println!("# puzzle, level, score, steps at that level, hardest technique");
    for c in &chosen {
        println!("{} {}", c.clues, c.grade);
    }
    eprint!("{report}");
}

/// Grades every puzzle on all cores and keeps those the rule allows: Hard,
/// or Expert with two Expert steps or more, and a position where only a row
/// or column has a single.
fn rate_all(puzzles: Vec<(Grid, String)>) -> Vec<Candidate> {
    let threads = thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = puzzles.len().div_ceil(threads).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = puzzles
            .chunks(chunk)
            .map(|part| s.spawn(move || part.iter().filter_map(rate).collect::<Vec<_>>()))
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

fn rate((clues, source): &(Grid, String)) -> Option<Candidate> {
    let grade = grade(clues)?;
    let allowed = match grade.level {
        Level::Hard => true,
        Level::Expert => grade.hardest_steps >= 2,
        _ => false,
    };
    if !allowed || needs_lines(clues, &grade.steps) == 0 {
        return None;
    }
    let Solutions::Unique(solution) = solve(clues) else {
        unreachable!()
    };
    let mut marks = Vec::new();
    for step in &grade.steps {
        if let Some(house) = read_along(step) {
            marks.push((step.technique.name().to_string(), kind(house).to_string()));
        }
    }
    let mut others = beside_hardest(&grade);
    if others.is_empty() {
        others.push("nothing else");
    }
    let family = format!("{} with", grade.hardest.unwrap().name());
    marks.extend(others.into_iter().map(|t| (family.clone(), t.to_string())));
    Some(Candidate {
        clues: *clues,
        solution,
        grade,
        source: source.clone(),
        marks,
    })
}

/// Picks the level's puzzles: an even share for each hardest technique, a
/// technique short of puzzles giving what it lacks to the rest, and no two
/// puzzles with one solution grid. Each pick is the candidate whose marks
/// the bank so far holds least of, against an even share of each family,
/// so shapes and the techniques that come with the hardest spread.
fn choose<'a>(level: Level, pool: &[&'a Candidate], report: &mut String) -> Vec<&'a Candidate> {
    let techniques: Vec<&str> = ORDER
        .iter()
        .filter(|t| t.level() == level)
        .map(|t| t.name())
        .collect();
    let mut by_technique: HashMap<&str, Vec<&Candidate>> = HashMap::new();
    for &c in pool {
        by_technique
            .entry(c.grade.hardest.unwrap().name())
            .or_default()
            .push(c);
    }
    let available = |t: &str| {
        let grids: HashSet<Grid> = by_technique
            .get(t)
            .map_or(HashSet::new(), |cs| cs.iter().map(|c| c.solution).collect());
        grids.len()
    };
    let quotas = shares(&techniques.iter().map(|t| available(t)).collect::<Vec<_>>());

    // A family's members, so a mark's share is weighed against an even one.
    let mut members: HashMap<&str, HashSet<&str>> = HashMap::new();
    for c in pool {
        for (family, member) in &c.marks {
            members.entry(family).or_default().insert(member);
        }
    }
    let mut counts: HashMap<(String, String), usize> = HashMap::new();
    let mut totals: HashMap<String, usize> = HashMap::new();
    let mut used_grids = HashSet::new();
    let mut chosen: Vec<&Candidate> = Vec::new();
    let mut taken = vec![0; techniques.len()];
    loop {
        let mut progressed = false;
        for (i, t) in techniques.iter().enumerate() {
            if taken[i] == quotas[i] {
                continue;
            }
            let cost = |c: &Candidate| {
                if c.marks.is_empty() {
                    return 0.0;
                }
                let sum: f64 = c
                    .marks
                    .iter()
                    .map(|m| {
                        let share = counts.get(m).copied().unwrap_or(0) as f64
                            / (totals.get(&m.0).copied().unwrap_or(0) + 1) as f64;
                        share * members[m.0.as_str()].len() as f64
                    })
                    .sum();
                sum / c.marks.len() as f64
            };
            let best = by_technique[t]
                .iter()
                .filter(|c| !used_grids.contains(&c.solution))
                .min_by(|a, b| cost(a).total_cmp(&cost(b)));
            let Some(&best) = best else { continue };
            used_grids.insert(best.solution);
            for m in &best.marks {
                *counts.entry(m.clone()).or_default() += 1;
                *totals.entry(m.0.clone()).or_default() += 1;
            }
            chosen.push(best);
            taken[i] += 1;
            progressed = true;
        }
        if !progressed {
            break;
        }
    }

    writeln!(
        report,
        "{level:?}: {} puzzles from {} candidates",
        chosen.len(),
        pool.len()
    )
    .unwrap();
    for (i, t) in techniques.iter().enumerate() {
        writeln!(
            report,
            "  hardest {t}: {} (candidates on {} grids)",
            taken[i],
            available(t)
        )
        .unwrap();
    }
    let mut sources: HashMap<&str, usize> = HashMap::new();
    for c in &chosen {
        *sources.entry(&c.source).or_default() += 1;
    }
    let mut sources: Vec<_> = sources.into_iter().collect();
    sources.sort();
    writeln!(report, "  sources: {sources:?}").unwrap();
    let mut families: Vec<_> = totals.iter().collect();
    families.sort();
    for (family, total) in families {
        let mut members: Vec<String> = counts
            .iter()
            .filter(|(m, _)| &m.0 == family)
            .map(|(m, n)| format!("{} {n} ({:.0}%)", m.1, 100.0 * *n as f64 / *total as f64))
            .collect();
        members.sort();
        writeln!(report, "  {family}: {}", members.join(", ")).unwrap();
    }
    chosen
}

/// Splits `PER_LEVEL` as evenly as it goes over techniques that have
/// `available` puzzles each; one with fewer than its share gets them all,
/// and the rest is split over the others.
fn shares(available: &[usize]) -> Vec<usize> {
    let mut quotas = vec![0; available.len()];
    let mut open: Vec<usize> = (0..available.len()).collect();
    let mut left = PER_LEVEL;
    loop {
        let share = left / open.len().max(1);
        let short: Vec<usize> = open
            .iter()
            .copied()
            .filter(|&i| available[i] < share)
            .collect();
        if short.is_empty() {
            for (n, &i) in open.iter().enumerate() {
                quotas[i] = share + usize::from(n < left % open.len());
            }
            return quotas;
        }
        for i in short {
            quotas[i] = available[i];
            left -= available[i];
            open.retain(|&j| j != i);
        }
    }
}

fn symmetric(clues: &Grid) -> bool {
    (0..CELLS).all(|c| clues.get(c).is_some() == clues.get(CELLS - 1 - c).is_some())
}

fn kind(house: House) -> &'static str {
    match house {
        House::Row(_) => "row",
        House::Col(_) => "column",
        House::Box(_) => "box",
    }
}
