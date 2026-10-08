//! Colour arithmetic shared by the theme tests and `examples/build_themes.rs`:
//! WCAG contrast, OKLab distance, simulated colour blindness, and the
//! floors. The generator builds to these floors and the tests check them,
//! so the two can't disagree about what a theme must meet.

pub type Rgb = [u8; 3];

/// WCAG 2.2: text needs 4.5:1 on what it sits on (1.4.3); lines, the frame
/// and the cursor's marks need 3:1 (1.4.11).
pub const TEXT_FLOOR: f64 = 4.5;
pub const GRAPHIC_FLOOR: f64 = 3.0;

/// How far above a floor the generator aims, so that rounding a colour to
/// whole RGB values can't take it under.
#[allow(dead_code, reason = "only the generator aims above a floor")]
pub const MARGIN: f64 = 0.05;

/// Box lines stand out from the lines between cells by at least this much
/// contrast against the background.
pub const BOX_LINE_LEAD: f64 = 0.5;

/// Highlights stay under this against the background: any more and the
/// text drawn on them could not keep its own floor.
pub const HIGHLIGHT_CEILING: f64 = 3.0;

/// Roles drawn as text, and the surfaces text sits on.
pub const TEXT: [&str; 11] = [
    "text",
    "dimmed",
    "accent",
    "clue",
    "digit",
    "candidate",
    "box_mark",
    "conflict",
    "wrong_digit",
    "hint_removal",
    "hint_placement",
];
pub const SURFACES: [&str; 7] = [
    "background",
    "cursor",
    "same_digit",
    "peer",
    "filter",
    "hint_area",
    "hint_cell",
];

/// Pairs that must look different, and the least OKLab distance between
/// them. Peer shading is meant to be faint. The text pairs also
/// differ by weight or a mark, so colour is a second cue for them.
pub const APART: &[(&str, &str, f64)] = &[
    ("background", "cursor", 0.04),
    // The cursor has its own marks, but a cursor that looked like the
    // same-digit highlight would still mislead.
    ("cursor", "same_digit", 0.02),
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

/// A kind of colour blindness, simulated with the matrices of Machado,
/// Oliveira and Fernandes (2009) at severity 1, on linear RGB.
#[derive(Debug, Clone, Copy)]
pub enum Cvd {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl Cvd {
    pub const ALL: [Cvd; 3] = [Cvd::Protanopia, Cvd::Deuteranopia, Cvd::Tritanopia];

    fn matrix(self) -> [[f64; 3]; 3] {
        match self {
            Cvd::Protanopia => [
                [0.152286, 1.052583, -0.204868],
                [0.114503, 0.786281, 0.099216],
                [-0.003882, -0.048116, 1.051998],
            ],
            Cvd::Deuteranopia => [
                [0.367322, 0.860646, -0.227968],
                [0.280085, 0.672501, 0.047413],
                [-0.011820, 0.042940, 0.968881],
            ],
            Cvd::Tritanopia => [
                [1.255528, -0.076749, -0.178779],
                [-0.078411, 0.930809, 0.147602],
                [0.004733, 0.691367, 0.303900],
            ],
        }
    }
}

/// An sRGB channel in linear light, from 0 to 1.
pub fn linear(v: u8) -> f64 {
    let v = f64::from(v) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

pub fn luminance(c: Rgb) -> f64 {
    let [r, g, b] = c.map(linear);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// WCAG 2.2 contrast ratio.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// OKLab from linear RGB.
pub fn oklab_from_linear([r, g, b]: [f64; 3]) -> [f64; 3] {
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

/// The colour in OKLab as someone with `cvd` sees it, or with full colour
/// vision for `None`.
pub fn oklab(c: Rgb, cvd: Option<Cvd>) -> [f64; 3] {
    let mut rgb = c.map(linear);
    if let Some(cvd) = cvd {
        let m = cvd.matrix();
        let mixed = |i: usize| (0..3).map(|j| m[i][j] * rgb[j]).sum::<f64>();
        rgb = [0, 1, 2].map(|i| mixed(i).clamp(0.0, 1.0));
    }
    oklab_from_linear(rgb)
}

pub fn distance(a: Rgb, b: Rgb, cvd: Option<Cvd>) -> f64 {
    let (p, q) = (oklab(a, cvd), oklab(b, cvd));
    (0..3).map(|i| (p[i] - q[i]).powi(2)).sum::<f64>().sqrt()
}

/// The pairs in `APART` that `role` puts too close, as `a/b distance`.
pub fn too_close(role: impl Fn(&str) -> Rgb, cvd: Option<Cvd>) -> Vec<String> {
    APART
        .iter()
        .filter_map(|&(a, b, floor)| {
            let d = distance(role(a), role(b), cvd);
            (d < floor).then(|| format!("{a}/{b} {d:.3}"))
        })
        .collect()
}
