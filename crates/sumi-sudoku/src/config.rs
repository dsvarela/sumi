//! `config.toml` in the config directory. A missing file is the defaults.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// The id of the theme to start with: a file name in `themes/`, without
    /// the extension.
    pub theme: Option<String>,
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
