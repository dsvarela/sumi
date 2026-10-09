//! The files sumi keeps between runs in its data directory, and the one
//! way it writes a file: whole, or not at all.

use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use sumi_core::grid::Grid;

use crate::game::Game;

/// Where sumi keeps the saved game, if the system has a home directory.
pub fn dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "sumi").map(|dirs| dirs.data_dir().to_path_buf())
}

/// The game in progress, the one autosave slot.
const GAME: &str = "game.json";

/// Saves `json`, a game, as the game in progress.
pub fn save_game(dir: &Path, json: &str) -> Result<(), String> {
    let path = dir.join(GAME);
    std::fs::create_dir_all(dir)
        .and_then(|()| write_whole(&path, json))
        .map_err(|err| format!("{}: {err}", path.display()))
}

/// The game in progress, if one is saved. A saved game that can't be read
/// is moved aside to `game.json.unreadable`, so the next save doesn't
/// destroy it, and the error says so.
pub fn load_game(dir: &Path) -> Result<Option<Game>, String> {
    let path = dir.join(GAME);
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(format!("{}: {err}", path.display())),
    };
    Game::from_json(&json).map(Some).map_err(|why| {
        let aside = dir.join(format!("{GAME}.unreadable"));
        match std::fs::rename(&path, &aside) {
            Ok(()) => format!("{}: {why}; moved to {}", path.display(), aside.display()),
            Err(err) => format!("{}: {why}; couldn't move it aside: {err}", path.display()),
        }
    })
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
    let path = dir.join(PLAYED);
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(HashSet::new()),
        Err(err) => return Err(format!("{}: {err}", path.display())),
    };
    let named = |why: String| format!("{}: {why}", path.display());
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
    let path = dir.join(PLAYED);
    std::fs::create_dir_all(dir)
        .and_then(|()| write_whole(&path, &json))
        .map_err(|err| format!("{}: {err}", path.display()))
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
}
