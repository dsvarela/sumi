//! The files sumi keeps between runs in its data directory, and the one
//! way it writes a file: whole, or not at all.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

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

/// Where sumi keeps the saved game, the bank puzzles played and the
/// stats, if the system has a home directory.
pub fn dir() -> Option<PathBuf> {
    xdg_dir("XDG_DATA_HOME", ".local/share")
}

/// sumi's folder under the XDG base directory `var` names, or under
/// `fallback` in the home directory when `var` is unset or relative, as
/// the XDG spec asks. macOS follows the same rule, so config and saves
/// stay apart there too.
pub fn xdg_dir(var: &str, fallback: &str) -> Option<PathBuf> {
    under_base(
        std::env::var_os(var).map(PathBuf::from),
        std::env::home_dir(),
        fallback,
    )
}

fn under_base(base: Option<PathBuf>, home: Option<PathBuf>, fallback: &str) -> Option<PathBuf> {
    let base = match base {
        Some(base) if base.is_absolute() => base,
        _ => home?.join(fallback),
    };
    Some(base.join("sumi"))
}

/// A file that was there but couldn't be read.
#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub message: String,
    /// It couldn't be moved aside either, so writing the file would
    /// destroy it.
    pub in_place: bool,
}

/// The game in progress, the one autosave slot.
const GAME: &str = "game.json";

/// Saves `json`, a game, as the game in progress.
pub fn save_game(dir: &Path, json: &str) -> Result<(), String> {
    write(dir, GAME, json)
}

/// The game in progress, if one is saved.
pub fn load_game(dir: &Path) -> Result<Option<Game>, Unreadable> {
    load(dir, GAME, Game::from_json)
}

/// Forgets the game in progress, once it is won.
pub fn remove_game(dir: &Path) -> Result<(), String> {
    let path = dir.join(GAME);
    let _writing = WRITES.lock().unwrap_or_else(PoisonError::into_inner);
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
pub fn load_played(dir: &Path) -> Result<HashSet<Grid>, Unreadable> {
    let parse = |json: &str| -> Result<HashSet<Grid>, String> {
        let played: Played = serde_json::from_str(json).map_err(|err| err.to_string())?;
        played
            .played
            .iter()
            .map(|clues| clues.parse().map_err(|_| format!("{clues:?} isn't a grid")))
            .collect()
    };
    Ok(load(dir, PLAYED, parse)?.unwrap_or_default())
}

/// Saves `ours` with the puzzles the file holds now, so another sumi's
/// are kept too, and gives back them all.
pub fn save_played(dir: &Path, ours: &HashSet<Grid>) -> Result<HashSet<Grid>, String> {
    let mut played = load_played(dir).map_err(|err| err.message)?;
    played.extend(ours);
    let mut all: Vec<String> = played.iter().map(Grid::to_string).collect();
    all.sort();
    let json = serde_json::to_string(&Played { played: all }).expect("strings make JSON");
    write(dir, PLAYED, &json)?;
    Ok(played)
}

/// Every game solved, for the stats.
const STATS: &str = "stats.json";

/// The stats kept, none if nothing is saved.
pub fn load_stats(dir: &Path) -> Result<Stats, Unreadable> {
    let parse = |json: &str| serde_json::from_str(json).map_err(|err| err.to_string());
    Ok(load(dir, STATS, parse)?.unwrap_or_default())
}

/// Changes the stats as the file holds them now, so another sumi's games
/// are kept too, and gives back the stats changed.
pub fn update_stats(dir: &Path, change: impl FnOnce(&mut Stats)) -> Result<Stats, String> {
    let mut stats = load_stats(dir).map_err(|err| err.message)?;
    change(&mut stats);
    write(
        dir,
        STATS,
        &serde_json::to_string(&stats).expect("stats make JSON"),
    )?;
    Ok(stats)
}

/// The file `name` in `dir` read by `parse`, or `None` if there is none.
/// One that can't be read is moved aside, so the next write doesn't
/// destroy it.
fn load<T>(
    dir: &Path,
    name: &str,
    parse: impl FnOnce(&str) -> Result<T, String>,
) -> Result<Option<T>, Unreadable> {
    let path = dir.join(name);
    let why = match std::fs::read_to_string(&path) {
        Ok(text) => match parse(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(why) => why,
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => err.to_string(),
    };
    Err(move_aside(dir, name, &why))
}

/// Writes `text` as the file `name` in `dir`, making `dir` if need be.
fn write(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    let path = dir.join(name);
    std::fs::create_dir_all(dir)
        .and_then(|()| write_whole(&path, text))
        .map_err(|err| format!("{}: {err}", path.display()))
}

/// Moves the file `name`, which couldn't be read because of `why`, to
/// `name.unreadable`, or `name.unreadable-2` and on if that is taken, so
/// nothing moved aside before is lost.
fn move_aside(dir: &Path, name: &str, why: &str) -> Unreadable {
    let path = dir.join(name);
    let aside = (1..)
        .map(|n| match n {
            1 => dir.join(format!("{name}.unreadable")),
            n => dir.join(format!("{name}.unreadable-{n}")),
        })
        .find(|aside| std::fs::symlink_metadata(aside).is_err())
        .expect("a free name");
    match std::fs::rename(&path, &aside) {
        Ok(()) => Unreadable {
            message: format!("{}: {why}; moved to {}", path.display(), aside.display()),
            in_place: false,
        },
        Err(err) => Unreadable {
            message: format!("{}: {why}; couldn't move it aside: {err}", path.display()),
            in_place: true,
        },
    }
}

/// Held while a file is written, so the signal thread, which holds it to
/// the end, never stops sumi with a file half written beside its target.
static WRITES: Mutex<()> = Mutex::new(());

/// What the signal thread holds once it has stopped every other write.
pub struct LastWrite {
    _writes: MutexGuard<'static, ()>,
}

/// Waits for any write under way, and stops every other.
pub fn last_write() -> LastWrite {
    LastWrite {
        _writes: WRITES.lock().unwrap_or_else(PoisonError::into_inner),
    }
}

impl LastWrite {
    pub fn save_game(&self, dir: &Path, json: &str) -> Result<(), String> {
        let path = dir.join(GAME);
        std::fs::create_dir_all(dir)
            .and_then(|()| write_unlocked(&path, json))
            .map_err(|err| format!("{}: {err}", path.display()))
    }
}

/// Writes `text` to `path` all at once: into a file beside it, flushed to
/// the disk, then renamed over it, so a crash, a power cut or a full disk
/// never leaves half a file. A symlink is followed, even to a file not
/// made yet, so the file it points to is the one written. The file beside
/// it is named for this process, so two sumis writing at once don't write
/// into each other's.
pub fn write_whole(path: &Path, text: &str) -> std::io::Result<()> {
    let _writing = WRITES.lock().unwrap_or_else(PoisonError::into_inner);
    write_unlocked(path, text)
}

fn write_unlocked(path: &Path, text: &str) -> std::io::Result<()> {
    let target = followed(path);
    let mut temporary = target.clone().into_os_string();
    temporary.push(format!(".sumi-new-{}", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written =
        write_synced(&temporary, text, &target).and_then(|()| std::fs::rename(&temporary, &target));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
        return written;
    }
    // The rename itself reaches the disk with the directory.
    if let Some(parent) = target.parent()
        && let Ok(dir) = std::fs::File::open(parent)
    {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// The file `path` names once its symlinks are followed, links to a file
/// not made yet included.
fn followed(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    for _ in 0..40 {
        match std::fs::read_link(&path) {
            Ok(to) => {
                path = match path.parent() {
                    Some(parent) if to.is_relative() => parent.join(to),
                    _ => to,
                }
            }
            Err(_) => break,
        }
    }
    path
}

/// Writes `text` to a new file at `path` with `like`'s permissions, if
/// `like` exists, and waits for it to reach the disk. The owner can always
/// read and write it, so a save never makes the next one unreadable.
fn write_synced(path: &Path, text: &str, like: &Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    if let Ok(metadata) = std::fs::metadata(like) {
        let mut permissions = metadata.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(permissions.mode() | 0o600);
        }
        file.set_permissions(permissions)?;
    }
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_base_directory_is_used_only_when_absolute() {
        let home = Some(PathBuf::from("/home/p"));
        let under = |base: Option<&str>, home: Option<PathBuf>| {
            under_base(base.map(PathBuf::from), home, ".local/share")
        };
        assert_eq!(
            under(Some("/x/data"), home.clone()),
            Some(PathBuf::from("/x/data/sumi"))
        );
        assert_eq!(
            under(Some("x/data"), home.clone()),
            Some(PathBuf::from("/home/p/.local/share/sumi"))
        );
        assert_eq!(
            under(Some(""), home.clone()),
            Some(PathBuf::from("/home/p/.local/share/sumi"))
        );
        assert_eq!(
            under(None, home),
            Some(PathBuf::from("/home/p/.local/share/sumi"))
        );
        assert_eq!(
            under(Some("/x/data"), None),
            Some(PathBuf::from("/x/data/sumi"))
        );
        assert_eq!(under(None, None), None);
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sumi-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// A shorter text replaces a longer one entirely, and the file written
    /// beside it is gone; a write that fails leaves the file as it was.
    #[test]
    fn a_write_replaces_the_whole_file_or_nothing() {
        let dir = scratch("whole");
        let path = dir.join("game.json");
        write_whole(&path, "a longer first text").unwrap();
        write_whole(&path, "short").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let left = names(&dir);
        // A directory where the file beside it would go makes the write
        // fail after the target exists.
        let blocker = dir.join(format!("game.json.sumi-new-{}", std::process::id()));
        std::fs::create_dir(&blocker).unwrap();
        let failed = write_whole(&path, "lost");
        let kept = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(text, "short");
        assert_eq!(left, ["game.json"]);
        assert!(failed.is_err());
        assert_eq!(kept, "short");
    }

    /// The owner can always read back what was written, whatever the old
    /// file's mode; and a symlink to a file not made yet is followed.
    #[cfg(unix)]
    #[test]
    fn a_write_stays_readable_and_follows_a_dangling_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("modes");
        let path = dir.join("stats.json");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o244)).unwrap();
        write_whole(&path, "new").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        let link = dir.join("game.json");
        std::os::unix::fs::symlink("elsewhere.json", &link).unwrap();
        write_whole(&link, "linked").unwrap();
        let still_link = std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink();
        let target = std::fs::read_to_string(dir.join("elsewhere.json"));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(mode, 0o644);
        assert!(still_link);
        assert_eq!(target.unwrap(), "linked");
    }

    /// A file that can't be read, as JSON or even as text, is moved
    /// aside, to a new name each time, not left for the next write.
    #[test]
    fn unreadable_files_are_moved_aside_without_losing_older_ones() {
        let dir = scratch("unreadable");
        std::fs::write(dir.join("game.json"), "{").unwrap();
        std::fs::write(dir.join("stats.json"), [b'{', 0xFF]).unwrap();
        std::fs::write(dir.join("played.json"), r#"{"played":["12"]}"#).unwrap();
        let game = load_game(&dir).err().unwrap();
        let stats = load_stats(&dir).unwrap_err();
        let played = load_played(&dir).unwrap_err();
        std::fs::write(dir.join("game.json"), "}").unwrap();
        let again = load_game(&dir).err().unwrap();
        let left = names(&dir);
        let first = std::fs::read_to_string(dir.join("game.json.unreadable"));
        std::fs::remove_dir_all(&dir).unwrap();
        for err in [&game, &stats, &played, &again] {
            assert!(err.message.contains("moved to") && !err.in_place, "{err:?}");
        }
        assert!(played.message.contains("isn't a grid"), "{played:?}");
        assert_eq!(first.unwrap(), "{", "the first one kept");
        assert_eq!(
            left,
            [
                "game.json.unreadable",
                "game.json.unreadable-2",
                "played.json.unreadable",
                "stats.json.unreadable"
            ]
        );
    }

    /// A file that can't be read or moved aside says it is still in place.
    #[cfg(unix)]
    #[test]
    fn a_file_that_cant_be_moved_aside_says_so() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("in-place");
        std::fs::write(dir.join("stats.json"), "{").unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).unwrap();
        let err = load_stats(&dir).unwrap_err();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(err.in_place, "{err:?}");
    }

    /// Stats and puzzles played are added to what the file holds, so
    /// another sumi's are kept.
    #[test]
    fn stats_and_puzzles_played_add_to_the_file() {
        let dir = scratch("merge");
        let hard = crate::fixtures::game().rated(Level::Hard);
        let easy = crate::fixtures::game().rated(Level::Easy);
        let ours = update_stats(&dir, |s| {
            s.record(&hard, std::time::Duration::from_secs(90))
        });
        // Another sumi, which read the file before ours was written.
        let mut theirs = Stats::default();
        theirs.record(&easy, std::time::Duration::from_secs(60));
        let both = update_stats(&dir, |s| {
            s.record(&easy, std::time::Duration::from_secs(60))
        });
        let a: Grid = crate::fixtures::CLUES.parse().unwrap();
        save_played(&dir, &[a].into()).unwrap();
        let played = save_played(&dir, &[Grid::EMPTY].into()).unwrap();
        let back = load_played(&dir);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(ours.unwrap().rows().len(), 1);
        assert_eq!(both.unwrap().rows().len(), 2);
        assert_eq!(played, [a, Grid::EMPTY].into());
        assert_eq!(back, Ok(played));
    }
}
