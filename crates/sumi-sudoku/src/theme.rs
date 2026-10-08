//! Colour roles. Every colour on screen comes from one of these, so a theme
//! changes everything by changing this struct. Themes are TOML files: the
//! built-in ones in `themes/` and the player's own in the config directory.

use std::collections::BTreeMap;
use std::path::Path;

use ratatui::style::Color;
use serde::Deserialize;

/// Declares `Theme` with one `Color` field per role, and the role list the
/// loader reads names from, so a role is added in one place.
macro_rules! roles {
    ($($(#[$doc:meta])* $role:ident),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub struct Theme {
            pub name: String,
            $($(#[$doc])* pub $role: Color,)*
            /// No colour is set at all (NO_COLOR). What colour showed is then
            /// shown by attributes: a hint's cells, the filter's and an
            /// animation's strong light are drawn reversed.
            pub monochrome: bool,
        }

        impl Theme {
            #[cfg(test)]
            fn role(&self, role: &str) -> Option<Color> {
                match role {
                    $(stringify!($role) => Some(self.$role),)*
                    _ => None,
                }
            }

            fn role_mut(&mut self, role: &str) -> Option<&mut Color> {
                match role {
                    $(stringify!($role) => Some(&mut self.$role),)*
                    _ => None,
                }
            }

            const ROLES: &[&str] = &[$(stringify!($role)),*];

            /// Every role set to `colour`.
            fn all(name: &str, colour: Color, monochrome: bool) -> Theme {
                Theme {
                    name: name.to_string(),
                    $($role: colour,)*
                    monochrome,
                }
            }
        }
    };
}

roles! {
    background,
    /// Text that isn't on the board.
    text,
    dimmed,
    /// Titles, keys in the key bar, and the marks at the cursor's corners.
    accent,
    frame,
    /// Lines between cells inside a box.
    line,
    /// Lines between boxes and around the board.
    box_line,
    clue,
    digit,
    candidate,
    box_mark,
    /// Background of the selected cell.
    cursor,
    /// Background of cells holding the digit under the cursor, and an
    /// animation's strong light.
    same_digit,
    /// Background of the cursor's row, column and box.
    peer,
    /// Background of cells the candidate filter lights up.
    filter,
    conflict,
    /// Digits `c` found different from the solution.
    wrong_digit,
    /// Background of the rows, columns and boxes a hint points at, of the
    /// second group of a colouring, and of an animation's faint light.
    hint_area,
    /// Background of the cells that make a hint's pattern.
    hint_cell,
    /// Candidates a hint removes.
    hint_removal,
    /// The digit a hint places.
    hint_placement,
}

/// What a theme file holds. With `extends`, the colours it lists replace the
/// base theme's and the rest carry over; without it, every role is needed.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    name: String,
    extends: Option<String>,
    #[serde(default)]
    colors: BTreeMap<String, String>,
}

/// The built-in themes by id (the file name), the default first.
const BUILTIN: &[(&str, &str)] = &[
    (
        "tokyo-night-amoled",
        include_str!("../themes/tokyo-night-amoled.toml"),
    ),
    ("tokyo-night", include_str!("../themes/tokyo-night.toml")),
    (
        "catppuccin-mocha",
        include_str!("../themes/catppuccin-mocha.toml"),
    ),
    ("nord", include_str!("../themes/nord.toml")),
    ("gruvbox", include_str!("../themes/gruvbox.toml")),
    ("paper", include_str!("../themes/paper.toml")),
    ("okabe-ito", include_str!("../themes/okabe-ito.toml")),
];

pub const DEFAULT: &str = "tokyo-night-amoled";

impl Theme {
    /// Reads one theme file. `known` is every theme loaded before it, which
    /// `extends` may name by id.
    pub fn parse(src: &str, known: &[(String, Theme)]) -> Result<Theme, String> {
        let file: File = toml::from_str(src).map_err(|err| err.message().to_string())?;
        let mut theme = match &file.extends {
            Some(base) => known
                .iter()
                .find(|(id, _)| id == base)
                .map(|(_, theme)| theme.clone())
                .ok_or_else(|| format!("extends \"{base}\", which isn't a theme"))?,
            None => Theme::all(&file.name, Color::Reset, false),
        };
        theme.name = file.name;
        for (role, value) in &file.colors {
            let slot = theme
                .role_mut(role)
                .ok_or_else(|| format!("\"{role}\" isn't a colour role"))?;
            *slot = hex(value).ok_or_else(|| format!("{role}: \"{value}\" isn't #rrggbb"))?;
        }
        if file.extends.is_none()
            && let Some(role) = Theme::ROLES
                .iter()
                .find(|role| !file.colors.contains_key(**role))
        {
            return Err(format!("{role} is missing"));
        }
        Ok(theme)
    }

    /// The built-in themes, then the `*.toml` files in `user_dir`, each under
    /// the id of its file name; a user file with a built-in's id replaces
    /// it. A file that can't be read is left out and named in the warnings.
    pub fn load_all(user_dir: Option<&Path>) -> (Vec<(String, Theme)>, Vec<String>) {
        let mut themes: Vec<(String, Theme)> = Vec::new();
        let mut warnings = Vec::new();
        for (id, src) in BUILTIN {
            add(&mut themes, &mut warnings, id, Ok((*src).to_string()));
        }
        let mut pending: Vec<(String, Result<String, String>)> = Vec::new();
        if let Some(dir) = user_dir {
            match std::fs::read_dir(dir) {
                Ok(entries) => {
                    let mut files: Vec<_> = entries
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
                        .collect();
                    files.sort();
                    for path in files {
                        let id = path.file_stem().unwrap_or_default().to_string_lossy();
                        let src = std::fs::read_to_string(&path).map_err(|err| err.to_string());
                        pending.push((id.to_string(), src));
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => warnings.push(format!("{}: {err}", dir.display())),
            }
        }
        // A theme may extend one whose file sorts after it: each round loads
        // the themes whose base is loaded, until a round loads none. What is
        // left extends a theme that never loads, and says so.
        loop {
            let waiting = |src: &Result<String, String>, themes: &[(String, Theme)]| {
                let base = src
                    .as_ref()
                    .ok()
                    .and_then(|src| toml::from_str::<File>(src).ok())
                    .and_then(|file| file.extends);
                base.is_some_and(|base| !themes.iter().any(|(id, _)| *id == base))
            };
            let before = pending.len();
            let mut still = Vec::new();
            for (id, src) in pending {
                if waiting(&src, &themes) {
                    still.push((id, src));
                } else {
                    add(&mut themes, &mut warnings, &id, src);
                }
            }
            pending = still;
            if pending.len() == before {
                break;
            }
        }
        for (id, src) in pending {
            add(&mut themes, &mut warnings, &id, src);
        }
        (themes, warnings)
    }

    /// The terminal's own colours and none of ours.
    pub fn monochrome() -> Theme {
        Theme::all("No colour", Color::Reset, true)
    }

    /// For terminals without 24-bit colour: the 16 named colours.
    pub fn sixteen_colour() -> Theme {
        let mut theme = Theme::all("16 colours", Color::Reset, false);
        let roles = [
            ("background", Color::Black),
            ("text", Color::White),
            ("dimmed", Color::Gray),
            ("accent", Color::Cyan),
            ("frame", Color::Gray),
            ("line", Color::DarkGray),
            ("box_line", Color::Gray),
            ("clue", Color::White),
            ("digit", Color::Cyan),
            ("candidate", Color::Gray),
            ("box_mark", Color::Yellow),
            ("cursor", Color::DarkGray),
            ("same_digit", Color::Blue),
            ("peer", Color::Black),
            ("filter", Color::Magenta),
            ("conflict", Color::LightRed),
            ("wrong_digit", Color::LightRed),
            ("hint_area", Color::Blue),
            ("hint_cell", Color::Green),
            ("hint_removal", Color::LightRed),
            ("hint_placement", Color::LightGreen),
        ];
        for (role, colour) in roles {
            if let Some(slot) = theme.role_mut(role) {
                *slot = colour;
            }
        }
        theme
    }

    /// The theme to start with: the player's choice if there is one that
    /// loaded, else the default. NO_COLOR (`no_color`) and a terminal
    /// without 24-bit colour (`!truecolor`) override a theme made of colours.
    pub fn choose(
        themes: &[(String, Theme)],
        wanted: Option<&str>,
        no_color: bool,
        truecolor: bool,
    ) -> Theme {
        if no_color {
            return Theme::monochrome();
        }
        if !truecolor {
            return Theme::sixteen_colour();
        }
        let find = |id: &str| themes.iter().find(|(known, _)| known == id);
        wanted
            .and_then(find)
            .or_else(|| find(DEFAULT))
            .map_or_else(Theme::monochrome, |(_, theme)| theme.clone())
    }
}

/// Reads a theme file as `id` into `themes`, replacing a theme of the same
/// id, or says in `warnings` why it can't.
fn add(
    themes: &mut Vec<(String, Theme)>,
    warnings: &mut Vec<String>,
    id: &str,
    source: Result<String, String>,
) {
    match source.and_then(|src| Theme::parse(&src, themes)) {
        Ok(theme) => match themes.iter_mut().find(|(known, _)| known == id) {
            Some(slot) => slot.1 = theme,
            None => themes.push((id.to_string(), theme)),
        },
        Err(err) => warnings.push(format!("theme {id}: {err}")),
    }
}

fn hex(value: &str) -> Option<Color> {
    let digits = value.strip_prefix('#')?;
    if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    Some(Color::Rgb(channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
impl Theme {
    /// The default theme, for tests that don't care which one.
    pub fn tokyo_night_amoled() -> Theme {
        Theme::load_all(None).0.remove(0).1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::{self, Cvd, SURFACES, TEXT};

    fn rgb(c: Color) -> colour::Rgb {
        let Color::Rgb(r, g, b) = c else {
            panic!("{c:?} is not an RGB colour");
        };
        [r, g, b]
    }

    fn contrast(a: Color, b: Color) -> f64 {
        colour::contrast(rgb(a), rgb(b))
    }

    fn too_close(theme: &Theme, cvd: Option<Cvd>) -> Vec<String> {
        colour::too_close(|role| rgb(theme.role(role).unwrap()), cvd)
    }

    /// Every built-in theme keeps its highlights and coloured text apart as
    /// a person with full colour vision sees them.
    #[test]
    fn every_theme_keeps_its_highlights_apart() {
        for (id, theme) in Theme::load_all(None).0 {
            assert_eq!(too_close(&theme, None), Vec::<String>::new(), "{id}");
        }
    }

    /// The theme named for colour blindness keeps the same pairs apart under
    /// simulated protanopia, deuteranopia and tritanopia.
    #[test]
    fn the_colour_blind_theme_keeps_them_apart_without_full_colour_vision() {
        let (themes, _) = Theme::load_all(None);
        let theme = &themes.iter().find(|(id, _)| id == "okabe-ito").unwrap().1;
        assert_eq!(theme.name, "Colour-blind safe");
        for cvd in Cvd::ALL {
            assert_eq!(too_close(theme, Some(cvd)), Vec::<String>::new(), "{cvd:?}");
        }
    }

    /// Box lines stand out from the lines between cells.
    #[test]
    fn box_lines_are_stronger_than_cell_lines() {
        for (id, t) in Theme::load_all(None).0 {
            let (line, box_line) = (
                contrast(t.line, t.background),
                contrast(t.box_line, t.background),
            );
            assert!(
                box_line > line + colour::BOX_LINE_LEAD,
                "{id}: {box_line:.2} against {line:.2}"
            );
        }
    }

    /// Text needs 4.5:1 (WCAG 1.4.3) on every background it can be drawn on,
    /// and any board text can sit on any cell highlight. Lines, the frame
    /// and the cursor's corner marks need 3:1 (WCAG 1.4.11). The highlights
    /// stay below 3:1 against the background, since text on them must stay
    /// readable; the cursor is shown by its own marks as well as its colour.
    #[test]
    fn every_colour_pair_on_screen_is_readable() {
        let (themes, warnings) = Theme::load_all(None);
        assert_eq!(warnings, Vec::<String>::new());
        assert_eq!(themes.len(), BUILTIN.len());
        for (id, t) in &themes {
            let role = |name| t.role(name).unwrap();
            for surface in SURFACES {
                for text in TEXT {
                    let ratio = contrast(role(text), role(surface));
                    assert!(
                        ratio >= colour::TEXT_FLOOR,
                        "{id}: {text} on {surface}: {ratio:.2}"
                    );
                }
            }
            let graphics = [
                ("line", t.line, t.background),
                ("box_line", t.box_line, t.background),
                ("frame", t.frame, t.background),
                ("cursor marks", t.accent, t.cursor),
            ];
            for (name, fg, bg) in graphics {
                let ratio = contrast(fg, bg);
                assert!(ratio >= colour::GRAPHIC_FLOOR, "{id}: {name}: {ratio:.2}");
            }
            for surface in &SURFACES[1..] {
                let ratio = contrast(role(surface), t.background);
                assert!(
                    ratio < colour::HIGHLIGHT_CEILING,
                    "{id}: {surface} against the background: {ratio:.2}"
                );
            }
        }
    }

    fn theme(src: &str) -> Result<Theme, String> {
        Theme::parse(src, &Theme::load_all(None).0)
    }

    #[test]
    fn a_theme_extends_another_and_changes_a_few_colours() {
        let nord =
            theme("name = \"Mine\"\nextends = \"nord\"\n[colors]\naccent = \"#ff0000\"").unwrap();
        let (themes, _) = Theme::load_all(None);
        let base = &themes.iter().find(|(id, _)| id == "nord").unwrap().1;
        assert_eq!(nord.name, "Mine");
        assert_eq!(nord.accent, Color::Rgb(255, 0, 0));
        assert_eq!(nord.background, base.background);
        assert_ne!(base.accent, nord.accent);
    }

    #[test]
    fn a_theme_without_a_base_needs_every_role() {
        let err = theme("name = \"Bare\"\n[colors]\nbackground = \"#000000\"").unwrap_err();
        assert_eq!(err, "text is missing");
    }

    #[test]
    fn bad_themes_say_what_is_wrong() {
        let bad = |colors: &str, extends: &str| {
            theme(&format!("name = \"x\"\n{extends}[colors]\n{colors}")).unwrap_err()
        };
        assert_eq!(
            bad("text = \"red\"", "extends = \"nord\"\n"),
            "text: \"red\" isn't #rrggbb"
        );
        assert_eq!(
            bad("text = \"#12345\"", "extends = \"nord\"\n"),
            "text: \"#12345\" isn't #rrggbb"
        );
        assert_eq!(
            bad("glow = \"#000000\"", "extends = \"nord\"\n"),
            "\"glow\" isn't a colour role"
        );
        assert_eq!(
            bad("", "extends = \"nowhere\"\n"),
            "extends \"nowhere\", which isn't a theme"
        );
    }

    #[test]
    fn user_themes_load_beside_the_built_in_ones_and_bad_files_are_named() {
        let dir = std::env::temp_dir().join(format!("sumi-themes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("mine.toml"),
            "name = \"Mine\"\nextends = \"gruvbox\"\n[colors]\naccent = \"#010203\"",
        )
        .unwrap();
        std::fs::write(dir.join("broken.toml"), "name = ").unwrap();
        let nord = "name = \"My Nord\"\nextends = \"nord\"\n";
        std::fs::write(dir.join("nord.toml"), nord).unwrap();
        std::fs::write(dir.join("notes.txt"), "not a theme").unwrap();
        let (themes, warnings) = Theme::load_all(Some(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
        let ids: Vec<_> = themes.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids.len(), BUILTIN.len() + 1);
        assert_eq!(ids.last(), Some(&"mine"));
        let nord = themes.iter().find(|(id, _)| id == "nord").unwrap();
        assert_eq!(nord.1.name, "My Nord");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].starts_with("theme broken: "), "{warnings:?}");
    }

    /// A theme may extend one whose file sorts after its own; a misspelt
    /// table is an error, not a theme that silently ignores it.
    #[test]
    fn user_themes_extend_in_any_order_and_unknown_tables_are_named() {
        let dir = std::env::temp_dir().join(format!("sumi-order-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, src: &str| std::fs::write(dir.join(name), src).unwrap();
        write("a-child.toml", "name = \"Child\"\nextends = \"b-base\"\n");
        write(
            "b-base.toml",
            "name = \"Base\"\nextends = \"nord\"\n[colors]\naccent = \"#010203\"\n",
        );
        write(
            "c-typo.toml",
            "name = \"Typo\"\nextends = \"nord\"\n[colours]\naccent = \"#010203\"\n",
        );
        write("d-loop.toml", "name = \"Loop\"\nextends = \"e-loop\"\n");
        write("e-loop.toml", "name = \"Loop\"\nextends = \"d-loop\"\n");
        let (themes, warnings) = Theme::load_all(Some(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
        let child = &themes.iter().find(|(id, _)| id == "a-child").unwrap().1;
        assert_eq!(child.accent, Color::Rgb(1, 2, 3));
        assert!(
            warnings
                .iter()
                .any(|w| w.starts_with("theme c-typo: ") && w.contains("colours")),
            "{warnings:?}"
        );
        assert!(
            warnings.iter().any(|w| w.starts_with("theme d-loop: ")),
            "{warnings:?}"
        );
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }

    #[test]
    fn a_colour_is_six_hex_digits() {
        assert_eq!(hex("#0a0B0c"), Some(Color::Rgb(10, 11, 12)));
        for bad in ["#+1+2+3", "0a0b0c", "#0a0b0", "#0a0b0g", "#0a0b0c0"] {
            assert_eq!(hex(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_environment_decides_before_the_player_does() {
        let (themes, _) = Theme::load_all(None);
        let pick =
            |wanted, no_color, truecolor| Theme::choose(&themes, wanted, no_color, truecolor);
        assert_eq!(pick(Some("nord"), false, true).name, "Nord");
        assert_eq!(
            pick(Some("nowhere"), false, true).name,
            "Tokyo Night AMOLED"
        );
        assert_eq!(pick(None, false, true).name, "Tokyo Night AMOLED");
        assert!(pick(Some("nord"), true, true).monochrome);
        assert!(pick(Some("nord"), true, false).monochrome);
        assert_eq!(pick(Some("nord"), false, false).name, "16 colours");
    }
}
