//! Colour roles. Every colour on screen comes from one of these, so a theme
//! changes everything by changing this struct.

use ratatui::style::Color;

pub struct Theme {
    pub background: Color,
    /// Text that isn't on the board.
    pub text: Color,
    pub dimmed: Color,
    pub accent: Color,
    pub frame: Color,
    /// Lines between cells inside a box.
    pub line: Color,
    /// Lines between boxes and around the board.
    pub box_line: Color,
    pub clue: Color,
    pub digit: Color,
    /// Background of the selected cell.
    pub cursor: Color,
    pub conflict: Color,
}

impl Theme {
    /// Tokyo Night with a pure black background, for AMOLED screens.
    pub fn tokyo_night_amoled() -> Theme {
        Theme {
            background: Color::Rgb(0x00, 0x00, 0x00),
            text: Color::Rgb(0xc0, 0xca, 0xf5),
            dimmed: Color::Rgb(0x56, 0x5f, 0x89),
            accent: Color::Rgb(0x7a, 0xa2, 0xf7),
            frame: Color::Rgb(0x3b, 0x42, 0x61),
            line: Color::Rgb(0x29, 0x2e, 0x42),
            box_line: Color::Rgb(0x56, 0x5f, 0x89),
            clue: Color::Rgb(0xc0, 0xca, 0xf5),
            digit: Color::Rgb(0x7d, 0xcf, 0xff),
            cursor: Color::Rgb(0x2e, 0x3c, 0x64),
            conflict: Color::Rgb(0xf7, 0x76, 0x8e),
        }
    }
}
