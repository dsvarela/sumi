//! The files sumi keeps between runs in its data directory, and the one
//! way it writes a file: whole, or not at all.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use sumi_core::grid::Grid;
use sumi_core::technique::Level;

use crate::game::Game;
use crate::stats::Stats;

/// The names files give the levels, apart from those the screen shows, so
/// renaming a level on screen keeps old files readable.
const LEVELS: [(Level, &str); 5] = [
    (Level::Easy, "easy"),
    (Level::Medium, "medium"),
    (Level::Hard, "hard"),
    (Level::Expert, "expert"),
    (Level::Beyond, "beyond"),
];

pub fn level_key(level: Level) -> &'static str {
    LEVELS
        .iter()
        .find(|(l, _)| *l == level)
        .expect("every level")
        .1
}

pub fn level_from_key(key: &str) -> Option<Level> {
    LEVELS.iter().find(|(_, k)| *k == key).map(|(l, _)| *l)
}

/// Where sumi keeps the saved game, if the system has a home directory.
pub fn dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "sumi").map(|dirs| dirs.data_dir().to_path_buf())
}

/// The game in progress, the one autosave slot.
const GAME: &str = "game.json";

/// Saves `json`, a game, as the game in progress.
pub fn save_game(dir: &Path, json: &str) -> Result<(), String> {
    write(dir, GAME, json)
}

/// The game in progress, if one is saved. A saved game that can't be read
/// is moved aside, so the next save doesn't destroy it.
pub fn load_game(dir: &Path) -> Result<Option<Game>, String> {
    let Some(json) = read(dir, GAME)? else {
        return Ok(None);
    };
    Game::from_json(&json)
        .map(Some)
        .map_err(|why| move_aside(dir, GAME, &why))
}

/// Forgets the game in progress, once it is won.
pub fn remove_game(dir: &Path) -> Result<(), String> {
    let path = dir.join(GAME);
    match std::fs::remove_file(&path) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("{}: {err}", path.display()))
        }
        _ => Ok(()),
    }
}

/// The bank puzzles played, by their clues, so none comes twice.
const PLAYED: &str = "played.json";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Played {
    played: Vec<String>,
}

/// The bank puzzles played, none if nothing is saved.
pub fn load_played(dir: &Path) -> Result<HashSet<Grid>, String> {
    let Some(json) = read(dir, PLAYED)? else {
        return Ok(HashSet::new());
    };
    let named = |why: String| format!("{}: {why}", dir.join(PLAYED).display());
    let played: Played = serde_json::from_str(&json).map_err(|err| named(err.to_string()))?;
    played
        .played
        .iter()
        .map(|clues| {
            clues
                .parse()
                .map_err(|_| named(format!("{clues:?} isn't a grid")))
        })
        .collect()
}

pub fn save_played(dir: &Path, played: &HashSet<Grid>) -> Result<(), String> {
    let mut clues: Vec<String> = played.iter().map(Grid::to_string).collect();
    clues.sort();
    let json = serde_json::to_string(&Played { played: clues }).expect("strings make JSON");
    write(dir, PLAYED, &json)
}

/// Every game solved, for the stats.
const STATS: &str = "stats.json";

/// The stats kept, none if nothing is saved. Stats that can't be read are
/// moved aside, so the next game solved doesn't destroy them.
pub fn load_stats(dir: &Path) -> Result<Stats, String> {
    let Some(json) = read(dir, STATS)? else {
        return Ok(Stats::default());
    };
    serde_json::from_str(&json).map_err(|err| move_aside(dir, STATS, &err.to_string()))
}

pub fn save_stats(dir: &Path, stats: &Stats) -> Result<(), String> {
    write(
        dir,
        STATS,
        &serde_json::to_string(stats).expect("stats make JSON"),
    )
}

/// The text of the file `name` in `dir`, or `None` if there is none.
fn read(dir: &Path, name: &str) -> Result<Option<String>, String> {
    let path = dir.join(name);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("{}: {err}", path.display())),
    }
}

/// Writes `text` as the file `name` in `dir`, making `dir` if need be.
fn write(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let path = dir.join(name);
    std::fs::create_dir_all(dir)
        .and_then(|()| write_whole(&path, text))
        .map_err(|err| format!("{}: {err}", path.display()))
}

/// Moves the file `name`, which couldn't be read because of `why`, to
/// `name.unreadable`, and says so.
fn move_aside(dir: &Path, name: &str, why: &str) -> String {
    let (path, aside) = (dir.join(name), dir.join(format!("{name}.unreadable")));
    match std::fs::rename(&path, &aside) {
        Ok(()) => format!("{}: {why}; moved to {}", path.display(), aside.display()),
        Err(err) => format!("{}: {why}; couldn't move it aside: {err}", path.display()),
    }
}

/// Writes `text` to `path` all at once: into a file beside it, flushed to
/// the disk, then renamed over it, so a crash, a power cut or a full disk
/// never leaves half a file. A symlink is followed, so the file it points
/// to is the one replaced. The file beside it is named for this process, so
/// two sumis writing at once don't write into each other's.
pub fn write_whole(path: &Path, text: &str) -> std::io::Result<()> {
    let target = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut temporary = target.clone().into_os_string();
    temporary.push(format!(".sumi-new-{}", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written =
        write_synced(&temporary, text, &target).and_then(|()| std::fs::rename(&temporary, &target));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
}

/// Writes `text` to a new file at `path` with `like`'s permissions, if
/// `like` exists, and waits for it to reach the disk.
fn write_synced(path: &Path, text: &str, like: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    if let Ok(metadata) = std::fs::metadata(like) {
        file.set_permissions(metadata.permissions())?;
    }
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shorter text replaces a longer one entirely, and the file written
    /// beside it is gone.
    #[test]
    fn a_write_replaces_the_whole_file_and_leaves_nothing_beside_it() {
        let dir = std::env::temp_dir().join(format!("sumi-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("game.json");
        write_whole(&path, "a longer first text").unwrap();
        write_whole(&path, "short").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let left: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(text, "short");
        assert_eq!(left, ["game.json"]);
    }

    /// A saved game that can't be read is moved aside, not left for the
    /// next save to overwrite.
    #[test]
    fn an_unreadable_save_is_moved_aside() {
        let dir = std::env::temp_dir().join(format!("sumi-unreadable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("game.json"), "{").unwrap();
        let err = load_game(&dir).err().unwrap();
        let aside = std::fs::read_to_string(dir.join("game.json.unreadable"));
        let again = load_game(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(err.contains("moved to"), "{err}");
        assert_eq!(aside.unwrap(), "{");
        assert!(matches!(again, Ok(None)));
    }

    /// The puzzles played read back as written; none before any is.
    #[test]
    fn the_puzzles_played_read_back() {
        let dir = std::env::temp_dir().join(format!("sumi-played-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let none = load_played(&dir);
        let played: HashSet<Grid> = [crate::fixtures::CLUES.parse().unwrap(), Grid::EMPTY].into();
        save_played(&dir, &played).unwrap();
        let back = load_played(&dir);
        std::fs::write(dir.join(PLAYED), r#"{"played":["12"]}"#).unwrap();
        let broken = load_played(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(none, Ok(HashSet::new()));
        assert_eq!(back, Ok(played));
        assert!(broken.unwrap_err().contains("isn't a grid"));
    }

    /// Stats read back as written; stats that can't be read are moved
    /// aside rather than overwritten.
    #[test]
    fn the_stats_read_back_and_are_never_overwritten_unread() {
        let dir = std::env::temp_dir().join(format!("sumi-stats-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let none = load_stats(&dir);
        let mut stats = Stats::default();
        let game = crate::fixtures::game().rated(Level::Hard);
        stats.record(&game, std::time::Duration::from_secs(90));
        save_stats(&dir, &stats).unwrap();
        let back = load_stats(&dir);
        std::fs::write(dir.join(STATS), "{").unwrap();
        let broken = load_stats(&dir);
        let aside = std::fs::read_to_string(dir.join("stats.json.unreadable"));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(none, Ok(Stats::default()));
        assert_eq!(back, Ok(stats));
        assert!(broken.unwrap_err().contains("moved to"));
        assert_eq!(aside.unwrap(), "{");
    }
}
