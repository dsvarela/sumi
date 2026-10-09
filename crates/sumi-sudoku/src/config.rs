//! `config.toml` in the config directory. A missing file is the defaults.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::store;

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
    /// The digits under the board, each with how many are left to place.
    pub pad: bool,
    /// Clicks and taps act on the board, the pad and the menus.
    pub mouse: bool,
    /// A completed row, column or box pulses, and a wave crosses the board
    /// at a win.
    pub animations: bool,
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
            pad: true,
            mouse: true,
            animations: true,
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

/// Puts `value` into `slot` through `make`, if it has the type `make`
/// takes, or says what's wrong with it.
fn read<T: serde::de::DeserializeOwned, U>(
    value: toml::Value,
    slot: &mut U,
    make: impl FnOnce(T) -> U,
) -> Result<(), String> {
    let found = value.type_str();
    let value: T = value
        .try_into()
        .map_err(|_| format!("a {found} isn't a value it takes"))?;
    *slot = make(value);
    Ok(())
}

impl Config {
    /// Reads `config.toml`'s text, key by key. A key with a value of the
    /// wrong type keeps its default, and one sumi doesn't know is skipped;
    /// both are named in the warnings, so one slip doesn't undo the rest of
    /// the file. Text that isn't TOML at all is an error.
    pub fn parse(src: &str) -> Result<(Config, Vec<String>), String> {
        let table: toml::Table = src
            .parse()
            .map_err(|err: toml::de::Error| err.message().to_string())?;
        let mut config = Config::default();
        let mut warnings = Vec::new();
        for (key, value) in table {
            let checked = match key.as_str() {
                "theme" => read(value, &mut config.theme, Some),
                "timer" => read(value, &mut config.timer, |v| v),
                // A rung past the ladder's ends is taken as the nearest end,
                // so the settings page shows what hints do.
                "hint_start" => {
                    let mut rung = 1i64;
                    read(value, &mut rung, |v: i64| v).and_then(|()| {
                        config.hint_start = rung.clamp(1, 5) as usize;
                        match rung {
                            1..=5 => Ok(()),
                            _ => Err(format!(
                                "{rung} isn't a rung, 1 to 5; using {}",
                                config.hint_start
                            )),
                        }
                    })
                }
                "marks_to_candidates" => read(value, &mut config.marks_to_candidates, |v| v),
                "key_bar" => read(value, &mut config.key_bar, |v| v),
                "pad" => read(value, &mut config.pad, |v| v),
                "mouse" => read(value, &mut config.mouse, |v| v),
                "animations" => read(value, &mut config.animations, |v| v),
                "highlights" => match value {
                    toml::Value::Table(table) => {
                        let h = &mut config.highlights;
                        for (name, value) in table {
                            let slot = match name.as_str() {
                                "same_digit" => &mut h.same_digit,
                                "peer" => &mut h.peer,
                                "complete_digits" => &mut h.complete_digits,
                                "note_clashes" => &mut h.note_clashes,
                                _ => {
                                    warnings.push(format!("highlights.{name} isn't a setting"));
                                    continue;
                                }
                            };
                            if let Err(err) = read(value, slot, |v| v) {
                                warnings.push(format!("highlights.{name}: {err}"));
                            }
                        }
                        Ok(())
                    }
                    _ => Err("should be a table".to_string()),
                },
                _ => {
                    warnings.push(format!("{key} isn't a setting"));
                    continue;
                }
            };
            if let Err(err) = checked {
                warnings.push(format!("{key}: {err}"));
            }
        }
        Ok((config, warnings))
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
        let mut table: &mut dyn toml_edit::TableLike = doc.as_table_mut();
        for name in tables {
            table = table
                .entry(name)
                .or_insert(toml_edit::table())
                .as_table_like_mut()
                .ok_or_else(|| format!("{}: {name} isn't a table", path.display()))?;
        }
        // A value already there keeps the spaces and comment around it.
        match table.get_mut(last).and_then(toml_edit::Item::as_value_mut) {
            Some(old) => {
                let decor = old.decor().clone();
                *old = value;
                *old.decor_mut() = decor;
            }
            None => {
                table.insert(last, toml_edit::value(value));
            }
        }
        std::fs::create_dir_all(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
        store::write_whole(&path, &doc.to_string())
            .map_err(|err| format!("{}: {err}", path.display()))
    }

    /// Reads `config.toml` from `dir`, with a line for each thing in it
    /// that couldn't be used. A file that can't be read or isn't TOML gives
    /// the defaults.
    pub fn load(dir: Option<&PathBuf>) -> (Config, Vec<String>) {
        let Some(dir) = dir else {
            return (Config::default(), Vec::new());
        };
        let path = dir.join("config.toml");
        let named = |line: String| format!("{}: {line}", path.display());
        match std::fs::read_to_string(&path) {
            Ok(src) => match Config::parse(&src) {
                Ok((config, warnings)) => (config, warnings.into_iter().map(named).collect()),
                Err(err) => (Config::default(), vec![named(err)]),
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                (Config::default(), Vec::new())
            }
            Err(err) => (Config::default(), vec![named(err.to_string())]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_is_off_and_every_highlight_on_unless_switched() {
        let config = Config::parse("").unwrap().0;
        assert!(!config.timer);
        assert_eq!(config.highlights, Highlights::default());
        assert!(Highlights::default().same_digit && Highlights::default().note_clashes);
        let config = Config::parse("timer = true\n[highlights]\npeer = false")
            .unwrap()
            .0;
        assert!(config.timer);
        assert!(!config.highlights.peer);
        assert!(config.highlights.same_digit);
    }

    #[test]
    fn the_rest_of_the_defaults() {
        let config = Config::parse("").unwrap().0;
        assert_eq!(config.hint_start, 1);
        assert_eq!(config.marks_to_candidates, MarksToCandidates::Fill);
        assert!(config.key_bar);
        let config = Config::parse("marks_to_candidates = \"merge\"").unwrap().0;
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
                .0
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
        let config = Config::parse(&src).unwrap().0;
        assert_eq!(config.theme.as_deref(), Some("gruvbox"));
        assert!(!config.highlights.peer);
        assert_eq!(config.hint_start, 3);
    }

    /// A wrong value costs only its own key, and every key that can't be
    /// used is named, so a slip doesn't undo the rest of the file.
    #[test]
    fn one_bad_value_keeps_the_rest_and_is_named() {
        let src = "hint_start = \"two\"\ntheme = \"nord\"\ntimmer = true\n\
                   [highlights]\npeer = \"no\"\nglow = true\nsame_digit = false\n";
        let (config, warnings) = Config::parse(src).unwrap();
        assert_eq!(config.theme.as_deref(), Some("nord"));
        assert_eq!(config.hint_start, 1);
        assert!(config.highlights.peer);
        assert!(!config.highlights.same_digit);
        let named = |key: &str| warnings.iter().any(|w| w.starts_with(key));
        for key in [
            "hint_start:",
            "timmer ",
            "highlights.peer:",
            "highlights.glow ",
        ] {
            assert!(named(key), "{key} in {warnings:?}");
        }
        assert_eq!(warnings.len(), 4, "{warnings:?}");
        let (_, warnings) = Config::parse("highlights = 3").unwrap();
        assert_eq!(warnings, ["highlights: should be a table"]);
        assert!(Config::parse("timer = ").is_err(), "not TOML at all");
    }

    /// A rung past either end is taken as that end, so the settings page
    /// shows what hints do.
    #[test]
    fn a_hint_start_out_of_range_is_taken_as_the_nearest_end() {
        let (config, warnings) = Config::parse("hint_start = 9").unwrap();
        assert_eq!((config.hint_start, warnings.len()), (5, 1));
        let (config, _) = Config::parse("hint_start = 0").unwrap();
        assert_eq!(config.hint_start, 1);
        let (config, warnings) = Config::parse("hint_start = 3").unwrap();
        assert_eq!((config.hint_start, warnings.len()), (3, 0));
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sumi-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_setting_saves_into_an_inline_table() {
        let dir = scratch("inline");
        let path = dir.join("config.toml");
        std::fs::write(&path, "highlights = { peer = false }\n").unwrap();
        Config::save(&dir, &["highlights", "same_digit"], false.into()).unwrap();
        let src = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        let (config, warnings) = Config::parse(&src).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(
            !config.highlights.peer && !config.highlights.same_digit,
            "{src}"
        );
    }

    /// A config.toml that is a symlink, as dotfiles often are, stays one:
    /// the file it points to is the one written, and nothing is left beside
    /// it.
    #[cfg(unix)]
    #[test]
    fn saving_follows_a_symlink_and_leaves_nothing_behind() {
        let dir = scratch("link");
        let real = dir.join("real.toml");
        std::fs::write(&real, "# kept\n").unwrap();
        let config_dir = dir.join("sumi");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::os::unix::fs::symlink(&real, config_dir.join("config.toml")).unwrap();
        Config::save(&config_dir, &["timer"], true.into()).unwrap();
        let link = std::fs::symlink_metadata(config_dir.join("config.toml")).unwrap();
        let written = std::fs::read_to_string(&real).unwrap();
        let mut left: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        left.sort();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(link.file_type().is_symlink());
        assert!(written.contains("# kept"), "{written}");
        assert!(Config::parse(&written).unwrap().0.timer);
        assert_eq!(left, ["real.toml", "sumi"]);
    }

    #[test]
    fn the_pad_can_be_hidden() {
        assert!(Config::default().pad);
        assert!(!Config::parse("pad = false").unwrap().0.pad);
    }

    #[test]
    fn a_config_picks_a_theme() {
        let config = Config::parse("theme = \"nord\"").unwrap().0;
        assert_eq!(config.theme.as_deref(), Some("nord"));
        assert_eq!(Config::parse("").unwrap().0, Config::default());
    }

    #[test]
    fn a_missing_file_is_the_defaults_and_a_broken_one_is_a_warning() {
        let dir = std::env::temp_dir().join(format!("sumi-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(Config::load(Some(&dir)), (Config::default(), Vec::new()));
        std::fs::write(dir.join("config.toml"), "theme = ").unwrap();
        let (config, warning) = Config::load(Some(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(config, Config::default());
        assert!(warning[0].contains("config.toml"), "{warning:?}");
    }
}
