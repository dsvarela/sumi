//! `config.toml` in the config directory. A missing file is the defaults.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// The id of the theme to start with: a file name in `themes/`, without
    /// the extension.
    pub theme: Option<String>,
    /// Show the time played. Off by default: sumi doesn't race a clock.
    pub timer: bool,
    /// The rung a hint opens at, 1 to 5.
    pub hint_start: usize,
    pub marks_to_candidates: MarksToCandidates,
    pub key_bar: bool,
    pub highlights: Highlights,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            theme: None,
            timer: false,
            hint_start: 1,
            marks_to_candidates: MarksToCandidates::Fill,
            key_bar: true,
            highlights: Highlights::default(),
        }
    }
}

/// What `t` does with box marks.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MarksToCandidates {
    /// Fill full candidates and clear the box marks.
    #[default]
    Fill,
    /// Add each cell's box marks to its candidates.
    Merge,
}

/// The highlights that can be switched off, all on by default. The filter
/// isn't here: it shows only when asked for with `f`.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Highlights {
    /// Every placed copy of the digit under the cursor.
    pub same_digit: bool,
    /// Faint shading on the cursor's row, column and box.
    pub peer: bool,
    /// A digit whose nine copies are placed is dimmed.
    pub complete_digits: bool,
    /// Notes that a placed digit in their row, column or box rules out.
    pub note_clashes: bool,
}

impl Default for Highlights {
    fn default() -> Highlights {
        Highlights {
            same_digit: true,
            peer: true,
            complete_digits: true,
            note_clashes: true,
        }
    }
}

impl Config {
    pub fn parse(src: &str) -> Result<Config, String> {
        toml::from_str(src).map_err(|err| err.message().to_string())
    }

    /// Where sumi keeps its files, if the system has a home directory.
    pub fn dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "sumi").map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Writes one setting into `config.toml` in `dir`, at `key` (a table's
    /// name, then the key in it, for a key in a table). The rest of the file,
    /// comments included, stays as it was; a missing file or directory is
    /// created.
    pub fn save(dir: &Path, key: &[&str], value: toml_edit::Value) -> Result<(), String> {
        let path = dir.join("config.toml");
        let src = match std::fs::read_to_string(&path) {
            Ok(src) => src,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(err) => return Err(format!("{}: {err}", path.display())),
        };
        let mut doc: toml_edit::DocumentMut = src
            .parse()
            .map_err(|err| format!("{}: {err}", path.display()))?;
        let (last, tables) = key.split_last().expect("a key has a name");
        let mut table = doc.as_table_mut();
        for name in tables {
            table = table
                .entry(name)
                .or_insert(toml_edit::table())
                .as_table_mut()
                .ok_or_else(|| format!("{}: {name} isn't a table", path.display()))?;
        }
        // A value already there keeps the spaces and comment around it.
        match table.get_mut(last).and_then(toml_edit::Item::as_value_mut) {
            Some(old) => {
                let decor = old.decor().clone();
                *old = value;
                *old.decor_mut() = decor;
            }
            None => table[*last] = toml_edit::value(value),
        }
        std::fs::create_dir_all(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
        std::fs::write(&path, doc.to_string()).map_err(|err| format!("{}: {err}", path.display()))
    }

    /// Reads `config.toml` from `dir`. A file that can't be read or parsed
    /// gives the defaults and a warning.
    pub fn load(dir: Option<&PathBuf>) -> (Config, Option<String>) {
        let Some(dir) = dir else {
            return (Config::default(), None);
        };
        let path = dir.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(src) => match Config::parse(&src) {
                Ok(config) => (config, None),
                Err(err) => (
                    Config::default(),
                    Some(format!("{}: {err}", path.display())),
                ),
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (Config::default(), None),
            Err(err) => (
                Config::default(),
                Some(format!("{}: {err}", path.display())),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_is_off_and_every_highlight_on_unless_switched() {
        let config = Config::parse("").unwrap();
        assert!(!config.timer);
        assert_eq!(config.highlights, Highlights::default());
        assert!(Highlights::default().same_digit && Highlights::default().note_clashes);
        let config = Config::parse("timer = true\n[highlights]\npeer = false").unwrap();
        assert!(config.timer);
        assert!(!config.highlights.peer);
        assert!(config.highlights.same_digit);
    }

    #[test]
    fn the_rest_of_the_defaults() {
        let config = Config::parse("").unwrap();
        assert_eq!(config.hint_start, 1);
        assert_eq!(config.marks_to_candidates, MarksToCandidates::Fill);
        assert!(config.key_bar);
        let config = Config::parse("marks_to_candidates = \"merge\"").unwrap();
        assert_eq!(config.marks_to_candidates, MarksToCandidates::Merge);
    }

    /// Saving one setting keeps the rest of the file, comments included,
    /// and makes the file when there is none.
    #[test]
    fn saving_a_setting_keeps_the_rest_of_the_file() {
        let dir = std::env::temp_dir().join(format!("sumi-save-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Config::save(&dir, &["timer"], true.into()).unwrap();
        let path = dir.join("config.toml");
        assert!(
            Config::parse(&std::fs::read_to_string(&path).unwrap())
                .unwrap()
                .timer
        );

        std::fs::write(&path, "# mine\ntheme = \"nord\" # dark\n").unwrap();
        Config::save(&dir, &["highlights", "peer"], false.into()).unwrap();
        Config::save(&dir, &["hint_start"], 3i64.into()).unwrap();
        Config::save(&dir, &["theme"], "gruvbox".into()).unwrap();
        let src = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(
            src.starts_with("# mine\ntheme = \"gruvbox\" # dark\n"),
            "{src}"
        );
        let config = Config::parse(&src).unwrap();
        assert_eq!(config.theme.as_deref(), Some("gruvbox"));
        assert!(!config.highlights.peer);
        assert_eq!(config.hint_start, 3);
    }

    #[test]
    fn a_config_picks_a_theme() {
        let config = Config::parse("theme = \"nord\"").unwrap();
        assert_eq!(config.theme.as_deref(), Some("nord"));
        assert_eq!(Config::parse("").unwrap(), Config::default());
    }

    #[test]
    fn a_missing_file_is_the_defaults_and_a_broken_one_is_a_warning() {
        let dir = std::env::temp_dir().join(format!("sumi-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(Config::load(Some(&dir)), (Config::default(), None));
        std::fs::write(dir.join("config.toml"), "theme = ").unwrap();
        let (config, warning) = Config::load(Some(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(config, Config::default());
        assert!(warning.unwrap().contains("config.toml"));
    }
}
