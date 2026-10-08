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

/// A cell is 3 rows tall in both layouts, with a one-character line between
/// cells. The full layout's cells are 7 columns wide.
const CELL_WIDTH: u16 = 7;
const CELL_HEIGHT: u16 = 3;
const BOARD_WIDTH: u16 = Density::Full.board_width();
const BOARD_HEIGHT: u16 = 9 * (CELL_HEIGHT + 1) + 1;

/// The panel under the full board is this tall, borders included: the
/// rung's number and two lines of text, which every hint rung fits in at
/// the board's width.
const PANEL_HEIGHT: u16 = 5;

/// The dense layout's hint area has no border: the rung's number and four
/// lines of text, which every rung fits in at 30 columns.
const DENSE_PANEL_HEIGHT: u16 = 5;

/// The dense key bar wraps to this many rows.
const DENSE_KEY_ROWS: u16 = 2;

/// " sumi ", as the frame's title and the dense layout's first line.
const TITLE: &str = " sumi ";

/// The most text the status line can have beside the frame's title: the
/// board's width less the title, a border character, and the line's padding.
const STATUS_WIDTH: usize = BOARD_WIDTH as usize - TITLE.len() - 1 - 2;

/// How big the board is drawn, chosen from the window's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    /// Cells 7 wide with the candidates spaced out, inside the frame.
    Full,
    /// Cells 3 wide with the candidates packed, and no frame: the board's
    /// outline is the frame. For phones and narrow windows.
    Dense,
}

impl Density {
    const fn cell_width(self) -> u16 {
        match self {
            Density::Full => CELL_WIDTH,
            Density::Dense => 3,
        }
    }

    const fn board_width(self) -> u16 {
        9 * (self.cell_width() + 1) + 1
    }

    /// Columns left of the board for the row numbers. The full layout writes
    /// them on the board's left line, beside a column notes never use; a
    /// dense cell's candidates start at its edge, so a number there would
    /// read as part of them.
    const fn gutter(self) -> u16 {
        match self {
            Density::Full => 0,
            Density::Dense => 1,
        }
    }

    /// The smallest window the layout fits in.
    const fn needed(self) -> (u16, u16) {
        match self {
            Density::Full => (FRAME_WIDTH, BOARD_HEIGHT + 2 + PANEL_HEIGHT),
            // The status line, the board, the hint and the key bar.
            Density::Dense => (
                Density::Dense.gutter() + Density::Dense.board_width(),
                1 + BOARD_HEIGHT + DENSE_PANEL_HEIGHT + DENSE_KEY_ROWS,
            ),
        }
    }
}

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
    /// The list of every key is open over the game.
    pub help: bool,
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

/// The full layout's board inside its frame and the panel under it.
const NEEDED: (u16, u16) = Density::Full.needed();

/// The layout a window of this size gets, the full one if it fits, or none
/// if even the dense one doesn't.
pub fn density(area: Rect) -> Option<Density> {
    let fits = |d: Density| area.width >= d.needed().0 && area.height >= d.needed().1;
    [Density::Full, Density::Dense]
        .into_iter()
        .find(|&d| fits(d))
}

/// Whether `draw` shows the board in a window of this size. The app asks
/// this too, so keys never act on a board the player can't see.
pub fn fits(area: Rect) -> bool {
    density(area).is_some()
}

pub fn draw(frame: &mut Frame, game: &Game, view: &View, theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);
    let Some(density) = density(area) else {
        too_small(frame, theme);
        return;
    };
    let rung = view.hint.as_ref().map(|open| open.hint.rung(open.rung));
    let board = Board {
        game,
        view,
        theme,
        rung,
        density,
    };
    if density == Density::Dense {
        draw_dense(frame, board);
        return;
    }

    let whole = area.centered(Constraint::Length(NEEDED.0), Constraint::Length(NEEDED.1));
    let [outer, side] = Layout::vertical([
        Constraint::Length(BOARD_HEIGHT + 2),
        Constraint::Length(PANEL_HEIGHT),
    ])
    .areas(whole);
    let block = frame_block(theme)
        .title_top(status(game, view, theme, STATUS_WIDTH).right_aligned())
        .title_bottom(key_bar(&keys(game, view), theme).centered());
    let inner = block.inner(outer);
    frame.render_widget(block, outer);
    if view.help {
        let [list] = Layout::horizontal([Constraint::Fill(1)])
            .horizontal_margin(2)
            .vertical_margin(1)
            .areas(inner);
        frame.render_widget(key_list(theme), list);
    } else {
        frame.render_widget(board, inner);
    }
    let block = panel_block(theme, "Hint");
    let inner = block.inner(side);
    frame.render_widget(block, side);
    frame.render_widget(hint_text(view, theme), inner);
}

/// The dense layout, top to bottom: the title and status line, the board,
/// the hint and the key bar.
fn draw_dense(frame: &mut Frame, board: Board) {
    let (game, view, theme) = (board.game, board.view, board.theme);
    let needed = Density::Dense.needed();
    let whole = frame
        .area()
        .centered(Constraint::Length(needed.0), Constraint::Length(needed.1));
    let [top, middle, hint, keys_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(BOARD_HEIGHT),
        Constraint::Length(DENSE_PANEL_HEIGHT),
        Constraint::Length(DENSE_KEY_ROWS),
    ])
    .areas(whole);
    let width = usize::from(needed.0) - TITLE.len() - 2;
    let [title_area, status_area] =
        Layout::horizontal([Constraint::Length(TITLE.len() as u16), Constraint::Fill(1)])
            .areas(top);
    frame.render_widget(title(theme), title_area);
    frame.render_widget(
        status(game, view, theme, width).right_aligned(),
        status_area,
    );
    if view.help {
        let [list] = Layout::horizontal([Constraint::Fill(1)])
            .horizontal_margin(1)
            .areas(middle);
        frame.render_widget(key_list(theme), list);
    } else {
        frame.render_widget(board, middle);
    }
    frame.render_widget(hint_text(view, theme), hint);
    let lines = key_bar_lines(&keys(game, view), theme, usize::from(needed.0));
    frame.render_widget(Paragraph::new(lines).centered(), keys_area);
}

/// The hint's words under its rung's number, or how to ask for one.
fn hint_text(view: &View, theme: &Theme) -> Paragraph<'static> {
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

/// The status line, at most `width` characters between its padding.
fn status(game: &Game, view: &View, theme: &Theme, width: usize) -> Line<'static> {
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
        shown.join(" · ").chars().count() <= width
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
/// Each list fits one line of the full layout; `i` lists the rest.
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
    let mut keys: Vec<(Action, &str)> = if view.help {
        vec![(Action::Close, "close")]
    } else if let Some(open) = &view.hint {
        let mut keys = Vec::new();
        if open.rung < open.hint.rungs() {
            keys.push((Action::Hint, "next rung"));
        } else if open.hint.step().is_some() {
            keys.push((Action::Confirm, "apply"));
        } else if let Hint::Mistake(_) | Hint::StruckOut(_) = open.hint {
            keys.push((Action::Confirm, "go to cell"));
        }
        keys.push((Action::Step(crate::game::Direction::Up), "look around"));
        keys.push((Action::Close, "close"));
        keys
    } else if game.is_won() {
        vec![(Action::Confirm, "new game")]
    } else {
        match game.mode() {
            Mode::Digit if !view.checked.is_empty() => vec![
                (Action::Digit(1), "place"),
                (Action::NextMode, "mode"),
                (Action::Hint, "hint"),
                (Action::Undo, "undo"),
                (Action::Close, "clear marks"),
            ],
            Mode::Digit => vec![
                (Action::Digit(1), "place"),
                (Action::Clear, "clear"),
                (Action::NextMode, "mode"),
                (Action::Filter, "filter"),
                (Action::Hint, "hint"),
                (Action::Undo, "undo"),
            ],
            Mode::Candidate => vec![
                (Action::Digit(1), "toggle"),
                (Action::ClearLayer, "clear candidates"),
                (Action::FillCandidates, "fill"),
                (Action::NextMode, "mode"),
            ],
            Mode::BoxMark => vec![
                (Action::Digit(1), "toggle"),
                (Action::MarksToCandidates, "fill, drop marks"),
                (Action::NextMode, "mode"),
            ],
        }
    };
    if !view.help {
        keys.push((Action::Help, "keys"));
    }
    keys.push((Action::Quit, "quit"));
    keys.into_iter()
        .map(|(action, words)| (key_label(action), words))
        .collect()
}

/// The rounded frame every screen sits in, titled with the program's name.
pub fn frame_block(theme: &Theme) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.frame))
        .title_top(title(theme))
}

fn title(theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(
        TITLE,
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    ))
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

/// `keys` in as many rows as it takes to keep each at most `width` columns
/// wide, no key split from its words.
pub fn key_bar_lines(keys: &[(&str, &str)], theme: &Theme, width: usize) -> Vec<Line<'static>> {
    let mut rows: Vec<Vec<(&str, &str)>> = vec![Vec::new()];
    for &entry in keys {
        let row = rows.last_mut().unwrap();
        row.push(entry);
        if row.len() > 1 && key_bar(row, theme).width() > width {
            row.pop();
            rows.push(vec![entry]);
        }
    }
    rows.iter().map(|row| key_bar(row, theme)).collect()
}

/// What the key list says an action does, in at most 28 characters so a
/// line fits the dense board's width.
fn help_words(action: Action) -> &'static str {
    match action {
        Action::Step(_) => "move (arrows too)",
        Action::Jump(_) => "next box",
        Action::Digit(_) => "place, or toggle a note",
        Action::Clear => "clear the cell",
        Action::NextMode => "mode: digit, candidate, mark",
        Action::FillCandidates => "fill empty cells' candidates",
        Action::RefillCandidates => "refill every empty cell",
        Action::MarksToCandidates => "fill, drop box marks",
        Action::ClearLayer => "clear this mode's notes",
        Action::ClearAllNotes => "clear all notes",
        Action::ToggleNotes => "hide or show notes",
        Action::Filter => "filter: 1-9, b pairs, 0 off",
        Action::Hint => "hint, then its next rung",
        Action::Confirm => "apply hint, go to mistake",
        Action::Check => "check for mistakes",
        Action::Reveal => "reveal this cell",
        Action::Close => "close, or clear marks",
        Action::Undo => "undo",
        Action::Redo => "redo",
        Action::Help => "this list",
        Action::Quit => "quit",
    }
}

/// The heading the key list puts an action under. `Action::ALL` keeps each
/// group together.
fn help_group(action: Action) -> &'static str {
    match action {
        Action::Step(_) | Action::Jump(_) => "Move",
        Action::Digit(_) | Action::Clear | Action::NextMode => "Write",
        Action::FillCandidates
        | Action::RefillCandidates
        | Action::MarksToCandidates
        | Action::ClearLayer
        | Action::ClearAllNotes
        | Action::ToggleNotes
        | Action::Filter => "Notes",
        Action::Hint | Action::Confirm | Action::Check | Action::Reveal | Action::Close => {
            "Hints and checks"
        }
        Action::Undo | Action::Redo | Action::Help | Action::Quit => "Game",
    }
}

/// Every action and its key, under headings, drawn where the board goes.
fn key_list(theme: &Theme) -> Paragraph<'static> {
    let mut lines = Vec::new();
    let mut group = "";
    for action in Action::ALL {
        if help_group(action) != group {
            group = help_group(action);
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::from(Span::styled(
                group,
                Style::new().fg(theme.text).add_modifier(Modifier::BOLD),
            )));
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>5}", key_label(action)),
                Style::new().fg(theme.accent),
            ),
            Span::styled(
                format!("  {}", help_words(action)),
                Style::new().fg(theme.text),
            ),
        ]));
    }
    Paragraph::new(lines)
}

/// The message wraps, so a narrow window still shows as much of it as fits.
fn too_small(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let smallest = Density::Dense.needed();
    let message = format!(
        "Make the window at least {}×{}. It is {}×{} now.",
        smallest.0, smallest.1, area.width, area.height
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
    density: Density,
}

impl Widget for Board<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let gutter = self.density.gutter();
        let whole = area.centered(
            Constraint::Length(gutter + self.density.board_width()),
            Constraint::Length(BOARD_HEIGHT),
        );
        let area = Rect {
            x: whole.x + gutter,
            width: whole.width - gutter,
            ..whole
        };
        self.render_lines(area, buf);
        let conflicts = self.game.board().conflicts();
        for (cell, &conflict) in conflicts.iter().enumerate() {
            self.render_cell(cell, conflict, area, buf);
        }
        if self.density == Density::Dense {
            self.render_cursor_frame(area, buf);
        }
        self.render_numbers(area, buf);
    }
}

impl Board<'_> {
    fn render_lines(&self, area: Rect, buf: &mut Buffer) {
        let cell_width = self.density.cell_width();
        for dy in 0..BOARD_HEIGHT {
            for dx in 0..self.density.board_width() {
                let on_vertical = dx % (cell_width + 1) == 0;
                let on_horizontal = dy % (CELL_HEIGHT + 1) == 0;
                let kx = dx / (cell_width + 1);
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
    /// into its left line, or the gutter left of it (`Density::gutter`), in
    /// the middle of the cell they belong to. The cursor's row and column
    /// stand out.
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
        let cell_width = self.density.cell_width();
        for i in 0..9 {
            let number = (i + 1).to_string();
            let offset = 1 + i as u16 * (cell_width + 1);
            buf.set_string(
                area.x + offset + cell_width / 2,
                area.y,
                &number,
                style(col(cursor) == i),
            );
            let offset = 1 + i as u16 * (CELL_HEIGHT + 1);
            buf.set_string(
                area.x - self.density.gutter(),
                area.y + offset + CELL_HEIGHT / 2,
                &number,
                style(row(cursor) == i),
            );
        }
    }

    fn render_cell(&self, cell: Cell, conflict: bool, area: Rect, buf: &mut Buffer) {
        let rect = self.cell_rect(cell, area);
        let (x, y) = (rect.x, rect.y);
        if let Some(bg) = self.background(cell) {
            buf.set_style(rect, Style::new().bg(bg));
        }
        // Without colour, the two highlights whose cells nothing else names
        // are drawn reversed: a hint's pattern and the filter's cells.
        let hint_cell = self.rung.as_ref().is_some_and(|r| r.cells.contains(&cell));
        if self.theme.monochrome && (hint_cell || self.filtered(cell)) {
            buf.set_style(rect, Style::new().add_modifier(Modifier::REVERSED));
        }
        if cell == self.game.cursor() && self.density == Density::Full {
            // Corner marks show the cursor without relying on colour. Notes
            // never use a cell's first or last column, so they don't collide.
            let mark = Style::new().fg(self.theme.accent);
            let (right, bottom) = (x + rect.width - 1, y + CELL_HEIGHT - 1);
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
                    x + rect.width / 2,
                    y + CELL_HEIGHT / 2,
                    digit.to_string(),
                    style,
                );
            }
            None if !self.view.notes_hidden => self.render_notes(cell, x, y, buf),
            None => {}
        }
        self.render_hint_marks(cell, rect, buf);
    }

    /// Whether the candidate filter lights `cell`.
    fn filtered(&self, cell: Cell) -> bool {
        let candidates = self.game.working_candidates(cell);
        match self.view.filter {
            Some(Filter::Digit(d)) => candidates.contains(d),
            Some(Filter::TwoCandidates) => candidates.len() == 2,
            None => false,
        }
    }

    /// Where a cell's contents go, inside the lines around it.
    fn cell_rect(&self, cell: Cell, board: Rect) -> Rect {
        let width = self.density.cell_width();
        let x = board.x + 1 + col(cell) as u16 * (width + 1);
        let y = board.y + 1 + row(cell) as u16 * (CELL_HEIGHT + 1);
        Rect::new(x, y, width, CELL_HEIGHT)
    }

    /// Where a note for `digit` goes in a cell whose contents start at
    /// `(x, y)`: a fixed spot in a 3×3 grid, 1 top left and 9 bottom right,
    /// spaced out in the full layout and packed in the dense one.
    fn note_spot(&self, digit: u8, x: u16, y: u16) -> (u16, u16) {
        let spot = u16::from(digit - 1);
        match self.density {
            Density::Full => (x + 1 + 2 * (spot % 3), y + spot / 3),
            Density::Dense => (x + spot % 3, y + spot / 3),
        }
    }

    /// The dense layout's cells have no room for corner marks, so the lines
    /// around the cursor are drawn double instead: a shape no other line on
    /// the board has.
    fn render_cursor_frame(&self, board: Rect, buf: &mut Buffer) {
        let cell = self.cell_rect(self.game.cursor(), board);
        let style = Style::new().fg(self.theme.accent);
        let (left, right) = (cell.x - 1, cell.x + cell.width);
        let (top, bottom) = (cell.y - 1, cell.y + cell.height);
        for x in cell.x..right {
            buf.set_string(x, top, "═", style);
            buf.set_string(x, bottom, "═", style);
        }
        for y in cell.y..bottom {
            buf.set_string(left, y, "║", style);
            buf.set_string(right, y, "║", style);
        }
        for (x, y, corner) in [
            (left, top, "╔"),
            (right, top, "╗"),
            (left, bottom, "╚"),
            (right, bottom, "╝"),
        ] {
            buf.set_string(x, y, corner, style);
        }
    }

    /// What the open hint writes in a cell: the candidates it removes,
    /// crossed out, or the digit it places, which takes the cell's notes'
    /// place.
    fn render_hint_marks(&self, cell: Cell, rect: Rect, buf: &mut Buffer) {
        let Some(rung) = &self.rung else { return };
        let (x, y) = (rect.x, rect.y);
        for &(_, digits) in rung.removals.iter().filter(|&&(c, _)| c == cell) {
            for digit in digits.iter() {
                let (nx, ny) = self.note_spot(digit, x, y);
                buf.set_string(
                    nx,
                    ny,
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
            // The full layout's notes never use a cell's first and last
            // columns, where the cursor's corner marks go.
            let blank = match self.density {
                Density::Full => Rect::new(x + 1, y, rect.width - 2, CELL_HEIGHT),
                Density::Dense => rect,
            };
            for dy in 0..CELL_HEIGHT {
                buf.set_string(
                    blank.x,
                    y + dy,
                    " ".repeat(usize::from(blank.width)),
                    Style::new(),
                );
            }
            buf.set_string(
                x + rect.width / 2,
                y + CELL_HEIGHT / 2,
                digit.to_string(),
                Style::new()
                    .fg(self.theme.hint_placement)
                    .add_modifier(Modifier::BOLD),
            );
        }
    }

    /// Each note goes in its digit's spot (`note_spot`). A box mark takes
    /// the spot over from a candidate.
    fn render_notes(&self, cell: Cell, x: u16, y: u16, buf: &mut Buffer) {
        let candidates = self.game.candidates(cell);
        let marks = self.game.marks(cell);
        for digit in candidates.union(marks).iter() {
            let (nx, ny) = self.note_spot(digit, x, y);
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
            buf.set_string(nx, ny, digit.to_string(), style);
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
        let filtered = self.filtered(cell);
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
    /// Without colour the filter's cells are drawn reversed: with notes
    /// hidden nothing else says which cells it lights.
    #[test]
    fn the_monochrome_theme_reverses_the_filters_cells() {
        let game = game();
        let view = View {
            filter: Some(Filter::Digit(5)),
            notes_hidden: true,
            ..View::default()
        };
        let lit = |cell| game.working_candidates(cell).contains(5);
        let empty: Vec<Cell> = (0..81).filter(|&c| game.board().get(c).is_none()).collect();
        let on = *empty.iter().find(|&&c| lit(c)).unwrap();
        let off = *empty.iter().find(|&&c| !lit(c)).unwrap();
        let backend = render_in(&game, &view, &Theme::monochrome());
        let reversed = |cell| {
            backend.buffer()[middle(cell)]
                .modifier
                .contains(Modifier::REVERSED)
        };
        assert!(reversed(on), "r{}c{}", row(on) + 1, col(on) + 1);
        assert!(!reversed(off));
    }

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
        let rows_used = |hint: &Hint, rung: usize, width: u16| {
            let area = Rect::new(0, 0, width, 10);
            let view = open_hint(hint.clone(), rung);
            let mut buffer = Buffer::empty(area);
            hint_text(&view, &theme).render(area, &mut buffer);
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
        // The full panel inside its border and padding, and the dense hint
        // area, which has neither.
        let layouts = [
            (FRAME_WIDTH - 4, PANEL_HEIGHT - 2),
            (DENSE.0, DENSE_PANEL_HEIGHT),
        ];
        for (width, rows) in layouts {
            let mut longest = 0;
            for hint in &hints {
                for rung in 1..=hint.rungs() {
                    let used = rows_used(hint, rung, width);
                    longest = longest.max(used);
                    assert!(
                        used <= usize::from(rows),
                        "{width} columns: {}",
                        hint.rung(rung).text
                    );
                }
            }
            assert!(
                longest >= usize::from(rows) - 1,
                "{width} columns: {longest} rows at most"
            );
        }
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

    const DENSE: (u16, u16) = Density::Dense.needed();

    fn render_dense(game: &Game, view: &View) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(DENSE.0, DENSE.1)).unwrap();
        let theme = Theme::tokyo_night_amoled();
        terminal
            .draw(|frame| draw(frame, game, view, &theme))
            .unwrap();
        terminal.backend().clone()
    }

    /// The top left of a cell's contents in the dense layout, in a window of
    /// exactly `DENSE`: the status line takes the first row, and the row
    /// numbers' gutter and the board's outline take a column each.
    fn dense_cell(cell: Cell) -> (u16, u16) {
        (2 + 4 * col(cell) as u16, 2 + 4 * row(cell) as u16)
    }

    /// A row's number stands apart from the candidates in its first cell,
    /// with the board's line between them.
    #[test]
    fn dense_row_numbers_sit_outside_the_board() {
        let mut game = game();
        game.next_mode();
        go(&mut game, 18); // r3c1
        game.press(4);
        go(&mut game, 40);
        let backend = render_dense(&game, &View::default());
        let (x, y) = dense_cell(18);
        let middle_row: String = (0..x + 1)
            .map(|dx| backend.buffer()[(dx, y + 1)].symbol())
            .collect();
        assert_eq!(middle_row, "3┃4");
    }

    #[test]
    fn the_window_picks_the_layout() {
        let at = |width, height| density(Rect::new(0, 0, width, height));
        assert_eq!(at(75, 44), Some(Density::Full));
        assert_eq!(at(74, 45), Some(Density::Dense));
        assert_eq!(at(38, 45), Some(Density::Dense));
        assert_eq!(at(37, 45), None);
        assert_eq!(at(38, 44), None);
        assert_eq!(at(200, 43), None);
        assert_eq!(DENSE, (38, 45));
    }

    #[test]
    fn dense_layout_with_notes() {
        let mut game = game();
        game.fill_candidates();
        go(&mut game, 2);
        game.press(5); // r1c3 clashes with the 5 at r1c1
        game.next_mode();
        game.next_mode();
        go(&mut game, 3);
        game.press(6); // a box mark at r1c4
        go(&mut game, 40);
        insta::assert_snapshot!(render_dense(&game, &View::default()));
    }

    /// The dense layout packs a cell's candidates into its 3×3 characters.
    #[test]
    fn dense_candidates_fill_the_cell() {
        let mut game = game();
        game.next_mode();
        go(&mut game, 3);
        for digit in [2, 6, 8] {
            game.press(digit);
        }
        go(&mut game, 40);
        let backend = render_dense(&game, &View::default());
        let (x, y) = dense_cell(3);
        let row = |dy| {
            (0..3)
                .map(|dx| backend.buffer()[(x + dx, y + dy)].symbol())
                .collect::<String>()
        };
        assert_eq!([row(0), row(1), row(2)], [" 2 ", "  6", " 8 "]);
    }

    /// No corner marks fit in a dense cell; the lines around the cursor are
    /// drawn double, a shape no other line has.
    #[test]
    fn the_dense_cursor_is_framed_with_double_lines() {
        let mut game = game();
        go(&mut game, 40);
        let backend = render_dense(&game, &View::default());
        let b = backend.buffer();
        let (x, y) = dense_cell(40);
        assert_eq!(b[(x - 1, y - 1)].symbol(), "╔");
        assert_eq!(b[(x + 3, y - 1)].symbol(), "╗");
        assert_eq!(b[(x - 1, y + 3)].symbol(), "╚");
        assert_eq!(b[(x + 3, y + 3)].symbol(), "╝");
        assert_eq!(b[(x + 1, y - 1)].symbol(), "═");
        assert_eq!(b[(x - 1, y + 1)].symbol(), "║");
        let (ox, oy) = dense_cell(70); // r8c8: thin lines meet at its top left
        assert_eq!(b[(ox - 1, oy - 1)].symbol(), "┼", "only the cursor's lines");
        assert_eq!(b[(x, y)].symbol(), " ", "no corner marks inside");
    }

    /// Placing a digit blanks the whole dense cell: its notes fill it, and
    /// none of them may show beside the digit.
    #[test]
    fn a_dense_hints_placement_takes_the_whole_cell() {
        let mut game = game();
        game.fill_candidates();
        let hint = game.hint();
        let (cell, digit) = hint.step().unwrap().placement.unwrap();
        let backend = render_dense(&game, &open_hint(hint, 5));
        let (x, y) = dense_cell(cell);
        let text: String = (0..3)
            .flat_map(|dy| (0..3).map(move |dx| (x + dx, y + dy)))
            .map(|pos| backend.buffer()[pos].symbol().to_string())
            .collect();
        assert_eq!(text, format!("    {digit}    "));
    }

    #[test]
    fn the_dense_status_line_drops_parts_before_the_title() {
        let mut game = game();
        game.next_mode();
        game.next_mode();
        let view = View {
            notes_hidden: true,
            filter: Some(Filter::TwoCandidates),
            notice: Some("no mistakes"),
            ..open(&game, 5)
        };
        let backend = render_dense(&game, &view);
        let top: String = (0..DENSE.0)
            .map(|x| backend.buffer()[(x, 0)].symbol())
            .collect();
        assert!(top.starts_with(" sumi "), "{top}");
        assert!(top.ends_with("● BOX MARK "), "{top}");
        // Whole parts only: the hint, then the mode.
        let status = top[TITLE.len()..].trim();
        assert_eq!(status, "hint 5/5 · ● BOX MARK");
    }

    /// The status line above the dense board, and the rung's number at the
    /// top of the hint area.
    #[test]
    fn the_dense_status_line_and_hint_fit() {
        let game = game();
        let view = open(&game, 2);
        let backend = render_dense(&game, &view);
        let b = backend.buffer();
        let top: String = (0..DENSE.0).map(|x| b[(x, 0)].symbol()).collect();
        assert!(top.starts_with(" sumi "), "{top}");
        assert!(top.trim_end().ends_with("● DIGIT"), "{top}");
        let first = 1 + BOARD_HEIGHT;
        let rung: String = (0..DENSE.0).map(|x| b[(x, first)].symbol()).collect();
        assert_eq!(rung.trim_end(), "Rung 2 of 5");
    }

    fn won_game() -> Game {
        let solution = solution();
        let mut game = game();
        for cell in 0..81 {
            if !game.is_clue(cell) {
                go(&mut game, cell);
                game.press(solution.get(cell).unwrap());
            }
        }
        assert!(game.is_won());
        game
    }

    /// Every state the key bar has keys of its own for.
    fn contexts() -> Vec<(&'static str, Game, View)> {
        let mode = |presses| {
            let mut game = game();
            for _ in 0..presses {
                game.next_mode();
            }
            game
        };
        let wrong = || {
            let mut game = game();
            go(&mut game, 2);
            game.press(1); // the answer is 4
            game
        };
        let (plain, mistake) = (game(), wrong().hint());
        let checked = wrong().wrong_digits();
        vec![
            ("digit", game(), View::default()),
            ("candidate", mode(1), View::default()),
            ("box mark", mode(2), View::default()),
            (
                "filter prompt",
                game(),
                View {
                    awaiting_filter: true,
                    ..View::default()
                },
            ),
            ("hint", game(), open(&plain, 2)),
            ("last rung", game(), open(&plain, 5)),
            ("mistake", wrong(), open_hint(mistake, 3)),
            (
                "marks",
                wrong(),
                View {
                    checked,
                    ..View::default()
                },
            ),
            (
                "key list",
                game(),
                View {
                    help: true,
                    ..View::default()
                },
            ),
            ("won", won_game(), View::default()),
        ]
    }

    /// Every key bar but the filter's prompt, where any other key cancels,
    /// ends with quit; and every one but those two and the key list itself
    /// offers the key list, so every key is in reach from anywhere.
    #[test]
    fn every_key_bar_offers_the_key_list_and_quit() {
        for (name, game, view) in contexts() {
            let keys = keys(&game, &view);
            let last = if name == "filter prompt" {
                ("other", "cancel")
            } else {
                ("q", "quit")
            };
            assert_eq!(keys.last(), Some(&last), "{name}");
            let list = keys.contains(&(key_label(Action::Help), "keys"));
            let expected = !matches!(name, "filter prompt" | "key list");
            assert_eq!(list, expected, "{name}: {keys:?}");
        }
    }

    #[test]
    fn each_key_bar_offers_what_its_state_needs() {
        let all: Vec<_> = contexts()
            .into_iter()
            .map(|(name, game, view)| (name, keys(&game, &view)))
            .collect();
        let has = |name: &str, entry: (&str, &str)| {
            let (_, keys) = all.iter().find(|(n, _)| *n == name).unwrap();
            assert!(keys.contains(&entry), "{name}: {keys:?}");
        };
        has("box mark", ("t", "fill, drop marks"));
        has("hint", ("?", "next rung"));
        has("last rung", ("Enter", "apply"));
        has("mistake", ("Enter", "go to cell"));
        has("marks", ("Esc", "clear marks"));
        has("key list", ("Esc", "close"));
        has("won", ("Enter", "new game"));
    }

    /// The full layout's key bar is one line in the frame's bottom edge;
    /// the dense one takes at most two rows, and shows every key.
    #[test]
    fn every_key_bar_fits_both_layouts() {
        let theme = Theme::tokyo_night_amoled();
        for (name, game, view) in contexts() {
            let keys = keys(&game, &view);
            let width = key_bar(&keys, &theme).width();
            assert!(width <= usize::from(FRAME_WIDTH) - 4, "{name}: {width}");
            let lines = key_bar_lines(&keys, &theme, usize::from(DENSE.0));
            assert!(lines.len() <= 2, "{name}: {} rows", lines.len());
            let backend = render_dense(&game, &view);
            let rows: String = (DENSE.1 - 2..DENSE.1)
                .flat_map(|y| (0..DENSE.0).map(move |x| (x, y)))
                .map(|pos| backend.buffer()[pos].symbol().to_string())
                .collect();
            for (key, words) in &keys {
                assert!(rows.contains(&format!("{key} {words}")), "{name}: {rows}");
            }
        }
    }

    #[test]
    fn the_key_list_in_both_layouts() {
        let view = View {
            help: true,
            ..View::default()
        };
        insta::assert_snapshot!("key_list_full", render(&game(), &view));
        insta::assert_snapshot!("key_list_dense", render_dense(&game(), &view));
    }

    /// The key list names every action with its key, in both layouts.
    #[test]
    fn the_key_list_shows_every_action() {
        let game = game();
        let view = View {
            help: true,
            ..View::default()
        };
        for backend in [render(&game, &view), render_dense(&game, &view)] {
            let buffer = backend.buffer();
            let text: String = (0..buffer.area.height)
                .map(|y| {
                    (0..buffer.area.width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                        + "\n"
                })
                .collect();
            for action in Action::ALL {
                let words = help_words(action);
                assert!(!words.is_empty(), "{action:?}");
                let line = format!("{}  {words}", key_label(action));
                assert!(text.contains(&line), "{line:?} in\n{text}");
            }
        }
    }
}
