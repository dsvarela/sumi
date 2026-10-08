//! `config.toml` in the config directory. A missing file is the defaults.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// The id of the theme to start with: a file name in `themes/`, without
    /// the extension.
    pub theme: Option<String>,
    /// Show the time played. Off by default: sumi doesn't race a clock.
    pub timer: bool,
    pub highlights: Highlights,
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
