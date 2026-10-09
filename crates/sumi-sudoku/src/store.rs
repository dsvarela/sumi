//! How sumi writes its files: whole, or not at all.

use std::io::Write;
use std::path::{Path, PathBuf};

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
}
