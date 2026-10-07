//! Drawing. Screens take every colour from the theme and draw their frame and
//! key bar with the shared functions here, so they all look alike.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget};

use crate::game::Game;
use crate::theme::Theme;

/// A cell is 7 columns by 3 rows, with a one-character line between cells.
const CELL_WIDTH: u16 = 7;
const CELL_HEIGHT: u16 = 3;
const BOARD_WIDTH: u16 = 9 * (CELL_WIDTH + 1) + 1;
const BOARD_HEIGHT: u16 = 9 * (CELL_HEIGHT + 1) + 1;

pub fn draw(frame: &mut Frame, game: &Game, theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);

    let needed = Rect::new(0, 0, BOARD_WIDTH + 2, BOARD_HEIGHT + 2);
    if area.width < needed.width || area.height < needed.height {
        too_small(frame, needed, theme);
        return;
    }

    let outer = area.centered(
        Constraint::Length(needed.width),
        Constraint::Length(needed.height),
    );
    let (status, keys) = if game.is_won() {
        (
            Span::styled("Solved", Style::new().fg(theme.accent)),
            key_bar(&[("Enter", "new game"), ("q", "quit")], theme),
        )
    } else {
        (
            Span::styled("● DIGIT", Style::new().fg(theme.dimmed)),
            key_bar(
                &[
                    ("hjkl", "move"),
                    ("1-9", "place"),
                    ("0", "clear"),
                    ("u", "undo"),
                    ("^r", "redo"),
                    ("q", "quit"),
                ],
                theme,
            ),
        )
    };
    let block = frame_block(theme)
        .title_top(Line::from(status).right_aligned())
        .title_bottom(keys.centered());
    let inner = block.inner(outer);
    frame.render_widget(block, outer);
    frame.render_widget(Board { game, theme }, inner);
}

/// The rounded frame every screen sits in, titled with the program's name.
pub fn frame_block(theme: &Theme) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.frame))
        .title_top(Line::from(Span::styled(
            " sumi ",
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
        )))
}

/// A row of keys and what they do, keys in the accent colour.
pub fn key_bar(keys: &[(&str, &str)], theme: &Theme) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, action)) in keys.iter().enumerate() {
        let gap = if i == 0 { " " } else { "  " };
        spans.push(Span::raw(gap));
        spans.push(Span::styled(key.to_string(), Style::new().fg(theme.accent)));
        spans.push(Span::styled(
            format!(" {action}"),
            Style::new().fg(theme.dimmed),
        ));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

fn too_small(frame: &mut Frame, needed: Rect, theme: &Theme) {
    let area = frame.area();
    let message = format!(
        "Make the window at least {}×{}.\nIt is {}×{} now.",
        needed.width, needed.height, area.width, area.height
    );
    let height = 2.min(area.height);
    let line = area.centered_vertically(Constraint::Length(height));
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::new().fg(theme.text))
            .centered(),
        line,
    );
}

struct Board<'a> {
    game: &'a Game,
    theme: &'a Theme,
}

impl Widget for Board<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.centered(
            Constraint::Length(BOARD_WIDTH),
            Constraint::Length(BOARD_HEIGHT),
        );
        self.render_lines(area, buf);
        let conflicts = self.game.board().conflicts();
        for (cell, &conflict) in conflicts.iter().enumerate() {
            self.render_cell(cell, conflict, area, buf);
        }
    }
}

impl Board<'_> {
    fn render_lines(&self, area: Rect, buf: &mut Buffer) {
        for dy in 0..BOARD_HEIGHT {
            for dx in 0..BOARD_WIDTH {
                let on_vertical = dx % (CELL_WIDTH + 1) == 0;
                let on_horizontal = dy % (CELL_HEIGHT + 1) == 0;
                let kx = dx / (CELL_WIDTH + 1);
                let ky = dy / (CELL_HEIGHT + 1);
                let (symbol, heavy) = match (on_vertical, on_horizontal) {
                    (true, true) => junction(kx, ky),
                    (true, false) if kx.is_multiple_of(3) => ("┃", true),
                    (true, false) => ("│", false),
                    (false, true) if ky.is_multiple_of(3) => ("━", true),
                    (false, true) => ("─", false),
                    (false, false) => continue,
                };
                let colour = if heavy {
                    self.theme.box_line
                } else {
                    self.theme.line
                };
                if let Some(c) = buf.cell_mut((area.x + dx, area.y + dy)) {
                    c.set_symbol(symbol).set_fg(colour);
                }
            }
        }
    }

    fn render_cell(&self, cell: usize, conflict: bool, area: Rect, buf: &mut Buffer) {
        let x = area.x + 1 + (cell % 9) as u16 * (CELL_WIDTH + 1);
        let y = area.y + 1 + (cell / 9) as u16 * (CELL_HEIGHT + 1);
        let rect = Rect::new(x, y, CELL_WIDTH, CELL_HEIGHT);

        if cell == self.game.cursor() {
            buf.set_style(rect, Style::new().bg(self.theme.cursor));
        }
        let Some(digit) = self.game.board().get(cell) else {
            return;
        };
        let colour = if conflict {
            self.theme.conflict
        } else if self.game.is_clue(cell) {
            self.theme.clue
        } else {
            self.theme.digit
        };
        buf.set_string(
            x + CELL_WIDTH / 2,
            y + CELL_HEIGHT / 2,
            digit.to_string(),
            Style::new().fg(colour).add_modifier(Modifier::BOLD),
        );
    }
}

/// Where a vertical line `kx` (0 to 9, left to right) crosses a horizontal
/// line `ky`. Every third line is heavy, the outer edge included. Returns the
/// symbol and whether any part of it is heavy.
fn junction(kx: u16, ky: u16) -> (&'static str, bool) {
    let heavy_v = kx.is_multiple_of(3);
    let heavy_h = ky.is_multiple_of(3);
    let symbol = match (kx, ky) {
        (0, 0) => "┏",
        (9, 0) => "┓",
        (0, 9) => "┗",
        (9, 9) => "┛",
        (_, 0) if heavy_v => "┳",
        (_, 0) => "┯",
        (_, 9) if heavy_v => "┻",
        (_, 9) => "┷",
        (0, _) if heavy_h => "┣",
        (0, _) => "┠",
        (9, _) if heavy_h => "┫",
        (9, _) => "┨",
        _ => match (heavy_v, heavy_h) {
            (true, true) => "╋",
            (true, false) => "╂",
            (false, true) => "┿",
            (false, false) => "┼",
        },
    };
    (symbol, heavy_v || heavy_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Direction;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use sumi_core::generate::Puzzle;
    use sumi_core::grid::Grid;
    use sumi_core::solver::{Solutions, solve};

    const CLUES: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    fn game() -> Game {
        let clues: Grid = CLUES.parse().unwrap();
        let Solutions::Unique(solution) = solve(&clues) else {
            panic!("test puzzle has one solution");
        };
        Game::new(Puzzle { clues, solution })
    }

    fn render(game: &Game, width: u16, height: u16) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| draw(frame, game, &Theme::tokyo_night_amoled()))
            .unwrap();
        terminal.backend().clone()
    }

    #[test]
    fn full_layout() {
        let mut game = game();
        game.step(Direction::Right);
        game.step(Direction::Right);
        game.write(Some(5)); // clashes with the 5 at r1c1
        insta::assert_snapshot!(render(&game, 75, 39));
    }

    #[test]
    fn clashing_digits_and_the_cursor_are_coloured() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        game.step(Direction::Right);
        game.step(Direction::Right);
        game.write(Some(5)); // r1c3, clashes with the 5 at r1c1
        let backend = render(&game, 75, 39);
        let buffer = backend.buffer();

        // Digits sit in the middle of their cell: column 5 + 8 per cell, row 3.
        let clue_5 = &buffer[(5, 3)];
        let clue_3 = &buffer[(13, 3)];
        let typed_5 = &buffer[(21, 3)];
        assert_eq!((clue_5.symbol(), clue_5.fg), ("5", theme.conflict));
        assert_eq!((clue_3.symbol(), clue_3.fg), ("3", theme.clue));
        assert_eq!((typed_5.symbol(), typed_5.fg), ("5", theme.conflict));
        assert_eq!(typed_5.bg, theme.cursor);
        assert_eq!(buffer[(18, 2)].bg, theme.cursor);
        assert_ne!(clue_3.bg, theme.cursor);

        game.write(Some(4));
        let backend = render(&game, 75, 39);
        assert_eq!(backend.buffer()[(21, 3)].fg, theme.digit);
        assert_eq!(backend.buffer()[(5, 3)].fg, theme.clue);
    }

    #[test]
    fn too_small_window() {
        insta::assert_snapshot!(render(&game(), 40, 10));
    }
}
