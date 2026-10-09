//! The settings page: the options in their groups, what each shows, and
//! how a key changes it. Every option is a line of `config.toml`.

use crate::config::{Config, MarksToCandidates};
use crate::game::Direction;
use crate::input::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Timer,
    Stats,
    Mouse,
    HintStart,
    MarksToCandidates,
    SameDigit,
    Peer,
    CompleteDigits,
    NoteClashes,
    Theme,
    KeyBar,
    Pad,
    Animations,
}

/// The groups in order, each with its settings. Keys has none yet:
/// remapping comes after v1, and the page says where the keys are listed.
pub const GROUPS: [(&str, &[Setting]); 6] = [
    (
        "Gameplay",
        &[Setting::Timer, Setting::Stats, Setting::Mouse],
    ),
    ("Hints", &[Setting::HintStart]),
    ("Candidates and marks", &[Setting::MarksToCandidates]),
    (
        "Highlights",
        &[
            Setting::SameDigit,
            Setting::Peer,
            Setting::CompleteDigits,
            Setting::NoteClashes,
        ],
    ),
    (
        "Look",
        &[
            Setting::Theme,
            Setting::KeyBar,
            Setting::Pad,
            Setting::Animations,
        ],
    ),
    ("Keys", &[]),
];

impl Setting {
    /// Every setting, in the order the page lists them.
    pub fn all() -> Vec<Setting> {
        GROUPS.iter().flat_map(|(_, s)| s.iter().copied()).collect()
    }

    pub fn label(self) -> &'static str {
        match self {
            Setting::Timer => "Timer",
            Setting::Stats => "Stats",
            Setting::Mouse => "Mouse and tap",
            Setting::HintStart => "A hint opens at rung",
            Setting::MarksToCandidates => "t on box marks",
            Setting::SameDigit => "Same digit",
            Setting::Peer => "Row, column and box",
            Setting::CompleteDigits => "Dim finished digits",
            Setting::NoteClashes => "Candidates that clash",
            Setting::Theme => "Theme",
            Setting::KeyBar => "Key bar",
            Setting::Pad => "Digit pad",
            Setting::Animations => "Animations",
        }
    }

    /// The setting's place in `config.toml`.
    pub fn key(self) -> &'static [&'static str] {
        match self {
            Setting::Timer => &["timer"],
            Setting::Stats => &["stats"],
            Setting::Mouse => &["mouse"],
            Setting::HintStart => &["hint_start"],
            Setting::MarksToCandidates => &["marks_to_candidates"],
            Setting::SameDigit => &["highlights", "same_digit"],
            Setting::Peer => &["highlights", "peer"],
            Setting::CompleteDigits => &["highlights", "complete_digits"],
            Setting::NoteClashes => &["highlights", "note_clashes"],
            Setting::Theme => &["theme"],
            Setting::KeyBar => &["key_bar"],
            Setting::Pad => &["pad"],
            Setting::Animations => &["animations"],
        }
    }

    /// What the page shows as the setting's value. `theme` is the name of
    /// the theme in use.
    pub fn shown(self, config: &Config, theme: &str) -> String {
        let on = |b: bool| if b { "on" } else { "off" }.to_string();
        match self {
            Setting::Timer => on(config.timer),
            Setting::Stats => on(config.stats),
            Setting::Mouse => on(config.mouse),
            Setting::HintStart => config.hint_start.to_string(),
            Setting::MarksToCandidates => match config.marks_to_candidates {
                MarksToCandidates::Fill => "fill, drop marks".to_string(),
                MarksToCandidates::Merge => "add to candidates".to_string(),
            },
            Setting::SameDigit => on(config.highlights.same_digit),
            Setting::Peer => on(config.highlights.peer),
            Setting::CompleteDigits => on(config.highlights.complete_digits),
            Setting::NoteClashes => on(config.highlights.note_clashes),
            Setting::Theme => theme.to_string(),
            Setting::KeyBar => if config.key_bar { "shown" } else { "hidden" }.to_string(),
            Setting::Pad => if config.pad { "shown" } else { "hidden" }.to_string(),
            Setting::Animations => on(config.animations),
        }
    }

    /// Moves the setting to its next value, or its previous one when
    /// `forward` is false, wrapping. `themes` are the theme ids in order.
    pub fn change(self, config: &mut Config, themes: &[String], forward: bool) {
        let h = &mut config.highlights;
        match self {
            Setting::Timer => config.timer = !config.timer,
            Setting::Stats => config.stats = !config.stats,
            Setting::Mouse => config.mouse = !config.mouse,
            Setting::HintStart => {
                let next = if forward {
                    config.hint_start % 5 + 1
                } else {
                    (config.hint_start + 3) % 5 + 1
                };
                config.hint_start = next;
            }
            Setting::MarksToCandidates => {
                config.marks_to_candidates = match config.marks_to_candidates {
                    MarksToCandidates::Fill => MarksToCandidates::Merge,
                    MarksToCandidates::Merge => MarksToCandidates::Fill,
                }
            }
            Setting::SameDigit => h.same_digit = !h.same_digit,
            Setting::Peer => h.peer = !h.peer,
            Setting::CompleteDigits => h.complete_digits = !h.complete_digits,
            Setting::NoteClashes => h.note_clashes = !h.note_clashes,
            Setting::Theme => {
                // No theme, or one that didn't load, means the default is
                // the one showing, so the cycle starts from it.
                let position = |id: &str| themes.iter().position(|t| t == id);
                let at = config
                    .theme
                    .as_deref()
                    .and_then(position)
                    .or_else(|| position(crate::theme::DEFAULT))
                    .unwrap_or(0);
                let n = themes.len();
                let next = if forward {
                    (at + 1) % n
                } else {
                    (at + n - 1) % n
                };
                config.theme = Some(themes[next].clone());
            }
            Setting::KeyBar => config.key_bar = !config.key_bar,
            Setting::Pad => config.pad = !config.pad,
            Setting::Animations => config.animations = !config.animations,
        }
    }

    /// The value `config.toml` gets for the setting.
    pub fn value(self, config: &Config) -> toml_edit::Value {
        let h = &config.highlights;
        match self {
            Setting::Timer => config.timer.into(),
            Setting::Stats => config.stats.into(),
            Setting::Mouse => config.mouse.into(),
            Setting::HintStart => (config.hint_start as i64).into(),
            Setting::MarksToCandidates => match config.marks_to_candidates {
                MarksToCandidates::Fill => "fill".into(),
                MarksToCandidates::Merge => "merge".into(),
            },
            Setting::SameDigit => h.same_digit.into(),
            Setting::Peer => h.peer.into(),
            Setting::CompleteDigits => h.complete_digits.into(),
            Setting::NoteClashes => h.note_clashes.into(),
            Setting::Theme => config.theme.clone().unwrap_or_default().into(),
            Setting::KeyBar => config.key_bar.into(),
            Setting::Pad => config.pad.into(),
            Setting::Animations => config.animations.into(),
        }
    }
}

/// What a key on the settings page asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// The setting changed, forward or back.
    Change(Setting, bool),
    Back,
}

/// The settings page: which setting is selected, and whether Esc returns
/// to the pause menu or the main one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub selected: usize,
    pub from_pause: bool,
}

impl Page {
    pub fn new(from_pause: bool) -> Page {
        Page {
            selected: 0,
            from_pause,
        }
    }

    pub fn setting(&self) -> Setting {
        Setting::all()[self.selected]
    }

    /// Up and down move, wrapping; right and Enter change the setting
    /// forward, left back; Esc leaves.
    pub fn handle(&mut self, action: Action) -> Option<Choice> {
        let count = Setting::all().len();
        match action {
            Action::Step(Direction::Up) => self.selected = (self.selected + count - 1) % count,
            Action::Step(Direction::Down) => self.selected = (self.selected + 1) % count,
            Action::Step(Direction::Right) | Action::Confirm => {
                return Some(Choice::Change(self.setting(), true));
            }
            Action::Step(Direction::Left) => return Some(Choice::Change(self.setting(), false)),
            Action::Close => return Some(Choice::Back),
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each setting's value goes to the key that reads it back: changing a
    /// setting, saving its value under its key and parsing the result gives
    /// the changed config.
    #[test]
    fn every_setting_saves_to_the_key_it_is_read_from() {
        let themes = ["a".to_string(), "b".to_string()];
        for setting in Setting::all() {
            let mut config = Config::default();
            setting.change(&mut config, &themes, true);
            assert_ne!(config, Config::default(), "{setting:?} changed nothing");
            let mut doc = toml_edit::DocumentMut::new();
            let (last, tables) = setting.key().split_last().unwrap();
            let mut table = doc.as_table_mut();
            for name in tables {
                table = table
                    .entry(name)
                    .or_insert(toml_edit::table())
                    .as_table_mut()
                    .unwrap();
            }
            table[*last] = toml_edit::value(setting.value(&config));
            assert_eq!(
                Config::parse(&doc.to_string()).unwrap().0,
                config,
                "{setting:?}"
            );
        }
    }

    #[test]
    fn the_hint_start_cycles_one_to_five_both_ways() {
        let mut config = Config::default();
        let mut seen = Vec::new();
        for _ in 0..5 {
            Setting::HintStart.change(&mut config, &[], true);
            seen.push(config.hint_start);
        }
        assert_eq!(seen, [2, 3, 4, 5, 1]);
        Setting::HintStart.change(&mut config, &[], false);
        assert_eq!(config.hint_start, 5);
    }

    /// With no theme set, or one that didn't load, the default is showing,
    /// so the cycle starts from it wherever it sits in the list.
    #[test]
    fn the_theme_cycle_starts_from_the_default() {
        let themes = ["a", crate::theme::DEFAULT, "c"].map(String::from);
        for start in [None, Some("missing".to_string())] {
            let mut config = Config {
                theme: start,
                ..Config::default()
            };
            Setting::Theme.change(&mut config, &themes, true);
            assert_eq!(config.theme.as_deref(), Some("c"));
        }
    }

    #[test]
    fn the_theme_cycles_through_the_ids_both_ways() {
        let themes = ["a", "b", "c"].map(String::from);
        let mut config = Config::default();
        Setting::Theme.change(&mut config, &themes, true);
        assert_eq!(config.theme.as_deref(), Some("b"));
        Setting::Theme.change(&mut config, &themes, false);
        Setting::Theme.change(&mut config, &themes, false);
        assert_eq!(config.theme.as_deref(), Some("c"));
    }

    #[test]
    fn keys_on_the_page() {
        let mut page = Page::new(true);
        assert_eq!(page.handle(Action::Step(Direction::Up)), None);
        assert_eq!(page.setting(), Setting::Animations);
        assert_eq!(
            page.handle(Action::Confirm),
            Some(Choice::Change(Setting::Animations, true))
        );
        assert_eq!(
            page.handle(Action::Step(Direction::Left)),
            Some(Choice::Change(Setting::Animations, false))
        );
        assert_eq!(page.handle(Action::Close), Some(Choice::Back));
        assert_eq!(page.handle(Action::Digit(4)), None);
    }
}
