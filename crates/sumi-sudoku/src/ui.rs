//! Drawing. Every colour comes from the theme, and every screen is meant to
//! draw its frame and key bar with `frame_block` and `key_bar`, so that
//! screens added later look like this one.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Widget, Wrap};

use sumi_core::grid::{Cell, col, row, sees};
use sumi_core::hint::{Hint, Rung};

use crate::game::{Game, Mode};
use crate::input::{Action, key_label};
use crate::theme::Theme;

/// A cell is 7 columns by 3 rows, with a one-character line between cells.
const CELL_WIDTH: u16 = 7;
const CELL_HEIGHT: u16 = 3;
const BOARD_WIDTH: u16 = 9 * (CELL_WIDTH + 1) + 1;
const BOARD_HEIGHT: u16 = 9 * (CELL_HEIGHT + 1) + 1;

/// The panel under the board is this tall, borders included: the rung's
/// number and two lines of text, which every hint rung fits in at the
/// board's width.
const PANEL_HEIGHT: u16 = 5;

/// The most text the status line can have beside the frame's title: the
/// board's width less " sumi ", a border character, and the line's padding.
const STATUS_WIDTH: usize = BOARD_WIDTH as usize - 6 - 1 - 2;

/// What the screen shows besides the game itself. None of it is undoable.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct View {
    pub filter: Option<Filter>,
    pub notes_hidden: bool,
    /// `f` was pressed and the next key picks the filter.
    pub awaiting_filter: bool,
    pub hint: Option<OpenHint>,
    /// The digits `c` found wrong. A mark stays while its cell still holds
    /// that digit.
    pub checked: Vec<(Cell, u8)>,
    /// A one-word answer to the last key, shown in the status line.
    pub notice: Option<&'static str>,
}

/// A hint on its way up the ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenHint {
    pub hint: Hint,
    /// The rung showing, from 1.
    pub rung: usize,
    /// The game's revision the hint was worked out at; at any other the
    /// board has changed and the hint no longer holds.
    pub revision: u64,
}

/// Which empty cells the candidate filter lights up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Cells where this digit is still a candidate.
    Digit(u8),
    /// Cells with exactly two candidates.
    TwoCandidates,
}

const FRAME_WIDTH: u16 = BOARD_WIDTH + 2;

/// The board inside its frame and the panel under it: the smallest window
/// the game can be drawn in.
const NEEDED: (u16, u16) = (FRAME_WIDTH, BOARD_HEIGHT + 2 + PANEL_HEIGHT);

/// Whether `draw` shows the board in a window of this size. The app asks
/// this too, so keys never act on a board the player can't see.
pub fn fits(area: Rect) -> bool {
    area.width >= NEEDED.0 && area.height >= NEEDED.1
}

pub fn draw(frame: &mut Frame, game: &Game, view: &View, theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);
    if !fits(area) {
        too_small(frame, theme);
        return;
    }

    let whole = area.centered(Constraint::Length(NEEDED.0), Constraint::Length(NEEDED.1));
    let [outer, side] = Layout::vertical([
        Constraint::Length(BOARD_HEIGHT + 2),
        Constraint::Length(PANEL_HEIGHT),
    ])
    .areas(whole);
    let block = frame_block(theme)
        .title_top(status(game, view, theme).right_aligned())
        .title_bottom(key_bar(&keys(game, view), theme).centered());
    let inner = block.inner(outer);
    frame.render_widget(block, outer);
    let rung = view.hint.as_ref().map(|open| open.hint.rung(open.rung));
    frame.render_widget(
        Board {
            game,
            view,
            theme,
            rung,
        },
        inner,
    );
    panel(frame, side, view, theme);
}

/// The panel under the board: the hint's words, or how to ask for one.
fn panel(frame: &mut Frame, area: Rect, view: &View, theme: &Theme) {
    let block = panel_block(theme, "Hint");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(panel_text(view, theme), inner);
}

/// The panel's words, wrapped as they are drawn.
fn panel_text(view: &View, theme: &Theme) -> Paragraph<'static> {
    let text = match &view.hint {
        Some(open) => {
            let rung = open.hint.rung(open.rung);
            vec![
                Line::from(Span::styled(
                    format!("Rung {} of {}", open.rung, open.hint.rungs()),
                    Style::new().fg(theme.dimmed),
                )),
                Line::from(Span::styled(rung.text, Style::new().fg(theme.text))),
            ]
        }
        None => vec![Line::from(Span::styled(
            format!("{} for a hint", key_label(Action::Hint)),
            Style::new().fg(theme.dimmed),
        ))],
    };
    Paragraph::new(text).wrap(Wrap { trim: true })
}

/// The block that holds a panel's text, with a title.
pub fn panel_block(theme: &Theme, title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.frame))
        .padding(Padding::horizontal(1))
        .title_top(Line::from(Span::styled(
            format!(" {title} "),
            Style::new().fg(theme.accent),
        )))
}

fn status(game: &Game, view: &View, theme: &Theme) -> Line<'static> {
    if game.is_won() {
        return Line::from(Span::styled(" Solved ", Style::new().fg(theme.accent)));
    }
    let mode = match game.mode() {
        Mode::Digit => "● DIGIT",
        Mode::Candidate => "● CANDIDATE",
        Mode::BoxMark => "● BOX MARK",
    };
    // Most useful first. What does not fit beside the title is left off from
    // the end, so the mode, which is always shown, is never lost.
    let mut optional: Vec<String> = Vec::new();
    if let Some(open) = &view.hint {
        optional.push(format!("hint {}/{}", open.rung, open.hint.rungs()));
    }
    if let Some(notice) = view.notice {
        optional.push(notice.to_string());
    }
    match view.filter {
        Some(Filter::Digit(d)) => optional.push(format!("filter {d}")),
        Some(Filter::TwoCandidates) => optional.push("filter pairs".to_string()),
        None => {}
    }
    if view.notes_hidden {
        optional.push("candidates and box marks hidden".to_string());
    }
    let fits = |parts: &[String]| {
        let shown: Vec<&str> = parts.iter().map(String::as_str).chain([mode]).collect();
        shown.join(" · ").chars().count() <= STATUS_WIDTH
    };
    while !fits(&optional) {
        optional.pop();
    }
    // Shown in the order the player reads them: the setting, the hint, the mode.
    optional.reverse();
    let mut parts = optional;
    parts.push(mode.to_string());
    Line::from(Span::styled(
        format!(" {} ", parts.join(" · ")),
        Style::new().fg(theme.dimmed),
    ))
}

/// The keys the key bar offers, as key text and what the key does. Key
/// text comes from `input::key_label`, so it always matches the bindings.
fn keys(game: &Game, view: &View) -> Vec<(&'static str, &'static str)> {
    if view.awaiting_filter {
        // The key after f is read by `input::filter_key`, not `input::action`.
        return vec![
            ("1-9", "digit"),
            ("b", "pairs"),
            ("0", "off"),
            ("other", "cancel"),
        ];
    }
    if let Some(open) = &view.hint {
        let mut keys = Vec::new();
        if open.rung < open.hint.rungs() {
            keys.push((Action::Hint, "next rung"));
        } else if open.hint.step().is_some() {
            keys.push((Action::Confirm, "apply"));
        }
        keys.push((Action::Step(crate::game::Direction::Up), "look around"));
        keys.push((Action::Close, "close"));
        keys.push((Action::Quit, "quit"));
        return keys
            .into_iter()
            .map(|(action, words)| (key_label(action), words))
            .collect();
    }
    let actions: &[(Action, &str)] = if game.is_won() {
        &[(Action::Confirm, "new game"), (Action::Quit, "quit")]
    } else {
        match game.mode() {
            Mode::Digit => &[
                (Action::Digit(1), "place"),
                (Action::Clear, "clear"),
                (Action::NextMode, "mode"),
                (Action::FillCandidates, "fill"),
                (Action::Filter, "filter"),
                (Action::Hint, "hint"),
                (Action::Undo, "undo"),
                (Action::Quit, "quit"),
            ],
            Mode::Candidate => &[
                (Action::Digit(1), "toggle"),
                (Action::FillCandidates, "fill"),
                (Action::ClearLayer, "clear candidates"),
                (Action::ClearAllNotes, "clear both"),
                (Action::NextMode, "mode"),
                (Action::Quit, "quit"),
            ],
            Mode::BoxMark => &[
                (Action::Digit(1), "toggle"),
                (Action::MarksToCandidates, "to candidates"),
                (Action::ClearLayer, "clear box marks"),
                (Action::NextMode, "mode"),
                (Action::Quit, "quit"),
            ],
        }
    };
    actions
        .iter()
        .map(|&(action, words)| (key_label(action), words))
        .collect()
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

/// The message wraps, so a narrow window still shows as much of it as fits.
fn too_small(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let message = format!(
        "Make the window at least {}×{}. It is {}×{} now.",
        NEEDED.0, NEEDED.1, area.width, area.height
    );
    frame.render_widget(
        Paragraph::new(message)
            .style(Style::new().fg(theme.text))
            .wrap(Wrap { trim: true })
            .centered(),
        area.centered_vertically(Constraint::Length(4.min(area.height))),
    );
}

struct Board<'a> {
    game: &'a Game,
    view: &'a View,
    theme: &'a Theme,
    /// What the open hint shows at its rung.
    rung: Option<Rung>,
}

impl Widget for Board<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.centered(
            Constraint::Length(BOARD_WIDTH),
            Constraint::Length(BOARD_HEIGHT),
        );
        self.render_lines(area, buf);
        self.render_numbers(area, buf);
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

    /// Writes each column's number into the board's top line and each row's
    /// into its left line, in the middle of the cell they belong to. Those
    /// lines are drawn anyway, so the numbers cost no room. The cursor's row
    /// and column stand out.
    fn render_numbers(&self, area: Rect, buf: &mut Buffer) {
        let cursor = self.game.cursor();
        let style = |here: bool| {
            if here {
                Style::new()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(self.theme.dimmed)
            }
        };
        for i in 0..9 {
            let number = (i + 1).to_string();
            let offset = 1 + i as u16 * (CELL_WIDTH + 1);
            buf.set_string(
                area.x + offset + CELL_WIDTH / 2,
                area.y,
                &number,
                style(col(cursor) == i),
            );
            let offset = 1 + i as u16 * (CELL_HEIGHT + 1);
            buf.set_string(
                area.x,
                area.y + offset + CELL_HEIGHT / 2,
                &number,
                style(row(cursor) == i),
            );
        }
    }

    fn render_cell(&self, cell: Cell, conflict: bool, area: Rect, buf: &mut Buffer) {
        let x = area.x + 1 + col(cell) as u16 * (CELL_WIDTH + 1);
        let y = area.y + 1 + row(cell) as u16 * (CELL_HEIGHT + 1);
        let rect = Rect::new(x, y, CELL_WIDTH, CELL_HEIGHT);
        if let Some(bg) = self.background(cell) {
            buf.set_style(rect, Style::new().bg(bg));
        }
        if self.theme.monochrome && self.rung.as_ref().is_some_and(|r| r.cells.contains(&cell)) {
            buf.set_style(rect, Style::new().add_modifier(Modifier::REVERSED));
        }
        if cell == self.game.cursor() {
            // Corner marks show the cursor without relying on colour. Notes
            // never use a cell's first or last column, so they don't collide.
            let mark = Style::new().fg(self.theme.accent);
            let (right, bottom) = (x + CELL_WIDTH - 1, y + CELL_HEIGHT - 1);
            for (cx, cy, symbol) in [
                (x, y, "┌"),
                (right, y, "┐"),
                (x, bottom, "└"),
                (right, bottom, "┘"),
            ] {
                buf.set_string(cx, cy, symbol, mark);
            }
        }

        match self.game.board().get(cell) {
            Some(digit) => {
                // On a solved board every digit is complete, and dimming them
                // all would grey out the finished puzzle.
                let checked = self.view.checked.contains(&(cell, digit));
                let colour = if checked {
                    self.theme.wrong_digit
                } else if conflict {
                    self.theme.conflict
                } else if self.game.is_complete(digit) && !self.game.is_won() {
                    self.theme.dimmed
                } else if self.game.is_clue(cell) {
                    self.theme.clue
                } else {
                    self.theme.digit
                };
                // Each kind of digit differs by more than hue: clues are
                // bold, a clash is reversed and a wrong digit underlined.
                let mut style = Style::new().fg(colour);
                if self.game.is_clue(cell) {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if conflict {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                if checked {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                buf.set_string(
                    x + CELL_WIDTH / 2,
                    y + CELL_HEIGHT / 2,
                    digit.to_string(),
                    style,
                );
            }
            None if !self.view.notes_hidden => self.render_notes(cell, x, y, buf),
            None => {}
        }
        self.render_hint_marks(cell, x, y, buf);
    }

    /// What the open hint writes in a cell: the candidates it removes,
    /// crossed out, or the digit it places, which takes the cell's notes'
    /// place.
    fn render_hint_marks(&self, cell: Cell, x: u16, y: u16, buf: &mut Buffer) {
        let Some(rung) = &self.rung else { return };
        for &(c, digits) in rung.removals.iter().filter(|&&(c, _)| c == cell) {
            debug_assert_eq!(c, cell);
            for digit in digits.iter() {
                let spot = u16::from(digit - 1);
                buf.set_string(
                    x + 1 + 2 * (spot % 3),
                    y + spot / 3,
                    digit.to_string(),
                    Style::new()
                        .fg(self.theme.hint_removal)
                        .add_modifier(Modifier::CROSSED_OUT),
                );
            }
        }
        if let Some((c, digit)) = rung.placement
            && c == cell
        {
            for dy in 0..CELL_HEIGHT {
                buf.set_string(
                    x + 1,
                    y + dy,
                    " ".repeat(usize::from(CELL_WIDTH) - 2),
                    Style::new(),
                );
            }
            buf.set_string(
                x + CELL_WIDTH / 2,
                y + CELL_HEIGHT / 2,
                digit.to_string(),
                Style::new()
                    .fg(self.theme.hint_placement)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }

    /// Each digit has a fixed spot in a 3×3 grid, 1 top left and 9 bottom
    /// right, spaced out across the cell's 7 columns. A box mark takes the
    /// spot over from a candidate.
    fn render_notes(&self, cell: Cell, x: u16, y: u16, buf: &mut Buffer) {
        let candidates = self.game.candidates(cell);
        let marks = self.game.marks(cell);
        for digit in candidates.union(marks).iter() {
            let spot = u16::from(digit - 1);
            let mut style = if marks.contains(digit) {
                Style::new()
                    .fg(self.theme.box_mark)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::new().fg(self.theme.candidate)
            };
            if self.game.note_clashes(cell, digit) {
                style = style
                    .fg(self.theme.conflict)
                    .add_modifier(Modifier::REVERSED);
            }
            buf.set_string(
                x + 1 + 2 * (spot % 3),
                y + spot / 3,
                digit.to_string(),
                style,
            );
        }
    }

    /// A hint's cells win over everything, the cursor's corner marks being
    /// enough to find the cursor. The cursor wins over the filter and the
    /// same-digit highlight, and those win over a hint's area, which wins
    /// over the shading of the cursor's row, column and box. The
    /// filter lights only empty cells and the same-digit highlight only
    /// filled ones, so those two never compete.
    fn background(&self, cell: Cell) -> Option<Color> {
        let game = self.game;
        let cursor = game.cursor();
        let filtered = match self.view.filter {
            Some(Filter::Digit(d)) => game.working_candidates(cell).contains(d),
            Some(Filter::TwoCandidates) => game.working_candidates(cell).len() == 2,
            None => false,
        };
        let digit = game.board().get(cell);
        let rung = self.rung.as_ref();
        if rung.is_some_and(|r| r.cells.contains(&cell)) {
            Some(self.theme.hint_cell)
        } else if cell == cursor {
            Some(self.theme.cursor)
        } else if filtered {
            Some(self.theme.filter)
        } else if digit.is_some() && digit == game.board().get(cursor) {
            Some(self.theme.same_digit)
        } else if rung.is_some_and(|r| {
            r.second_group.contains(&cell) || r.areas.iter().any(|h| h.contains(cell))
        }) {
            Some(self.theme.hint_area)
        } else if sees(cell, cursor) {
            Some(self.theme.peer)
        } else {
            None
        }
    }
}

/// Where a vertical line `kx` (0 to 9, left to right) crosses a horizontal
/// line `ky` (0 to 9, top to bottom). Every third line is heavy, the outer
/// edge included. Returns the symbol and whether any part of it is heavy.
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
    use crate::fixtures::{game, solution};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(game: &Game, view: &View) -> TestBackend {
        render_in(game, view, &Theme::tokyo_night_amoled())
    }

    fn render_in(game: &Game, view: &View, theme: &Theme) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(NEEDED.0, NEEDED.1)).unwrap();
        terminal
            .draw(|frame| draw(frame, game, view, theme))
            .unwrap();
        terminal.backend().clone()
    }

    /// The screen position of a cell's middle character, for the full layout
    /// in a window of exactly `NEEDED`: the frame and the board's outer line take one
    /// column and row each.
    fn middle(cell: Cell) -> (u16, u16) {
        (5 + 8 * col(cell) as u16, 3 + 4 * row(cell) as u16)
    }

    /// Moves the cursor from wherever it is to `cell`, going right along the
    /// row and then down the column, wrapping as needed.
    fn go(game: &mut Game, cell: Cell) {
        use crate::game::Direction;
        while game.cursor() != cell {
            if col(game.cursor()) != col(cell) {
                game.step(Direction::Right);
            } else {
                game.step(Direction::Down);
            }
        }
    }

    #[test]
    fn full_layout_with_notes() {
        let mut game = game();
        game.fill_candidates();
        go(&mut game, 2);
        game.press(5); // r1c3 clashes with the 5 at r1c1
        game.next_mode();
        game.next_mode();
        go(&mut game, 3);
        game.press(6); // a box mark at r1c4
        insta::assert_snapshot!(render(&game, &View::default()));
    }

    #[test]
    fn digits_are_coloured_by_kind() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 2);
        game.press(5);
        let backend = render(&game, &View::default());
        let buffer = backend.buffer();
        let clue_5 = &buffer[middle(0)];
        let clue_3 = &buffer[middle(1)];
        let typed_5 = &buffer[middle(2)];
        assert_eq!((clue_5.symbol(), clue_5.fg), ("5", theme.conflict));
        assert_eq!((clue_3.symbol(), clue_3.fg), ("3", theme.clue));
        assert_eq!((typed_5.symbol(), typed_5.fg), ("5", theme.conflict));

        game.press(4);
        let backend = render(&game, &View::default());
        assert_eq!(backend.buffer()[middle(2)].fg, theme.digit);
        assert_eq!(backend.buffer()[middle(0)].fg, theme.clue);
    }

    /// With NO_COLOR nothing on screen sets a colour, in the plainest game
    /// and in one with every highlight, hint and mark on.
    #[test]
    fn the_monochrome_theme_sets_no_colour_anywhere() {
        let mut game = game();
        game.fill_candidates();
        go(&mut game, 2);
        game.press(5);
        let view = View {
            checked: game.wrong_digits(),
            filter: Some(Filter::Digit(5)),
            ..open_hint(pointing_hint(), 5)
        };
        let backend = render_in(&game, &view, &Theme::monochrome());
        for cell in backend.buffer().content() {
            assert_eq!((cell.fg, cell.bg), (Color::Reset, Color::Reset), "{cell:?}");
        }
    }

    /// Without colour the cells of a hint's pattern are drawn reversed.
    #[test]
    fn the_monochrome_theme_reverses_a_hints_cells() {
        let game = game();
        let view = open_hint(pointing_hint(), 4);
        let backend = render_in(&game, &view, &Theme::monochrome());
        let reversed = |cell| {
            backend.buffer()[middle(cell)]
                .modifier
                .contains(Modifier::REVERSED)
        };
        assert!(reversed(79));
        assert!(!reversed(70));
        let coloured = render(&game, &view);
        let modifier = coloured.buffer()[middle(79)].modifier;
        assert!(
            !modifier.contains(Modifier::REVERSED),
            "colour has its own cue"
        );
    }

    /// A candidate's spot on the screen, relative to its cell's middle.
    fn note_at(cell: Cell, digit: u8) -> (u16, u16) {
        let (x, y) = middle(cell);
        let spot = u16::from(digit - 1);
        (x - 2 + 2 * (spot % 3), y - 1 + spot / 3)
    }

    /// Colour must not be the only way to tell things apart: a clue is bold
    /// and a digit the player typed is not.
    #[test]
    fn clues_are_bold_and_typed_digits_are_not() {
        let mut game = game();
        go(&mut game, 2);
        game.press(4);
        let backend = render(&game, &View::default());
        let buffer = backend.buffer();
        assert!(buffer[middle(0)].modifier.contains(Modifier::BOLD));
        assert!(!buffer[middle(2)].modifier.contains(Modifier::BOLD));
    }

    /// A clash is shown by reversing the digit, whatever the colours are.
    #[test]
    fn a_clash_reverses_the_digit_and_the_note() {
        let mut game = game();
        go(&mut game, 2);
        game.press(5); // clashes with the clue 5 at r1c1
        game.next_mode();
        go(&mut game, 3);
        game.press(5); // a candidate 5 in a row that holds a 5
        game.press(2);
        let backend = render(&game, &View::default());
        let buffer = backend.buffer();
        let reversed = |pos| buffer[pos].modifier.contains(Modifier::REVERSED);
        assert!(reversed(middle(2)));
        assert!(reversed(middle(0)));
        assert!(reversed(note_at(3, 5)));
        assert!(!reversed(note_at(3, 2)));
        assert!(!reversed(middle(1)));
    }

    /// A box mark and a candidate differ by more than hue.
    #[test]
    fn a_box_mark_is_underlined_and_a_candidate_is_not() {
        let mut game = game();
        game.next_mode();
        go(&mut game, 3);
        game.press(7);
        game.next_mode();
        game.press(8);
        let backend = render(&game, &View::default());
        let buffer = backend.buffer();
        let underlined = |digit| {
            buffer[note_at(3, digit)]
                .modifier
                .contains(Modifier::UNDERLINED)
        };
        assert!(underlined(8));
        assert!(!underlined(7));
    }

    /// The board's top line carries the column numbers and its left line the
    /// row numbers, so a hint can say r2c3 and be found. The cursor's own
    /// row and column are in the accent colour.
    #[test]
    fn the_board_edges_number_rows_and_columns() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 2 * 9 + 5); // r3c6
        let backend = render(&game, &View::default());
        let b = backend.buffer();
        for i in 0..9u16 {
            let (top, left) = (&b[(5 + 8 * i, 1)], &b[(1, 3 + 4 * i)]);
            let expected = char::from(b'1' + i as u8).to_string();
            assert_eq!((top.symbol(), left.symbol()), (&*expected, &*expected));
            assert_eq!(top.fg, if i == 5 { theme.accent } else { theme.dimmed });
            assert_eq!(left.fg, if i == 2 { theme.accent } else { theme.dimmed });
        }
        // The line between the numbers is still there.
        assert_eq!(b[(6, 1)].symbol(), "━");
        assert_eq!(b[(1, 5)].symbol(), "┠");
    }

    /// A pointing pair in box 9 along row 9: r9c8 and r9c9 hold the digit 3
    /// in the box, so r9c2 loses its 3.
    fn pointing_hint() -> Hint {
        use sumi_core::grid::House;
        use sumi_core::hint::StepHint;
        let three: sumi_core::digits::Digits = [3].into_iter().collect();
        Hint::Step(Box::new(StepHint {
            step: sumi_core::step::Step {
                technique: &sumi_core::technique::Pointing,
                digits: three,
                houses: vec![House::Box(8), House::Row(8)],
                cells: vec![79, 80],
                removals: vec![(73, three)],
                placement: None,
                key_cells: vec![],
            },
            level: sumi_core::technique::Level::Medium,
            count: 2,
        }))
    }

    fn open_hint(hint: Hint, rung: usize) -> View {
        View {
            hint: Some(OpenHint {
                hint,
                rung,
                revision: 0,
            }),
            ..View::default()
        }
    }

    fn background_at(game: &Game, view: &View, cell: Cell) -> Color {
        render(game, view).buffer()[middle(cell)].bg
    }

    #[test]
    fn a_hint_shades_its_area_then_marks_its_cells() {
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let at = |rung, cell| background_at(&game, &open_hint(pointing_hint(), rung), cell);
        // r8c8 is in box 9 only: not the pattern, and not a peer of the cursor.
        assert_ne!(at(2, 70), theme.hint_area);
        assert_eq!(at(3, 70), theme.hint_area);
        assert_eq!(at(3, 79), theme.hint_area);
        assert_eq!(at(4, 79), theme.hint_cell);
        assert_eq!(at(4, 70), theme.hint_area);
        // r9c1 is in row 9 and a peer of the cursor: the hint wins.
        assert_eq!(at(3, 72), theme.hint_area);
    }

    /// Colouring's second group is shaded as the area is, and its first group
    /// as a hint's cells are.
    #[test]
    fn a_colourings_second_group_is_drawn_fainter_than_its_first() {
        use sumi_core::hint::StepHint;
        let one: sumi_core::digits::Digits = [1].into_iter().collect();
        let hint = Hint::Step(Box::new(StepHint {
            step: sumi_core::step::Step {
                technique: &sumi_core::technique::SimpleColouring,
                digits: one,
                houses: vec![],
                cells: vec![40, 41, 49],
                removals: vec![(80, one)],
                placement: None,
                key_cells: vec![40],
            },
            level: sumi_core::technique::Level::Expert,
            count: 1,
        }));
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let at = |cell| background_at(&game, &open_hint(hint.clone(), 4), cell);
        assert_eq!(at(40), theme.hint_cell);
        assert_eq!(at(41), theme.hint_area);
        assert_eq!(at(49), theme.hint_area);
    }

    #[test]
    fn a_hint_cell_beats_the_cursor() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 79);
        let view = open_hint(pointing_hint(), 4);
        assert_eq!(background_at(&game, &view, 79), theme.hint_cell);
        // The corner marks still show where the cursor is.
        let (x, y) = middle(79);
        assert_eq!(render(&game, &view).buffer()[(x - 3, y - 1)].symbol(), "┌");
    }

    #[test]
    fn the_last_rung_crosses_out_the_candidates_it_removes() {
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let (x, y) = middle(73);
        let spot = |backend: &TestBackend| backend.buffer()[(x + 2, y - 1)].clone();
        let before = spot(&render(&game, &open_hint(pointing_hint(), 4)));
        assert_eq!(before.symbol(), " ", "3 is not drawn before the last rung");
        let after = spot(&render(&game, &open_hint(pointing_hint(), 5)));
        assert_eq!(after.symbol(), "3");
        assert_eq!(after.fg, theme.hint_removal);
        assert!(after.modifier.contains(Modifier::CROSSED_OUT));
    }

    #[test]
    fn the_last_rung_writes_the_digit_it_places() {
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let hint = game.hint();
        let (cell, digit) = hint.step().unwrap().placement.unwrap();
        let centre =
            |rung| render(&game, &open_hint(hint.clone(), rung)).buffer()[middle(cell)].clone();
        assert_eq!(centre(4).symbol(), " ");
        let placed = centre(5);
        assert_eq!(placed.symbol(), digit.to_string());
        assert_eq!(placed.fg, theme.hint_placement);
        assert!(placed.modifier.contains(Modifier::BOLD));
    }

    /// The placed digit takes the cell over: the other candidates written in
    /// it do not show around the digit.
    #[test]
    fn a_placed_digit_replaces_the_cells_candidates() {
        let mut game = game();
        game.fill_candidates();
        let hint = game.hint();
        let (cell, digit) = hint.step().unwrap().placement.unwrap();
        let other = (1..=9).find(|&d| d != digit && d != 5 && game.candidates(cell).contains(d));
        let other = other.expect("the cell lists a second candidate");
        let (x, y) = middle(cell);
        let spot = |backend: &TestBackend| {
            let n = u16::from(other - 1);
            backend.buffer()[(x - 2 + 2 * (n % 3), y - 1 + n / 3)]
                .symbol()
                .to_string()
        };
        assert_eq!(
            spot(&render(&game, &open_hint(hint.clone(), 4))),
            other.to_string()
        );
        assert_eq!(spot(&render(&game, &open_hint(hint, 5))), " ");
    }

    #[test]
    fn a_mistake_shades_its_box_then_marks_its_cell() {
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let at = |rung, cell| background_at(&game, &open_hint(Hint::Mistake(20), rung), cell);
        assert_ne!(at(1, 19), theme.hint_area);
        assert_eq!(at(2, 19), theme.hint_area);
        assert_eq!(at(2, 20), theme.hint_area);
        assert_eq!(at(3, 20), theme.hint_cell);
    }

    #[test]
    fn checked_digits_are_marked_while_they_stay_wrong() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 2);
        game.press(1); // the answer is 4
        let view = View {
            checked: game.wrong_digits(),
            ..View::default()
        };
        let marked = render(&game, &view).buffer()[middle(2)].clone();
        assert_eq!(marked.fg, theme.wrong_digit);
        assert!(marked.modifier.contains(Modifier::UNDERLINED));

        game.press(2); // another wrong digit: not what was checked
        let again = render(&game, &view).buffer()[middle(2)].clone();
        assert_eq!(again.fg, theme.digit);
        game.press(4);
        assert_eq!(render(&game, &view).buffer()[middle(2)].fg, theme.digit);
    }

    /// The screen as text, then what the colours say about each board cell:
    /// `!` a placed digit, `x` a removed candidate, `w` a wrong digit, `#` a
    /// hint's cell, `+` its area, `.` nothing. Snapshots of text alone would
    /// not show any of that.
    fn described(backend: &TestBackend) -> String {
        let theme = Theme::tokyo_night_amoled();
        let buffer = backend.buffer();
        let mut map = String::new();
        for r in 0..9 {
            for c in 0..9 {
                let cell = r * 9 + c;
                let (x, y) = middle(cell);
                let fg = |dx: i16, dy: i16, colour| {
                    buffer[((x as i16 + dx) as u16, (y as i16 + dy) as u16)].fg == colour
                };
                let any = |colour| (-2..=2).any(|dx| (-1..=1).any(|dy| fg(dx, dy, colour)));
                map.push(if any(theme.hint_placement) {
                    '!'
                } else if any(theme.hint_removal) {
                    'x'
                } else if any(theme.wrong_digit) {
                    'w'
                } else if buffer[(x, y)].bg == theme.hint_cell {
                    '#'
                } else if buffer[(x, y)].bg == theme.hint_area {
                    '+'
                } else {
                    '.'
                });
            }
            map.push('\n');
        }
        format!("{backend}\n{map}")
    }

    #[test]
    fn each_rung_of_a_placing_hint() {
        let game = game();
        let hint = game.hint();
        for rung in 1..=5 {
            insta::assert_snapshot!(
                format!("placing_rung_{rung}"),
                described(&render(&game, &open_hint(hint.clone(), rung)))
            );
        }
    }

    #[test]
    fn the_last_rung_of_a_removing_hint() {
        let game = game();
        insta::assert_snapshot!(described(&render(&game, &open_hint(pointing_hint(), 5))));
    }

    #[test]
    fn the_cell_rung_of_a_mistake() {
        let mut game = game();
        go(&mut game, 2);
        game.press(1);
        let hint = game.hint();
        assert_eq!(hint, Hint::Mistake(2));
        insta::assert_snapshot!(described(&render(&game, &open_hint(hint, 3))));
    }

    #[test]
    fn the_board_after_a_check() {
        let mut game = game();
        go(&mut game, 2);
        game.press(1);
        let view = View {
            checked: game.wrong_digits(),
            ..View::default()
        };
        insta::assert_snapshot!(described(&render(&game, &view)));
    }

    #[test]
    fn the_cursor_has_corner_marks() {
        let mut game = game();
        go(&mut game, 2);
        let backend = render(&game, &View::default());
        let (x, y) = middle(2);
        let b = backend.buffer();
        let corners = [
            b[(x - 3, y - 1)].symbol(),
            b[(x + 3, y - 1)].symbol(),
            b[(x - 3, y + 1)].symbol(),
            b[(x + 3, y + 1)].symbol(),
        ];
        assert_eq!(corners, ["┌", "┐", "└", "┘"]);
        let (x, y) = middle(3);
        assert_eq!(b[(x - 3, y - 1)].symbol(), " ");
    }

    #[test]
    fn notes_sit_in_fixed_spots_and_can_be_hidden() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 2); // r1c3: possible 1, 2, 4
        game.next_mode();
        for d in [1, 2, 4, 5] {
            game.press(d);
        }
        game.next_mode();
        game.press(2);

        let backend = render(&game, &View::default());
        let buffer = backend.buffer();
        let (x, y) = middle(2);
        let at = |dx: i16, dy: i16| {
            let cell = &buffer[((x as i16 + dx) as u16, (y as i16 + dy) as u16)];
            (cell.symbol().to_string(), cell.fg)
        };
        // 1 top left, 2 top middle, 4 middle left, 5 in the centre.
        assert_eq!(at(-2, -1), ("1".into(), theme.candidate));
        assert_eq!(at(0, -1), ("2".into(), theme.box_mark));
        assert_eq!(at(-2, 0).0, "4");
        // 5 is already in row 1, so that candidate shows as a clash.
        assert_eq!(at(0, 0), ("5".into(), theme.conflict));

        let hidden = View {
            notes_hidden: true,
            ..View::default()
        };
        let backend = render(&game, &hidden);
        assert_eq!(backend.buffer()[(x - 2, y - 1)].symbol(), " ");
    }

    /// Each check puts two highlights on the same cell, so it fails if
    /// their order in `background` is swapped.
    #[test]
    fn the_cursor_beats_the_filter_and_both_highlights_beat_peer_shading() {
        let theme = Theme::tokyo_night_amoled();
        let filter_4 = View {
            filter: Some(Filter::Digit(4)),
            ..View::default()
        };
        let bg = |game: &Game, view: &View, cell| render(game, view).buffer()[middle(cell)].bg;

        let mut game = game();
        go(&mut game, 2); // r1c3, where 4 is possible
        assert_eq!(bg(&game, &filter_4, 2), theme.cursor);
        // r1c6 could hold 4 and shares row 1 with the cursor.
        assert_eq!(bg(&game, &filter_4, 5), theme.filter);

        game.press(5); // r1c3 now holds a 5, in the same row as the clue 5 at r1c1
        go(&mut game, 0);
        let plain = View::default();
        assert_eq!(bg(&game, &plain, 2), theme.same_digit); // a peer holding 5
        assert_eq!(bg(&game, &plain, 14), theme.same_digit); // r2c6, not a peer
        assert_eq!(bg(&game, &plain, 36), theme.peer); // r5c1 holds 4
    }

    #[test]
    fn the_pairs_filter_lights_cells_with_two_candidates() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        go(&mut game, 2); // r1c3: possible 1, 2, 4
        game.next_mode();
        game.press(1);
        game.press(4);
        go(&mut game, 80); // move away so the cursor doesn't cover it
        let pairs = View {
            filter: Some(Filter::TwoCandidates),
            ..View::default()
        };
        let backend = render(&game, &pairs);
        assert_eq!(backend.buffer()[middle(2)].bg, theme.filter);

        go(&mut game, 2);
        game.press(2); // three candidates now
        go(&mut game, 80);
        let backend = render(&game, &pairs);
        assert_ne!(backend.buffer()[middle(2)].bg, theme.filter);
    }

    #[test]
    fn a_complete_digit_is_dimmed_but_not_on_a_solved_board() {
        let theme = Theme::tokyo_night_amoled();
        let mut game = game();
        let solution = solution();
        for cell in 0..81 {
            if solution.get(cell) == Some(5) && !game.is_clue(cell) {
                go(&mut game, cell);
                game.press(5);
            }
        }
        let backend = render(&game, &View::default());
        assert_eq!(backend.buffer()[middle(0)].fg, theme.dimmed);

        for cell in 0..81 {
            if !game.is_clue(cell) && game.board().get(cell).is_none() {
                go(&mut game, cell);
                game.press(solution.get(cell).unwrap());
            }
        }
        assert!(game.is_won());
        let backend = render(&game, &View::default());
        assert_eq!(backend.buffer()[middle(0)].fg, theme.clue);
    }

    /// Every optional part at once, in the mode with the longest name: the
    /// title stays, the mode and the hint stay, and what does not fit goes.
    #[test]
    fn the_status_line_gives_up_its_least_useful_parts_before_the_title() {
        let mut game = game();
        game.next_mode();
        game.next_mode();
        let view = View {
            notes_hidden: true,
            filter: Some(Filter::TwoCandidates),
            notice: Some("no mistakes"),
            ..open(&game, 5)
        };
        let backend = render(&game, &view);
        let top: String = (0..FRAME_WIDTH)
            .map(|x| backend.buffer()[(x, 0)].symbol())
            .collect();
        assert!(top.starts_with("╭ sumi "), "{top}");
        assert!(top.ends_with("● BOX MARK ╮"), "{top}");
        assert!(top.contains("hint 5/5"), "{top}");
        assert!(top.contains("no mistakes"), "{top}");
        assert!(!top.contains("hidden"), "{top}");
    }

    fn open(game: &Game, rung: usize) -> View {
        open_hint(game.hint(), rung)
    }

    /// The panel's rows as text, with the trailing spaces gone.
    fn panel_rows(backend: &TestBackend) -> Vec<String> {
        let buffer = backend.buffer();
        let top = BOARD_HEIGHT + 2;
        (top + 1..top + PANEL_HEIGHT - 1)
            .map(|y| {
                (2..FRAME_WIDTH - 2)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn the_panel_says_how_to_ask_and_then_shows_the_rung() {
        let game = game();
        let closed = panel_rows(&render(&game, &View::default()));
        assert_eq!(closed[0], "? for a hint");

        let lines = panel_rows(&render(&game, &open(&game, 2)));
        assert_eq!(lines[0], "Rung 2 of 5");
        assert!(lines[1].starts_with("Look for"), "{lines:?}");
    }

    /// Every rung of every hint met while following hints through Hard and
    /// Expert bank puzzles, and the mistake ladders, wrap within the panel
    /// as ratatui wraps them. `PANEL_HEIGHT` holds two lines of text.
    #[test]
    fn every_rung_fits_the_panel() {
        use rand::SeedableRng;
        use sumi_core::technique::Level;
        let theme = Theme::tokyo_night_amoled();
        let area = Rect::new(0, 0, FRAME_WIDTH - 4, 10);
        let rows_used = |hint: &Hint, rung: usize| {
            let view = open_hint(hint.clone(), rung);
            let mut buffer = Buffer::empty(area);
            panel_text(&view, &theme).render(area, &mut buffer);
            (0..area.height)
                .filter(|&y| (0..area.width).any(|x| buffer[(x, y)].symbol() != " "))
                .count()
        };
        let mut hints = vec![Hint::Mistake(80), Hint::StruckOut(80), Hint::Stuck];
        let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(5);
        for level in [Level::Hard, Level::Expert] {
            for _ in 0..8 {
                let mut game = Game::new(sumi_core::bank::pick(&mut rng, level).unwrap());
                while let Some(step) = game.hint().step().cloned() {
                    hints.push(game.hint());
                    game.apply_hint(&step);
                }
            }
        }
        let mut longest = 0;
        for hint in &hints {
            for rung in 1..=hint.rungs() {
                let used = rows_used(hint, rung);
                longest = longest.max(used);
                assert!(
                    used <= usize::from(PANEL_HEIGHT - 2),
                    "{}",
                    hint.rung(rung).text
                );
            }
        }
        assert_eq!(longest, 3, "some rung uses both lines of text");
    }

    #[test]
    fn the_status_line_names_the_rung() {
        let game = game();
        let backend = render(&game, &open(&game, 3));
        let top: String = (0..FRAME_WIDTH)
            .map(|x| backend.buffer()[(x, 0)].symbol())
            .collect();
        assert!(top.contains("hint 3/5"), "{top}");
    }

    #[test]
    fn every_key_bar_fits_under_the_board() {
        let mut game = game();
        let views = [
            View::default(),
            View {
                awaiting_filter: true,
                ..View::default()
            },
            open(&game, 1),
            open(&game, 5),
            open_hint(Hint::Mistake(2), 3),
        ];
        for view in views {
            for _ in 0..3 {
                let width = key_bar(&keys(&game, &view), &Theme::tokyo_night_amoled()).width();
                // The bottom border has BOARD_WIDTH columns between its
                // corners; keep one line character visible at each end.
                assert!(
                    width <= usize::from(BOARD_WIDTH) - 2,
                    "{:?}: {width}",
                    game.mode()
                );
                game.next_mode();
            }
        }
    }

    #[test]
    fn too_small_window() {
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
        terminal
            .draw(|frame| {
                draw(
                    frame,
                    &game(),
                    &View::default(),
                    &Theme::tokyo_night_amoled(),
                )
            })
            .unwrap();
        insta::assert_snapshot!(terminal.backend());
    }
}
