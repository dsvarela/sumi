//! Colour roles. Every colour on screen comes from one of these, so a theme
//! changes everything by changing this struct.

use ratatui::style::Color;

pub struct Theme {
    pub background: Color,
    /// Text that isn't on the board.
    pub text: Color,
    pub dimmed: Color,
    /// Titles, keys in the key bar, and the marks at the cursor's corners.
    pub accent: Color,
    pub frame: Color,
    /// Lines between cells inside a box.
    pub line: Color,
    /// Lines between boxes and around the board.
    pub box_line: Color,
    pub clue: Color,
    pub digit: Color,
    pub candidate: Color,
    pub box_mark: Color,
    /// Background of the selected cell.
    pub cursor: Color,
    /// Background of cells holding the digit under the cursor.
    pub same_digit: Color,
    /// Background of the cursor's row, column and box.
    pub peer: Color,
    /// Background of cells the candidate filter lights up.
    pub filter: Color,
    pub conflict: Color,
}

impl Theme {
    /// Tokyo Night with a pure black background, for AMOLED screens.
    pub fn tokyo_night_amoled() -> Theme {
        Theme {
            background: Color::Rgb(0x00, 0x00, 0x00),
            text: Color::Rgb(0xc0, 0xca, 0xf5),
            dimmed: Color::Rgb(0x8f, 0x99, 0xc6),
            accent: Color::Rgb(0x7a, 0xa2, 0xf7),
            frame: Color::Rgb(0x5a, 0x63, 0x90),
            line: Color::Rgb(0x54, 0x5c, 0x7e),
            box_line: Color::Rgb(0x73, 0x7a, 0xa2),
            clue: Color::Rgb(0xc0, 0xca, 0xf5),
            digit: Color::Rgb(0x7d, 0xcf, 0xff),
            candidate: Color::Rgb(0x9a, 0xa5, 0xce),
            box_mark: Color::Rgb(0xff, 0x9e, 0x64),
            cursor: Color::Rgb(0x22, 0x2c, 0x4c),
            same_digit: Color::Rgb(0x1c, 0x24, 0x40),
            peer: Color::Rgb(0x0f, 0x11, 0x19),
            filter: Color::Rgb(0x2e, 0x26, 0x10),
            conflict: Color::Rgb(0xf7, 0x76, 0x8e),
        }
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

    /// Text needs 4.5:1 (WCAG 1.4.3) on every background it can be drawn on,
    /// and any board text can sit on any cell highlight. Lines, the frame
    /// and the cursor's corner marks need 3:1 (WCAG 1.4.11). The highlights
    /// stay below 3:1 against black, since text on them must stay readable;
    /// the cursor is shown by its corner marks as well as its colour.
    #[test]
    fn every_colour_pair_on_screen_is_readable() {
        let t = Theme::tokyo_night_amoled();
        let backgrounds = [
            ("background", t.background),
            ("cursor", t.cursor),
            ("same_digit", t.same_digit),
            ("peer", t.peer),
            ("filter", t.filter),
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
        ];
        for (bg_name, bg) in backgrounds {
            for (fg_name, fg) in text {
                let ratio = contrast(fg, bg);
                assert!(ratio >= 4.5, "{fg_name} on {bg_name}: {ratio:.2}");
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
            assert!(ratio >= 3.0, "{name}: {ratio:.2}");
        }
    }
}
