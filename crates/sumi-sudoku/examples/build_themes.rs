//! Builds the built-in themes in `themes/` from a few colours of each source
//! palette, and prints how each one meets the floors the theme tests check:
//!
//! `cargo run -p sumi-sudoku --example build_themes -- crates/sumi-sudoku/themes`
//!
//! Without a directory it only prints. The rules:
//!
//! - Text colours are lifted in OKLCh lightness, hue kept, until each reads
//!   at `LIFT`:1 on the background. That leaves room for the highlights,
//!   which must keep every text colour at 4.5:1 on them.
//! - Each highlight is the background moved toward its own source colour
//!   until its target contrast against the background, or as far as the
//!   text allows.
//! - Lines, the frame and box lines are the background moved toward the
//!   text until 3:1, 3.6:1 and 4.5:1, so each kind of line stands apart.

#[path = "../src/colour.rs"]
#[allow(dead_code)]
mod colour;

use std::collections::BTreeMap;
use std::path::Path;
use std::{env, fs};

use colour::{
    Cvd, GRAPHIC_FLOOR, MARGIN, Rgb, SURFACES, TEXT, TEXT_FLOOR, contrast, luminance, too_close,
};

const LIFT: f64 = 8.0;

/// Each highlight: the role it moves toward, and its target contrast
/// against the background. Highlights that share a source colour are kept
/// apart by their targets.
const HIGHLIGHTS: [(&str, &str, f64); 6] = [
    ("cursor", "accent", 1.7),
    ("hint_cell", "hint_placement", 1.7),
    ("same_digit", "digit", 1.45),
    ("filter", "box_mark", 1.45),
    ("hint_area", "hint_placement", 1.25),
    ("peer", "text", 1.08),
];

/// The order roles are written in.
const ORDER: [&str; 21] = [
    "background",
    "text",
    "dimmed",
    "accent",
    "frame",
    "line",
    "box_line",
    "clue",
    "digit",
    "candidate",
    "box_mark",
    "cursor",
    "same_digit",
    "peer",
    "filter",
    "conflict",
    "wrong_digit",
    "hint_area",
    "hint_cell",
    "hint_removal",
    "hint_placement",
];

struct Source {
    id: &'static str,
    name: &'static str,
    background: &'static str,
    text: &'static str,
    accent: &'static str,
    digit: &'static str,
    box_mark: &'static str,
    conflict: &'static str,
    placement: &'static str,
}

const THEMES: [Source; 7] = [
    Source {
        id: "tokyo-night-amoled",
        name: "Tokyo Night AMOLED",
        background: "#000000",
        text: "#c0caf5",
        accent: "#7aa2f7",
        digit: "#7dcfff",
        box_mark: "#ff9e64",
        conflict: "#f7768e",
        placement: "#9ece6a",
    },
    Source {
        id: "tokyo-night",
        name: "Tokyo Night",
        background: "#1a1b26",
        text: "#c0caf5",
        accent: "#7aa2f7",
        digit: "#7dcfff",
        box_mark: "#ff9e64",
        conflict: "#f7768e",
        placement: "#9ece6a",
    },
    Source {
        id: "catppuccin-mocha",
        name: "Catppuccin Mocha",
        background: "#1e1e2e",
        text: "#cdd6f4",
        accent: "#89b4fa",
        digit: "#89dceb",
        box_mark: "#fab387",
        conflict: "#f38ba8",
        placement: "#a6e3a1",
    },
    Source {
        id: "nord",
        name: "Nord",
        background: "#2e3440",
        text: "#eceff4",
        accent: "#88c0d0",
        digit: "#8fbcbb",
        box_mark: "#d08770",
        conflict: "#bf616a",
        placement: "#a3be8c",
    },
    Source {
        id: "gruvbox",
        name: "Gruvbox",
        background: "#282828",
        text: "#ebdbb2",
        accent: "#83a598",
        digit: "#8ec07c",
        box_mark: "#fe8019",
        conflict: "#fb4934",
        placement: "#b8bb26",
    },
    Source {
        id: "paper",
        name: "Paper",
        background: "#f6f2e7",
        text: "#26283a",
        accent: "#1d4f9f",
        digit: "#0a5a85",
        box_mark: "#8a4300",
        conflict: "#a3162f",
        placement: "#2a6615",
    },
    // Okabe and Ito's palette: sky blue, orange, vermillion, bluish green.
    Source {
        id: "okabe-ito",
        name: "Colour-blind safe",
        background: "#000000",
        text: "#f2f2f2",
        accent: "#56b4e9",
        digit: "#56b4e9",
        box_mark: "#e69f00",
        conflict: "#d55e00",
        placement: "#009e73",
    },
];

fn hex(s: &str) -> Rgb {
    let channel = |i: usize| u8::from_str_radix(&s[1 + i..3 + i], 16).unwrap();
    [channel(0), channel(2), channel(4)]
}

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [0, 1, 2].map(|i| (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * t).round() as u8)
}

fn gamma(v: f64) -> f64 {
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Linear RGB from OKLab.
fn linear_from_oklab([l, a, b]: [f64; 3]) -> [f64; 3] {
    let l_ = (l + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m_ = (l - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s_ = (l - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    [
        4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
        -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
        -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_,
    ]
}

/// The colour at OKLCh (l, c, h), with its chroma cut until it is in sRGB.
fn from_lch(l: f64, mut c: f64, h: f64) -> Rgb {
    loop {
        let lin = linear_from_oklab([l, c * h.cos(), c * h.sin()]);
        if lin.iter().all(|v| (-1e-4..=1.0 + 1e-4).contains(v)) || c < 1e-4 {
            return lin.map(|v| (gamma(v.clamp(0.0, 1.0)) * 255.0).round() as u8);
        }
        c *= 0.97;
    }
}

/// `colour` with its OKLCh lightness moved away from `background` until it
/// reads at `floor`:1.
fn lift(mut colour: Rgb, background: Rgb, floor: f64) -> Rgb {
    let [l, a, b] = colour::oklab(colour, None);
    let (chroma, hue) = (a.hypot(b), b.atan2(a));
    let step = if luminance(background) < 0.18 {
        0.005
    } else {
        -0.005
    };
    let mut l = l;
    while contrast(colour, background) < floor && (0.0..1.0).contains(&l) {
        l += step;
        colour = from_lch(l, chroma, hue);
    }
    colour
}

fn build(s: &Source) -> BTreeMap<&'static str, Rgb> {
    let (bg, text) = (hex(s.background), hex(s.text));
    let toward = if luminance(bg) > 0.18 {
        [0; 3]
    } else {
        [255; 3]
    };
    let mut c: BTreeMap<&str, Rgb> = BTreeMap::new();
    c.insert("background", bg);
    c.insert("text", text);
    c.insert("clue", text);
    c.insert("accent", hex(s.accent));
    c.insert("digit", hex(s.digit));
    c.insert("box_mark", hex(s.box_mark));
    c.insert("conflict", hex(s.conflict));
    c.insert("hint_placement", hex(s.placement));
    c.insert("wrong_digit", mix(hex(s.conflict), toward, 0.2));
    c.insert("hint_removal", hex(s.conflict));
    c.insert("dimmed", mix(text, bg, 0.22));
    c.insert("candidate", mix(text, bg, 0.12));
    for role in TEXT {
        c.insert(role, lift(c[role], bg, LIFT));
    }
    // The frame sits between the two kinds of line, and box lines at 4.5:1
    // lead cell lines by well over BOX_LINE_LEAD, which the tests check.
    for (role, floor) in [
        ("line", GRAPHIC_FLOOR + MARGIN),
        ("frame", 3.6),
        ("box_line", 4.5),
    ] {
        let mut t = 0.0;
        while contrast(mix(bg, text, t), bg) < floor {
            t += 0.005;
        }
        c.insert(role, mix(bg, text, t));
    }
    for (role, source, target) in HIGHLIGHTS {
        let source = c[source];
        let (mut t, mut best) = (0.0, bg);
        while t < 1.0 {
            t += 0.005;
            let colour = mix(bg, source, t);
            let unreadable = TEXT
                .iter()
                .any(|x| contrast(c[x], colour) < TEXT_FLOOR + MARGIN);
            let cursor_marks =
                role == "cursor" && contrast(c["accent"], colour) < GRAPHIC_FLOOR + MARGIN;
            if unreadable || cursor_marks {
                break;
            }
            best = colour;
            if contrast(colour, bg) >= target {
                break;
            }
        }
        c.insert(role, best);
    }
    c
}

fn main() {
    let out = env::args().nth(1);
    for source in &THEMES {
        let c = build(source);
        let worst = TEXT
            .iter()
            .flat_map(|t| SURFACES.iter().map(move |s| (t, s)))
            .map(|(t, s)| contrast(c[t], c[s]))
            .fold(f64::INFINITY, f64::min);
        // Only the colour-blind theme promises the simulated floors.
        let simulated: &[Cvd] = if source.id == "okabe-ito" {
            &Cvd::ALL
        } else {
            &[]
        };
        let mut close = too_close(|r| c[r], None);
        for &cvd in simulated {
            close.extend(
                too_close(|r| c[r], Some(cvd))
                    .into_iter()
                    .map(|p| format!("{cvd:?} {p}")),
            );
        }
        let verdict = if close.is_empty() {
            "ok".to_string()
        } else {
            close.join("; ")
        };
        println!("{:20} text {worst:.2}  {verdict}", source.id);
        if let Some(dir) = &out {
            let mut body = format!("name = \"{}\"\n\n[colors]\n", source.name);
            for role in ORDER {
                let [r, g, b] = c[role];
                body += &format!("{role} = \"#{r:02x}{g:02x}{b:02x}\"\n");
            }
            fs::write(Path::new(dir).join(format!("{}.toml", source.id)), body).unwrap();
        }
    }
}
