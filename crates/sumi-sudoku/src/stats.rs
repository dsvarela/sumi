//! The stats: for each level, the games solved, the best and average time
//! and the hints per game, and which techniques hints took most. Kept for
//! every game solved; shown only once turned on. Checks (`c`) and reveals
//! (`r`) aren't counted.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sumi_core::technique::Level;

use crate::game::Game;
use crate::store;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stats {
    /// By the level's name in files.
    #[serde(default)]
    levels: BTreeMap<String, Solved>,
    /// How many hints took each technique, by its name.
    #[serde(default)]
    hint_techniques: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Solved {
    games: u32,
    best_ms: u64,
    total_ms: u64,
    hints: u32,
}

/// One level's line in the stats, for a level with games solved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub level: Level,
    pub solved: u32,
    pub best: Duration,
    pub average: Duration,
    pub hints_per_game: f32,
}

impl Stats {
    /// Counts `game`, solved in `time`. A game without a rating isn't
    /// counted.
    pub fn record(&mut self, game: &Game, time: Duration) {
        let Some(level) = game.level() else {
            return;
        };
        let ms = time.as_millis() as u64;
        let solved = self
            .levels
            .entry(store::level_key(level).to_string())
            .or_default();
        solved.best_ms = if solved.games == 0 {
            ms
        } else {
            solved.best_ms.min(ms)
        };
        // A hand-edited file can hold any count; one more doesn't wrap.
        solved.games = solved.games.saturating_add(1);
        solved.total_ms = solved.total_ms.saturating_add(ms);
        solved.hints = solved.hints.saturating_add(game.hints_used());
        for (technique, count) in game.hint_techniques() {
            let total = self.hint_techniques.entry(technique.clone()).or_default();
            *total = total.saturating_add(*count);
        }
    }

    /// A line for each level with games solved, easiest first.
    pub fn rows(&self) -> Vec<Row> {
        let mut rows: Vec<Row> = self
            .levels
            .iter()
            .filter_map(|(key, solved)| {
                let level = store::level_from_key(key)?;
                (solved.games > 0).then(|| Row {
                    level,
                    solved: solved.games,
                    best: Duration::from_millis(solved.best_ms),
                    average: Duration::from_millis(solved.total_ms / u64::from(solved.games)),
                    hints_per_game: solved.hints as f32 / solved.games as f32,
                })
            })
            .collect();
        rows.sort_by_key(|row| row.level);
        rows
    }

    /// The `n` techniques hints took most, with how often, most first.
    pub fn most_hinted(&self, n: usize) -> Vec<(&str, u32)> {
        let mut techniques: Vec<(&str, u32)> = self
            .hint_techniques
            .iter()
            .map(|(name, &count)| (name.as_str(), count))
            .collect();
        techniques.sort_by_key(|&(name, count)| (std::cmp::Reverse(count), name));
        techniques.truncate(n);
        techniques
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use sumi_core::hint::Hint;

    fn solved(level: Level, hints: u32) -> Game {
        let mut game = fixtures::game().rated(level);
        for _ in 0..hints {
            let hint = game.hint();
            game.count_hint(&hint);
        }
        game
    }

    /// Each level keeps its count, its best and average time and its hints
    /// per game; a game without a rating isn't counted.
    #[test]
    fn a_solved_game_counts_in_its_level() {
        let mut stats = Stats::default();
        let secs = Duration::from_secs;
        stats.record(&solved(Level::Hard, 2), secs(600));
        stats.record(&solved(Level::Hard, 1), secs(300));
        stats.record(&solved(Level::Easy, 0), secs(120));
        stats.record(&fixtures::game(), secs(1));
        assert_eq!(
            stats.rows(),
            [
                Row {
                    level: Level::Easy,
                    solved: 1,
                    best: secs(120),
                    average: secs(120),
                    hints_per_game: 0.0
                },
                Row {
                    level: Level::Hard,
                    solved: 2,
                    best: secs(300),
                    average: secs(450),
                    hints_per_game: 1.5
                },
            ]
        );
        assert_eq!(stats.most_hinted(5), [("Hidden single", 3)]);
    }

    /// A file can hold any count; one more game stops at the most a count
    /// holds rather than wrapping or stopping sumi.
    #[test]
    fn counts_stop_at_their_most() {
        let mut stats = Stats::default();
        stats.levels.insert(
            "hard".to_string(),
            Solved {
                games: u32::MAX,
                best_ms: 1,
                total_ms: u64::MAX,
                hints: u32::MAX,
            },
        );
        stats.record(&solved(Level::Hard, 1), Duration::from_secs(5));
        let row = stats.rows()[0];
        assert_eq!(row.solved, u32::MAX);
        assert_eq!(row.best, Duration::from_millis(1));
    }

    /// Hints that name no technique, as a mistake's, count as hints and
    /// not under any technique; the most hinted come most first, then by
    /// name.
    #[test]
    fn techniques_are_counted_by_the_hints_that_took_them() {
        let mut game = fixtures::game().rated(Level::Easy);
        game.count_hint(&Hint::Mistake(2));
        let mut stats = Stats::default();
        stats.record(&game, Duration::ZERO);
        assert_eq!(stats.rows()[0].hints_per_game, 1.0);
        assert!(stats.most_hinted(5).is_empty());
        stats.hint_techniques = [("X-Wing", 5), ("Naked pair", 2), ("Full house", 2)]
            .map(|(name, count)| (name.to_string(), count))
            .into();
        assert_eq!(stats.most_hinted(2), [("X-Wing", 5), ("Full house", 2)]);
    }
}
