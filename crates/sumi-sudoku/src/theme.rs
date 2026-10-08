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
            /// shown by attributes: a hint's cells are drawn reversed.
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
    /// Background of cells holding the digit under the cursor.
    same_digit,
    /// Background of the cursor's row, column and box.
    peer,
    /// Background of cells the candidate filter lights up.
    filter,
    conflict,
    /// Digits `c` found different from the solution.
    wrong_digit,
    /// Background of the rows, columns and boxes a hint points at, and of the
    /// second group of a colouring.
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
        let mut add = |id: String, source: Result<String, String>| {
            let theme = source.and_then(|src| Theme::parse(&src, &themes));
            match theme {
                Ok(theme) => match themes.iter_mut().find(|(known, _)| *known == id) {
                    Some(slot) => slot.1 = theme,
                    None => themes.push((id, theme)),
                },
                Err(err) => warnings.push(format!("theme {id}: {err}")),
            }
        };
        for (id, src) in BUILTIN {
            add((*id).to_string(), Ok((*src).to_string()));
        }
        let mut files: Vec<_> = user_dir
            .and_then(|dir| std::fs::read_dir(dir).ok())
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        for path in files {
            let id = path.file_stem().unwrap_or_default().to_string_lossy();
            add(
                id.to_string(),
                std::fs::read_to_string(&path).map_err(|err| err.to_string()),
            );
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

fn hex(value: &str) -> Option<Color> {
    let digits = value.strip_prefix('#')?;
    if digits.len() != 6 || !digits.is_ascii() {
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

    /// WCAG 2.2 contrast ratio between two colours.
    fn contrast(a: Color, b: Color) -> f64 {
        fn luminance(c: Color) -> f64 {
            let Color::Rgb(r, g, b) = c else {
                panic!("{c:?} is not an RGB colour");
            };
            let channel = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
        }
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// A colour in OKLab, after an optional colour-blindness simulation
    /// (Machado, Oliveira and Fernandes 2009, severity 1, on linear RGB).
    fn seen(c: Color, simulate: Option<&[[f64; 3]; 3]>) -> [f64; 3] {
        let Color::Rgb(r, g, b) = c else {
            panic!("{c:?} is not an RGB colour");
        };
        let linear = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let mut rgb = [linear(r), linear(g), linear(b)];
        if let Some(m) = simulate {
            let mixed = |i: usize| (0..3).map(|j| m[i][j] * rgb[j]).sum::<f64>();
            rgb = [0, 1, 2].map(|i| mixed(i).clamp(0.0, 1.0));
        }
        let [r, g, b] = rgb;
        let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
        let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
        let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
        [
            0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
        ]
    }

    const PROTANOPIA: [[f64; 3]; 3] = [
        [0.152286, 1.052583, -0.204868],
        [0.114503, 0.786281, 0.099216],
        [-0.003882, -0.048116, 1.051998],
    ];
    const DEUTERANOPIA: [[f64; 3]; 3] = [
        [0.367322, 0.860646, -0.227968],
        [0.280085, 0.672501, 0.047413],
        [-0.011820, 0.042940, 0.968881],
    ];
    const TRITANOPIA: [[f64; 3]; 3] = [
        [1.255528, -0.076749, -0.178779],
        [-0.078411, 0.930809, 0.147602],
        [0.004733, 0.691367, 0.303900],
    ];

    /// Pairs that must look different, and the least OKLab distance between
    /// them. The cursor has its own shape, so it is held apart only from the
    /// background; peer shading is meant to be faint. The text pairs also
    /// differ by weight or a mark, so colour is a second cue for them.
    const APART: &[(&str, &str, f64)] = &[
        ("background", "cursor", 0.04),
        ("background", "hint_cell", 0.04),
        ("background", "same_digit", 0.03),
        ("background", "filter", 0.03),
        ("background", "hint_area", 0.025),
        ("background", "peer", 0.012),
        ("hint_cell", "hint_area", 0.03),
        ("hint_area", "same_digit", 0.02),
        ("hint_area", "filter", 0.02),
        ("same_digit", "filter", 0.02),
        ("digit", "clue", 0.06),
        ("conflict", "digit", 0.06),
        ("box_mark", "candidate", 0.06),
        ("hint_removal", "hint_placement", 0.06),
    ];

    fn too_close(theme: &Theme, simulate: Option<&[[f64; 3]; 3]>) -> Vec<String> {
        let at = |role| seen(theme.role(role).unwrap(), simulate);
        APART
            .iter()
            .filter_map(|&(a, b, floor)| {
                let (p, q) = (at(a), at(b));
                let distance = (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f64>().sqrt();
                (distance < floor).then(|| format!("{a}/{b} {distance:.3}"))
            })
            .collect()
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
        for simulate in [PROTANOPIA, DEUTERANOPIA, TRITANOPIA] {
            assert_eq!(too_close(theme, Some(&simulate)), Vec::<String>::new());
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
                box_line > line + 0.5,
                "{id}: {box_line:.2} against {line:.2}"
            );
        }
    }

    /// Text needs 4.5:1 (WCAG 1.4.3) on every background it can be drawn on,
    /// and any board text can sit on any cell highlight. Lines, the frame
    /// and the cursor's corner marks need 3:1 (WCAG 1.4.11). The highlights
    /// stay below 3:1 against black, since text on them must stay readable;
    /// the cursor is shown by its corner marks as well as its colour.
    #[test]
    fn every_colour_pair_on_screen_is_readable() {
        let (themes, warnings) = Theme::load_all(None);
        assert_eq!(warnings, Vec::<String>::new());
        assert_eq!(themes.len(), BUILTIN.len());
        for (id, theme) in &themes {
            check_readable(id, theme);
        }
    }

    fn check_readable(id: &str, t: &Theme) {
        let backgrounds = [
            ("background", t.background),
            ("cursor", t.cursor),
            ("same_digit", t.same_digit),
            ("peer", t.peer),
            ("filter", t.filter),
            ("hint_area", t.hint_area),
            ("hint_cell", t.hint_cell),
        ];
        let text = [
            ("text", t.text),
            ("dimmed", t.dimmed),
            ("accent", t.accent),
            ("clue", t.clue),
            ("digit", t.digit),
            ("candidate", t.candidate),
            ("box_mark", t.box_mark),
            ("conflict", t.conflict),
            ("wrong_digit", t.wrong_digit),
            ("hint_removal", t.hint_removal),
            ("hint_placement", t.hint_placement),
        ];
        for (bg_name, bg) in backgrounds {
            for (fg_name, fg) in text {
                let ratio = contrast(fg, bg);
                assert!(ratio >= 4.5, "{id}: {fg_name} on {bg_name}: {ratio:.2}");
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
            assert!(ratio >= 3.0, "{id}: {name}: {ratio:.2}");
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
