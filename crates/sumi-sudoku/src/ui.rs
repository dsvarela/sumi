//! Drawing, and where a click lands. Every colour comes from the theme, and
//! every frame, panel and key bar is drawn with `frame_block`, `panel_block`
//! and `key_bar`, so that screens added later look like these.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph, Widget, Wrap};

use sumi_core::grid::{Cell, col, row, sees};
use sumi_core::hint::{Hint, Rung};
use sumi_core::technique::Level;

use crate::animation::{Animation, Light};
use crate::config::Config;
use crate::config::Highlights;
use crate::game::{CandidatesFromMarks, Game, Mode};
use crate::input::{Action, FilterKey, filter_label, key_label, left_right_label, up_down_label};
use crate::menu::{Item, Menu};
use crate::settings::{GROUPS, Page, Setting};
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
/// lines of text, which every rung fits in at the layout's width.
const DENSE_PANEL_HEIGHT: u16 = 5;

/// The pad under the board: each digit, and under it how many are left to
/// place.
const PAD_HEIGHT: u16 = 2;

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

    /// The smallest window the layout fits in, with the pad or without.
    const fn needed(self, pad_hidden: bool) -> (u16, u16) {
        let pad = pad_height(pad_hidden);
        match self {
            Density::Full => (FRAME_WIDTH, BOARD_HEIGHT + 2 + pad + PANEL_HEIGHT),
            // The status line, the board, the pad, the hint and the key bar.
            Density::Dense => (
                Density::Dense.gutter() + Density::Dense.board_width(),
                1 + BOARD_HEIGHT + pad + DENSE_PANEL_HEIGHT + DENSE_KEY_ROWS,
            ),
        }
    }
}

const fn pad_height(hidden: bool) -> u16 {
    if hidden { 0 } else { PAD_HEIGHT }
}

impl Density {}

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
    /// The cells the last move changed, shaded when a saved game comes
    /// back until the first key.
    pub last_move: Vec<Cell>,
    /// A one-word answer to the last key, shown in the status line.
    pub notice: Option<&'static str>,
    /// The list of every key is open over the game.
    pub help: bool,
    /// The game is paused, and the board hidden.
    pub paused: bool,
    /// The time played, when the timer is on.
    pub time: Option<std::time::Duration>,
    pub highlights: Highlights,
    /// The settings hide the key bar.
    pub key_bar_hidden: bool,
    /// The settings hide the pad, and the layout leaves out its rows.
    pub pad_hidden: bool,
    /// Animations running, oldest first.
    pub animations: Vec<Animation>,
}

impl View {
    /// The open hint, if it shows: pausing hides it with the board, so it
    /// is neither drawn, counted in the status line nor closed by a click.
    pub fn shown_hint(&self) -> Option<&OpenHint> {
        self.hint.as_ref().filter(|_| !self.paused)
    }
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

/// The full layout's board inside its frame, the pad and the panel under it.
#[cfg(test)]
const NEEDED: (u16, u16) = Density::Full.needed(false);

/// The layout a window of this size gets, the full one if it fits, or none
/// if even the dense one doesn't.
pub fn density(area: Rect, pad_hidden: bool) -> Option<Density> {
    let fits = |d: Density| {
        let (width, height) = d.needed(pad_hidden);
        area.width >= width && area.height >= height
    };
    [Density::Full, Density::Dense]
        .into_iter()
        .find(|&d| fits(d))
}

/// Whether `draw` shows the board in a window of this size. The app asks
/// this too, so keys never act on a board the player can't see.
pub fn fits(area: Rect, pad_hidden: bool) -> bool {
    density(area, pad_hidden).is_some()
}

/// Where each part of the game screen goes. Drawing and clicks both work
/// from this, so a click lands on what is drawn under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Areas {
    pub density: Density,
    /// The frame, in the full layout; the dense one has none.
    frame: Option<Rect>,
    /// The dense layout's title and status line.
    status: Rect,
    /// What the key list or the pause screen covers: the board with its
    /// row numbers.
    cover: Rect,
    /// The board's lines and cells.
    board: Rect,
    pad: Rect,
    /// The hint panel, border included in the full layout.
    hint: Rect,
    /// The dense layout's key bar.
    keys: Rect,
}

impl Areas {
    /// The areas in a window of this size, or none if it is too small.
    pub fn of(area: Rect, pad_hidden: bool) -> Option<Areas> {
        let density = density(area, pad_hidden)?;
        let needed = density.needed(pad_hidden);
        let pad_height = pad_height(pad_hidden);
        let whole = area.centered(Constraint::Length(needed.0), Constraint::Length(needed.1));
        let (frame, status, cover, pad, hint, keys) = match density {
            Density::Full => {
                let [frame, pad, hint] = Layout::vertical([
                    Constraint::Length(BOARD_HEIGHT + 2),
                    Constraint::Length(pad_height),
                    Constraint::Length(PANEL_HEIGHT),
                ])
                .areas(whole);
                let cover = frame.inner(ratatui::layout::Margin::new(1, 1));
                (
                    Some(frame),
                    Rect::default(),
                    cover,
                    pad,
                    hint,
                    Rect::default(),
                )
            }
            Density::Dense => {
                let [status, cover, pad, hint, keys] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(BOARD_HEIGHT),
                    Constraint::Length(pad_height),
                    Constraint::Length(DENSE_PANEL_HEIGHT),
                    Constraint::Length(DENSE_KEY_ROWS),
                ])
                .areas(whole);
                (None, status, cover, pad, hint, keys)
            }
        };
        let gutter = density.gutter();
        let numbered = cover.centered(
            Constraint::Length(gutter + density.board_width()),
            Constraint::Length(BOARD_HEIGHT),
        );
        let board = Rect {
            x: numbered.x + gutter,
            width: numbered.width - gutter,
            ..numbered
        };
        Some(Areas {
            density,
            frame,
            status,
            cover,
            board,
            pad,
            hint,
            keys,
        })
    }

    /// Where a cell's contents go, inside the lines around it.
    pub fn cell(&self, cell: Cell) -> Rect {
        let width = self.density.cell_width();
        let x = self.board.x + 1 + col(cell) as u16 * (width + 1);
        let y = self.board.y + 1 + row(cell) as u16 * (CELL_HEIGHT + 1);
        Rect::new(x, y, width, CELL_HEIGHT)
    }

    /// Where a note for `digit` goes in `cell`: a fixed spot in a 3×3 grid,
    /// 1 top left and 9 bottom right, spaced out in the full layout and
    /// packed in the dense one.
    fn note(&self, cell: Cell, digit: u8) -> (u16, u16) {
        let Rect { x, y, .. } = self.cell(cell);
        let spot = u16::from(digit - 1);
        match self.density {
            Density::Full => (x + 1 + 2 * (spot % 3), y + spot / 3),
            Density::Dense => (x + spot % 3, y + spot / 3),
        }
    }

    /// The pad's button for `digit`: the column of the board's cells above
    /// it, both rows. Empty when the pad is hidden.
    pub fn pad_button(&self, digit: u8) -> Rect {
        let width = self.density.cell_width();
        let x = self.board.x + 1 + u16::from(digit - 1) * (width + 1);
        Rect::new(x, self.pad.y, width, self.pad.height)
    }

    /// The button that closes the hint: on the panel's top edge in the full
    /// layout, at the end of the rung's line in the dense one.
    pub fn close_hint(&self) -> Rect {
        match self.density {
            Density::Full => Rect::new(self.hint.right() - 4, self.hint.y, 3, 1),
            Density::Dense => Rect::new(self.hint.right() - 1, self.hint.y, 1, 1),
        }
    }
}

/// What a click at a spot of the game screen lands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Cell(Cell),
    /// A note's spot in a cell, in the full layout.
    Note(Cell, u8),
    Pad(u8),
    CloseHint,
}

/// What the click at `(x, y)` lands on, in a window of `area`. The board
/// and the pad take no clicks while the key list or the pause screen covers
/// them, and the close button is there only with a hint open.
pub fn hit(area: Rect, view: &View, x: u16, y: u16) -> Option<Target> {
    let areas = Areas::of(area, view.pad_hidden)?;
    let at = ratatui::layout::Position::new(x, y);
    if view.shown_hint().is_some() && areas.close_hint().contains(at) {
        return Some(Target::CloseHint);
    }
    if view.help || view.paused {
        return None;
    }
    for cell in 0..81 {
        if areas.cell(cell).contains(at) {
            if areas.density == Density::Full {
                for digit in 1..=9 {
                    if areas.note(cell, digit) == (x, y) {
                        return Some(Target::Note(cell, digit));
                    }
                }
            }
            return Some(Target::Cell(cell));
        }
    }
    (1..=9)
        .find(|&d| areas.pad_button(d).contains(at))
        .map(Target::Pad)
}

pub fn draw(frame: &mut Frame, game: &Game, view: &View, theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);
    let Some(areas) = Areas::of(area, view.pad_hidden) else {
        too_small(frame, theme, Density::Dense.needed(view.pad_hidden));
        return;
    };
    let rung = view.shown_hint().map(|open| open.hint.rung(open.rung));
    let board = Board {
        game,
        view,
        theme,
        rung,
        areas,
    };
    let keys = shown_keys(game, view);
    match areas.frame {
        Some(outer) => {
            let mut block = frame_block(theme)
                .title_top(title(theme))
                .title_top(status(game, view, theme, STATUS_WIDTH).right_aligned());
            if !keys.is_empty() {
                block = block.title_bottom(key_bar(&keys, theme).centered());
            }
            frame.render_widget(block, outer);
        }
        None => {
            let width = areas.status.width as usize - TITLE.len() - 2;
            let [title_area, status_area] =
                Layout::horizontal([Constraint::Length(TITLE.len() as u16), Constraint::Fill(1)])
                    .areas(areas.status);
            frame.render_widget(title(theme), title_area);
            frame.render_widget(
                status(game, view, theme, width).right_aligned(),
                status_area,
            );
            let lines = key_bar_lines(&keys, theme, usize::from(areas.keys.width));
            frame.render_widget(Paragraph::new(lines).centered(), areas.keys);
        }
    }
    if view.help {
        let margin = match areas.density {
            Density::Full => ratatui::layout::Margin::new(2, 1),
            Density::Dense => ratatui::layout::Margin::new(1, 0),
        };
        frame.render_widget(key_list(theme), areas.cover.inner(margin));
    } else if view.paused {
        frame.render_widget(
            paused(theme),
            areas.cover.centered_vertically(Constraint::Length(1)),
        );
    } else {
        frame.render_widget(board, areas.board);
        draw_pad(frame, game, view, theme, &areas);
    }
    let text = hint_text(game, view, theme);
    let close = Span::styled(
        "×",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    );
    match areas.density {
        Density::Full => {
            let mut block = panel_block(theme, if game.is_won() { "Solved" } else { "Hint" });
            if view.shown_hint().is_some() {
                block = block
                    .title_top(Line::from(vec![" ".into(), close, " ".into()]).right_aligned());
            }
            let inner = block.inner(areas.hint);
            frame.render_widget(block, areas.hint);
            frame.render_widget(text, inner);
        }
        Density::Dense => {
            frame.render_widget(text, areas.hint);
            if view.shown_hint().is_some() {
                frame.render_widget(close, areas.close_hint());
            }
        }
    }
}

/// Each digit under its column of the board, and how many of it are left
/// to place. A finished digit is dimmed, as on the board.
fn draw_pad(frame: &mut Frame, game: &Game, view: &View, theme: &Theme, areas: &Areas) {
    if view.pad_hidden {
        return;
    }
    let buf = frame.buffer_mut();
    for digit in 1..=9u8 {
        let button = areas.pad_button(digit);
        let placed = (0..81)
            .filter(|&c| game.board().get(c) == Some(digit))
            .count();
        let left = 9usize.saturating_sub(placed);
        let finished = view.highlights.complete_digits && game.is_complete(digit);
        let style = if finished {
            Style::new().fg(theme.dimmed)
        } else {
            Style::new().fg(theme.text).add_modifier(Modifier::BOLD)
        };
        let middle = button.x + button.width / 2;
        buf.set_string(middle, button.y, digit.to_string(), style);
        buf.set_string(
            middle,
            button.y + 1,
            left.to_string(),
            Style::new().fg(theme.dimmed),
        );
    }
}

/// What shows in the board's place while the game is paused.
fn paused(theme: &Theme) -> Paragraph<'static> {
    Paragraph::new(Span::styled(
        "Paused",
        Style::new().fg(theme.text).add_modifier(Modifier::BOLD),
    ))
    .centered()
}

/// The panel's words: the hint, how to ask for one, or once the puzzle is
/// solved, what it took.
fn hint_text(game: &Game, view: &View, theme: &Theme) -> Paragraph<'static> {
    if game.is_won() {
        return Paragraph::new(solved_text(game, view, theme)).wrap(Wrap { trim: true });
    }
    if view.paused {
        return Paragraph::new("");
    }
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

/// The win screen's words: the rating, the hints and reveals, and the time
/// if the timer is on. Nothing is judged.
fn solved_text(game: &Game, view: &View, theme: &Theme) -> Vec<Line<'static>> {
    let first = match game.level() {
        Some(level) => format!("Solved, {}.", level_name(level)),
        None => "Solved.".to_string(),
    };
    let mut parts = vec![match game.hints_used() {
        0 => "no hints".to_string(),
        1 => "1 hint".to_string(),
        n => format!("{n} hints"),
    }];
    match game.revealed() {
        0 => {}
        1 => parts.push("1 cell revealed".to_string()),
        n => parts.push(format!("{n} cells revealed")),
    }
    if let Some(time) = view.time {
        parts.push(clock_text(time));
    }
    let mut second = parts.join(", ");
    second[..1].make_ascii_uppercase();
    vec![
        Line::from(Span::styled(first, Style::new().fg(theme.text))),
        Line::from(Span::styled(
            format!("{second}."),
            Style::new().fg(theme.dimmed),
        )),
    ]
}

fn level_name(level: Level) -> &'static str {
    match level {
        Level::Easy => "Easy",
        Level::Medium => "Medium",
        Level::Hard => "Hard",
        Level::Expert => "Expert",
        Level::Beyond => "Beyond",
    }
}

/// Minutes and seconds, with hours in front once there are any.
fn clock_text(time: std::time::Duration) -> String {
    let seconds = time.as_secs();
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
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
    // Each part with its place on the line, most useful first: what the
    // player has turned on or asked for comes before the level and the
    // time. What does not fit beside the title is shortened, then left off
    // from the end, so the mode, which is always shown, is never lost.
    let mut optional: Vec<(u8, String)> = Vec::new();
    if let Some(open) = view.shown_hint() {
        optional.push((5, format!("hint {}/{}", open.rung, open.hint.rungs())));
    }
    if let Some(notice) = view.notice {
        optional.push((4, notice.to_string()));
    }
    match view.filter {
        Some(Filter::Digit(d)) => optional.push((3, format!("filter {d}"))),
        Some(Filter::TwoCandidates) => optional.push((3, "filter pairs".to_string())),
        None => {}
    }
    if view.notes_hidden {
        optional.push((2, "candidates and box marks hidden".to_string()));
    }
    if let Some(time) = view.time {
        optional.push((1, clock_text(time)));
    }
    if let Some(level) = game.level() {
        optional.push((0, level_name(level).to_string()));
    }
    let fits = |parts: &[(u8, String)]| {
        let shown: Vec<&str> = parts
            .iter()
            .map(|(_, p)| p.as_str())
            .chain([mode])
            .collect();
        shown.join(" · ").chars().count() <= width
    };
    if !fits(&optional) {
        for (place, part) in &mut optional {
            if *place == 2 {
                *part = "notes hidden".to_string();
            }
        }
    }
    while !fits(&optional) {
        optional.pop();
    }
    // Shown in the order the player reads them: the level and time, the
    // settings, the hint, the mode.
    optional.sort_by_key(|&(place, _)| place);
    let mut parts: Vec<String> = optional.into_iter().map(|(_, part)| part).collect();
    parts.push(mode.to_string());
    Line::from(Span::styled(
        format!(" {} ", parts.join(" · ")),
        Style::new().fg(theme.dimmed),
    ))
}

/// The key bar's keys: none when the settings hide it, except for the
/// filter's prompt, which can't be read without them.
fn shown_keys(game: &Game, view: &View) -> Vec<(&'static str, &'static str)> {
    if view.key_bar_hidden && !view.awaiting_filter {
        return Vec::new();
    }
    keys(game, view)
}

/// The keys the key bar offers, as key text and what the key does. Key
/// text comes from `input::key_label`, so it always matches the bindings.
/// Each list fits one line of the full layout; `i` lists the rest.
fn keys(game: &Game, view: &View) -> Vec<(&'static str, &'static str)> {
    if view.awaiting_filter {
        // The key after f is read by `input::filter_key`, not `input::action`.
        return vec![
            (filter_label(FilterKey::Digit(1)), "digit"),
            (filter_label(FilterKey::TwoCandidates), "pairs"),
            (filter_label(FilterKey::Off), "off"),
            ("other", "cancel"),
        ];
    }
    let mut keys: Vec<(Action, &str)> = if view.help {
        vec![(Action::Close, "close")]
    } else if view.paused {
        vec![(Action::Pause, "carry on")]
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
        vec![(Action::Confirm, "new game"), (Action::Close, "menu")]
    } else if game.at_dead_end() && !game.bookmarks().is_empty() {
        // A guess has gone wrong: going back comes first.
        vec![
            (Action::BackToBookmark, "back to bookmark"),
            (Action::Undo, "undo"),
            (Action::Digit(1), "place"),
            (Action::NextMode, "mode"),
        ]
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
                (
                    Action::MarksToCandidates,
                    match game.candidates_from_marks() {
                        CandidatesFromMarks::Fill => "fill, drop marks",
                        CandidatesFromMarks::Merge => "add to candidates",
                    },
                ),
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

/// The rounded frame the full layout, the menus and the settings sit in. A
/// game titles it with the program's name; a menu has the logo inside. The
/// dense layout has none: the board's outline is its frame.
pub fn frame_block(theme: &Theme) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme.frame))
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
    key_rows(keys, width)
        .iter()
        .map(|row| key_bar(row, theme))
        .collect()
}

/// `keys` split into rows of at most `width` columns, as `key_bar` draws
/// them.
fn key_rows<'a>(keys: &[(&'a str, &'a str)], width: usize) -> Vec<Vec<(&'a str, &'a str)>> {
    // A space each side, two between keys, and one between a key and its
    // words.
    let width_of = |row: &[(&str, &str)]| {
        let text: usize = row
            .iter()
            .map(|(k, w)| k.chars().count() + 1 + w.chars().count())
            .sum();
        text + 2 + 2 * row.len().saturating_sub(1)
    };
    let mut rows: Vec<Vec<(&str, &str)>> = vec![Vec::new()];
    for &entry in keys {
        let row = rows.last_mut().unwrap();
        row.push(entry);
        if row.len() > 1 && width_of(row) > width {
            row.pop();
            rows.push(vec![entry]);
        }
    }
    rows
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
        Action::MarksToCandidates => "box marks to candidates",
        Action::ClearLayer => "clear this mode's notes",
        Action::ClearAllNotes => "clear all notes",
        Action::ToggleNotes => "hide or show notes",
        Action::Filter => "filter: 1-9, b pairs, 0 off",
        Action::Hint => "hint, then its next rung",
        Action::Confirm => "apply hint, go to mistake",
        Action::Check => "check for mistakes",
        Action::Reveal => "reveal this cell",
        Action::Close => "close, clear marks, menu",
        Action::Undo => "undo",
        Action::Redo => "redo",
        Action::Bookmark => "bookmark the board",
        Action::BackToBookmark => "back to the last bookmark",
        Action::Help => "this list",
        Action::Pause => "pause, or carry on (Space)",
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
        Action::Undo
        | Action::Redo
        | Action::Bookmark
        | Action::BackToBookmark
        | Action::Pause
        | Action::Help
        | Action::Quit => "Game",
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
fn too_small(frame: &mut Frame, theme: &Theme, smallest: (u16, u16)) {
    let area = frame.area();
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
    areas: Areas,
}

impl Widget for Board<'_> {
    /// `area` is `Areas::board`, where the board's lines go.
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_lines(area, buf);
        let conflicts = self.game.board().conflicts();
        for (cell, &conflict) in conflicts.iter().enumerate() {
            self.render_cell(cell, conflict, buf);
        }
        if self.areas.density == Density::Dense {
            self.render_cursor_frame(buf);
        }
        self.render_numbers(area, buf);
    }
}

impl Board<'_> {
    fn render_lines(&self, area: Rect, buf: &mut Buffer) {
        let cell_width = self.areas.density.cell_width();
        for dy in 0..BOARD_HEIGHT {
            for dx in 0..self.areas.density.board_width() {
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
        let cell_width = self.areas.density.cell_width();
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
                area.x - self.areas.density.gutter(),
                area.y + offset + CELL_HEIGHT / 2,
                &number,
                style(row(cursor) == i),
            );
        }
    }

    fn render_cell(&self, cell: Cell, conflict: bool, buf: &mut Buffer) {
        let rect = self.areas.cell(cell);
        let (x, y) = (rect.x, rect.y);
        if let Some(bg) = self.background(cell) {
            buf.set_style(rect, Style::new().bg(bg));
        }
        // Without colour, what nothing else names is drawn reversed: a
        // hint's pattern, the filter's cells and an animation's strong light.
        let hint_cell = self.rung.as_ref().is_some_and(|r| r.cells.contains(&cell));
        let pulse = self.light(cell) == Some(Light::Strong);
        if self.theme.monochrome && (hint_cell || pulse || self.filtered(cell)) {
            buf.set_style(rect, Style::new().add_modifier(Modifier::REVERSED));
        }
        if cell == self.game.cursor() && self.areas.density == Density::Full {
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
                } else if self.view.highlights.complete_digits
                    && self.game.is_complete(digit)
                    && !self.game.is_won()
                {
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
            None if !self.view.notes_hidden => self.render_notes(cell, buf),
            None => {}
        }
        self.render_hint_marks(cell, rect, buf);
    }

    /// How the animations running light `cell`; the strongest wins.
    fn light(&self, cell: Cell) -> Option<Light> {
        let lights = self.view.animations.iter().filter_map(|a| a.light(cell));
        lights.min_by_key(|light| *light != Light::Strong)
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

    /// The dense layout's cells have no room for corner marks, so the lines
    /// around the cursor are drawn double instead: a shape no other line on
    /// the board has.
    fn render_cursor_frame(&self, buf: &mut Buffer) {
        let cell = self.areas.cell(self.game.cursor());
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
                let (nx, ny) = self.areas.note(cell, digit);
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
            let blank = match self.areas.density {
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

    /// Each note goes in its digit's spot (`Areas::note`). A box mark takes
    /// the spot over from a candidate.
    fn render_notes(&self, cell: Cell, buf: &mut Buffer) {
        let candidates = self.game.candidates(cell);
        let marks = self.game.marks(cell);
        for digit in candidates.union(marks).iter() {
            let (nx, ny) = self.areas.note(cell, digit);
            let mut style = if marks.contains(digit) {
                Style::new()
                    .fg(self.theme.box_mark)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::new().fg(self.theme.candidate)
            };
            if self.view.highlights.note_clashes && self.game.note_clashes(cell, digit) {
                style = style
                    .fg(self.theme.conflict)
                    .add_modifier(Modifier::REVERSED);
            }
            buf.set_string(nx, ny, digit.to_string(), style);
        }
    }

    /// A hint's cells win over everything, the cursor's corner marks being
    /// enough to find the cursor. The cursor wins over an animation's light,
    /// which wins over the filter and the same-digit highlight; those win
    /// over a hint's area and the last move's cells on coming back to a
    /// saved game, which win over the shading of the cursor's row,
    /// column and box. The
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
        } else if let Some(light) = self.light(cell) {
            Some(match light {
                Light::Strong => self.theme.same_digit,
                Light::Faint => self.theme.hint_area,
            })
        } else if filtered {
            Some(self.theme.filter)
        } else if self.view.highlights.same_digit
            && digit.is_some()
            && digit == game.board().get(cursor)
        {
            Some(self.theme.same_digit)
        } else if rung.is_some_and(|r| {
            r.second_group.contains(&cell) || r.areas.iter().any(|h| h.contains(cell))
        }) || self.view.last_move.contains(&cell)
        {
            Some(self.theme.hint_area)
        } else if self.view.highlights.peer && sees(cell, cursor) {
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

/// The logo as pixels: lowercase "sumi", 17 wide and 6 tall, the dot of
/// the i on the top row.
const LOGO: [&str; 6] = [
    "................#",
    ".................",
    ".###.#..#.####..#",
    "##...#..#.#.#.#.#",
    "..##.#..#.#.#.#.#",
    "###...###.#.#.#.#",
];

/// The logo drawn with half blocks, each pixel `scale` characters wide and
/// `scale` half rows tall: 3 rows at scale 1, 6 at scale 2.
fn logo(scale: usize) -> Vec<String> {
    let pixels: Vec<Vec<bool>> = LOGO
        .iter()
        .flat_map(|row| {
            let wide: Vec<bool> = row
                .chars()
                .flat_map(|c| std::iter::repeat_n(c == '#', scale))
                .collect();
            std::iter::repeat_n(wide, scale)
        })
        .collect();
    pixels
        .chunks(2)
        .map(|pair| {
            (0..pair[0].len())
                .map(|x| match (pair[0][x], pair[1][x]) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// The smallest window a menu with `items` can be drawn in: the frame
/// around the name in plain text, or the menu's question, and the items, and under it the key bar
/// in as many rows as it takes.
fn menu_needed(menu: &Menu, items: &[Item]) -> (u16, u16) {
    let keys = menu_keys(menu);
    let heading = menu.question().unwrap_or("sumi").to_string();
    let width = MenuAreas::column(&[heading], items) + 6;
    let key_rows = key_rows(&keys, usize::from(width)).len() as u16;
    (width, MenuAreas::height(1, items) + key_rows)
}

pub fn menu_fits(area: Rect, menu: &Menu, items: &[Item]) -> bool {
    let (width, height) = menu_needed(menu, items);
    area.width >= width && area.height >= height
}

/// The keys a menu offers.
fn menu_keys(menu: &Menu) -> Vec<(&'static str, &'static str)> {
    let mut keys = vec![
        (up_down_label(), "move"),
        (key_label(Action::Confirm), "choose"),
    ];
    if menu.has_back() {
        keys.push((key_label(Action::Close), "back"));
    }
    keys.push((key_label(Action::Quit), "quit"));
    keys
}

/// Where a menu's parts go: the box, the logo that fits, the column the
/// logo and items are drawn in, and the key bar's rows.
struct MenuAreas {
    boxed: Rect,
    logo: Vec<String>,
    column: Rect,
    keys: Rect,
}

impl MenuAreas {
    fn of(area: Rect, menu: &Menu, items: &[Item]) -> MenuAreas {
        let key_rows = key_rows(&menu_keys(menu), usize::from(area.width)).len() as u16;
        let rows = items.len() as u16;
        // The frame and a row of space inside it, top and bottom.
        let around = 4 + key_rows;
        let fits = |logo: &[String]| {
            let width = logo.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
            area.width >= width + 6 && area.height >= logo.len() as u16 + 1 + rows + around
        };
        let logo = match menu.question() {
            Some(question) => vec![question.to_string()],
            None => [logo(2), logo(1)]
                .into_iter()
                .find(|logo| fits(logo))
                .unwrap_or_else(|| vec!["sumi".to_string()]),
        };
        let column = MenuAreas::column(&logo, items);
        let height = MenuAreas::height(logo.len() as u16, items);
        let [boxed, keys] =
            Layout::vertical([Constraint::Length(height), Constraint::Length(key_rows)]).areas(
                area.centered(
                    Constraint::Length((column + 6).min(area.width)),
                    Constraint::Length(height + key_rows),
                ),
            );
        let inner = boxed.inner(ratatui::layout::Margin::new(2, 2));
        let column = inner.centered_horizontally(Constraint::Length(column.min(inner.width)));
        let keys = Rect {
            x: area.x,
            width: area.width,
            ..keys
        };
        MenuAreas {
            boxed,
            logo,
            column,
            keys,
        }
    }

    /// The width of the column the logo and items are drawn in: the widest
    /// of them, the items with their selection marker.
    fn column(logo: &[String], items: &[Item]) -> u16 {
        logo.iter()
            .map(|l| l.chars().count())
            .chain(items.iter().map(|item| item.label().chars().count() + 2))
            .max()
            .unwrap_or(0) as u16
    }

    /// The box's height: the logo's rows, a blank row, the items, and the
    /// frame with a row of space inside it, top and bottom.
    fn height(logo_rows: u16, items: &[Item]) -> u16 {
        logo_rows + 1 + items.len() as u16 + 4
    }

    /// The row of the item at `index`, as wide as the column it is drawn
    /// in.
    fn item(&self, index: usize) -> Rect {
        let y = self.column.y + self.logo.len() as u16 + 1 + index as u16;
        Rect::new(self.column.x, y, self.column.width, 1)
    }

    /// Where a click chooses the item at `index`: its row, across the box.
    fn target(&self, index: usize) -> Rect {
        let row = self.item(index);
        Rect::new(self.boxed.x + 1, row.y, self.boxed.width - 2, 1)
    }
}

/// The item a click at `(x, y)` lands on.
pub fn menu_hit(area: Rect, menu: &Menu, items: &[Item], x: u16, y: u16) -> Option<usize> {
    if !menu_fits(area, menu, items) {
        return None;
    }
    let areas = MenuAreas::of(area, menu, items);
    let at = ratatui::layout::Position::new(x, y);
    (0..items.len()).find(|&i| areas.target(i).contains(at))
}

/// A menu in its frame, with the biggest logo that fits above the items
/// and the key bar under it.
pub fn draw_menu(frame: &mut Frame, menu: &Menu, items: &[Item], theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);
    if !menu_fits(area, menu, items) {
        too_small(frame, theme, menu_needed(menu, items));
        return;
    }
    let areas = MenuAreas::of(area, menu, items);
    frame.render_widget(frame_block(theme), areas.boxed);
    let accent = Style::new().fg(theme.accent).add_modifier(Modifier::BOLD);
    let logo: Vec<Line> = areas
        .logo
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), accent)))
        .collect();
    frame.render_widget(Paragraph::new(logo), areas.column);
    for (i, item) in items.iter().enumerate() {
        // The marker shows the selection without relying on colour.
        let line = if i == menu.selected {
            Span::styled(format!("› {}", item.label()), accent)
        } else {
            Span::styled(format!("  {}", item.label()), Style::new().fg(theme.text))
        };
        frame.render_widget(line, areas.item(i));
    }
    let keys = key_bar_lines(&menu_keys(menu), theme, usize::from(area.width));
    frame.render_widget(Paragraph::new(keys).centered(), areas.keys);
}

/// What the settings page draws.
pub struct SettingsPage<'a> {
    pub page: &'a Page,
    pub config: &'a Config,
    /// The theme the config asks for, by name.
    pub theme_name: &'a str,
    /// Why the last change couldn't be saved.
    pub error: Option<&'a str>,
    /// Why the theme chosen isn't the one drawn, when the terminal decides.
    pub note: Option<&'a str>,
}

/// A row of the settings page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsRow {
    Blank,
    Heading(&'static str),
    /// A setting, and its place in `Setting::all`.
    Setting(usize, Setting),
    /// Where a group has no settings: the Keys group.
    NoSettings,
}

/// The settings page's rows, in order. Drawing and clicks both use it.
fn settings_rows() -> Vec<SettingsRow> {
    let mut rows = Vec::new();
    let mut index = 0;
    for (name, settings) in GROUPS {
        if !rows.is_empty() {
            rows.push(SettingsRow::Blank);
        }
        rows.push(SettingsRow::Heading(name));
        for &setting in settings {
            rows.push(SettingsRow::Setting(index, setting));
            index += 1;
        }
        if settings.is_empty() {
            rows.push(SettingsRow::NoSettings);
        }
    }
    rows
}

fn settings_lines(page: &SettingsPage, theme: &Theme) -> Vec<Line<'static>> {
    let heading = Style::new().fg(theme.text).add_modifier(Modifier::BOLD);
    let accent = Style::new().fg(theme.accent).add_modifier(Modifier::BOLD);
    let selected = page.page.setting();
    let mut lines = Vec::new();
    for row in settings_rows() {
        let setting = match row {
            SettingsRow::Blank => {
                lines.push(Line::default());
                continue;
            }
            SettingsRow::Heading(name) => {
                lines.push(Line::from(Span::styled(name, heading)));
                continue;
            }
            SettingsRow::NoSettings => {
                lines.push(Line::from(Span::styled(
                    format!("  {} in a game lists them", key_label(Action::Help)),
                    Style::new().fg(theme.dimmed),
                )));
                continue;
            }
            SettingsRow::Setting(_, setting) => setting,
        };
        {
            let value = shorten(&setting.shown(page.config, page.theme_name), VALUE_WIDTH);
            let line = if setting == selected {
                Line::from(vec![
                    Span::styled(format!("› {:<20}  ", setting.label()), accent),
                    Span::styled(format!("‹ {value} ›"), accent),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("  {:<20}  ", setting.label()),
                        Style::new().fg(theme.text),
                    ),
                    Span::styled(format!("  {value}"), Style::new().fg(theme.dimmed)),
                ])
            };
            lines.push(line);
        }
    }
    lines
}

/// The widest value a setting row has room for beside its label; a longer
/// one, such as a user theme's name, is shortened.
const VALUE_WIDTH: usize = 18;

/// `text`, cut to `width` characters with … as the last when it is longer.
fn shorten(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let kept: String = text.chars().take(width - 1).collect();
    format!("{kept}…")
}

/// The page's last two rows: why the last change wasn't saved, or else why
/// the theme chosen isn't the one drawn.
fn settings_message(page: &SettingsPage, theme: &Theme) -> Paragraph<'static> {
    let line = match (page.error, page.note) {
        // The page always saves to config.toml, so its path (named on stderr)
        // gives way to the reason, and a long error to its first line.
        (Some(error), _) => Line::from(Span::styled(
            format!(
                "config.toml not saved: {}",
                error
                    .split_once(": ")
                    .map_or(error, |(_, reason)| reason)
                    .lines()
                    .next()
                    .unwrap_or_default()
            ),
            Style::new().fg(theme.conflict),
        )),
        (None, Some(note)) => Line::from(Span::styled(
            note.to_string(),
            Style::new().fg(theme.dimmed),
        )),
        (None, None) => Line::default(),
    };
    Paragraph::new(line).wrap(Wrap { trim: true })
}

/// The settings page's width: its widest row, a label and the longest
/// built-in theme name, in the frame.
const SETTINGS_WIDTH: u16 = 54;

/// The smallest window the settings page fits in: its width, and its rows
/// with the two kept for a save error, the frame, and the keys.
fn settings_needed() -> (u16, u16) {
    let key_rows = key_rows(&settings_keys(), usize::from(SETTINGS_WIDTH)).len() as u16;
    (
        SETTINGS_WIDTH,
        settings_rows().len() as u16 + 2 + 4 + key_rows,
    )
}

pub fn settings_fit(area: Rect) -> bool {
    let (width, height) = settings_needed();
    area.width >= width && area.height >= height
}

fn settings_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        (up_down_label(), "move"),
        (left_right_label(), "change"),
        (key_label(Action::Close), "back"),
        (key_label(Action::Quit), "quit"),
    ]
}

/// Where the settings page's box, its rows and its key bar go. The rows
/// are `settings_rows` and the two kept for a save error.
fn settings_areas(area: Rect) -> (Rect, Rect, Rect) {
    let rows = settings_rows().len() as u16 + 2;
    let key_rows = key_rows(&settings_keys(), usize::from(area.width)).len() as u16;
    let width = SETTINGS_WIDTH - 2;
    let height = rows + 4;
    let [boxed, keys] =
        Layout::vertical([Constraint::Length(height), Constraint::Length(key_rows)]).areas(
            area.centered(
                Constraint::Length(width.min(area.width)),
                Constraint::Length((height + key_rows).min(area.height)),
            ),
        );
    let inner = boxed.inner(ratatui::layout::Margin::new(3, 2));
    let keys = Rect {
        x: area.x,
        width: area.width,
        ..keys
    };
    (boxed, inner, keys)
}

/// The setting a click at `(x, y)` lands on, by its place in `Setting::all`.
pub fn settings_hit(area: Rect, x: u16, y: u16) -> Option<usize> {
    if !settings_fit(area) {
        return None;
    }
    let rows = settings_rows();
    let (_, inner, _) = settings_areas(area);
    if !inner.contains(ratatui::layout::Position::new(x, y)) {
        return None;
    }
    match rows.get(usize::from(y - inner.y))? {
        SettingsRow::Setting(index, _) => Some(*index),
        _ => None,
    }
}

pub fn draw_settings(frame: &mut Frame, page: &SettingsPage, theme: &Theme) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme.background)), area);
    if !settings_fit(area) {
        too_small(frame, theme, settings_needed());
        return;
    }
    let lines = settings_lines(page, theme);
    let (boxed, inner, keys_area) = settings_areas(area);
    let [rows, message] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(inner);
    let block = frame_block(theme).title_top(Line::from(Span::styled(
        " Settings ",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(block, boxed);
    frame.render_widget(Paragraph::new(lines), rows);
    frame.render_widget(settings_message(page, theme), message);
    let keys = key_bar_lines(&settings_keys(), theme, usize::from(area.width));
    frame.render_widget(Paragraph::new(keys).centered(), keys_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{game, solution};
    use crate::menu::Progress;
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

    /// The last move's cells are shaded; the cells around them are not.
    #[test]
    fn the_last_move_is_shaded() {
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let view = View {
            last_move: vec![70],
            ..View::default()
        };
        assert_eq!(background_at(&game, &view, 70), theme.hint_area);
        assert_ne!(background_at(&game, &view, 71), theme.hint_area);
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
        let top = BOARD_HEIGHT + 2 + PAD_HEIGHT;
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
            hint_text(&game(), &view, &theme).render(area, &mut buffer);
            (0..area.height)
                .filter(|&y| (0..area.width).any(|x| buffer[(x, y)].symbol() != " "))
                .count()
        };
        let mut hints = vec![Hint::Mistake(80), Hint::StruckOut(80), Hint::Stuck];
        let mut rng = rand::rngs::Xoshiro256PlusPlus::seed_from_u64(5);
        for level in [Level::Hard, Level::Expert] {
            for _ in 0..8 {
                let mut game =
                    Game::new(sumi_core::bank::pick(&mut rng, level, &Default::default()).unwrap());
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

    const DENSE: (u16, u16) = Density::Dense.needed(false);

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
        let at = |width, height| density(Rect::new(0, 0, width, height), false);
        assert_eq!(at(75, 47), Some(Density::Full));
        assert_eq!(at(75, 46), Some(Density::Full));
        assert_eq!(at(74, 47), Some(Density::Dense));
        assert_eq!(at(38, 47), Some(Density::Dense));
        assert_eq!(at(37, 47), None);
        assert_eq!(at(38, 46), None);
        assert_eq!(at(200, 45), None);
        assert_eq!((NEEDED, DENSE), ((75, 46), (38, 47)));
    }

    /// Without the pad, both layouts need its two rows less, nothing
    /// is drawn or clicked where it was, and the hint sits under the
    /// board.
    #[test]
    fn a_hidden_pad_gives_its_rows_back() {
        assert_eq!(
            (Density::Full.needed(true), Density::Dense.needed(true)),
            ((75, 44), (38, 45))
        );
        let at = |width, height| density(Rect::new(0, 0, width, height), true);
        assert_eq!(
            (at(75, 44), at(38, 45), at(38, 44)),
            (Some(Density::Full), Some(Density::Dense), None)
        );
        let view = View {
            pad_hidden: true,
            ..View::default()
        };
        for area in [Rect::new(0, 0, 75, 44), Rect::new(0, 0, 38, 45)] {
            let areas = Areas::of(area, true).unwrap();
            assert_eq!(
                areas.hint.y,
                areas.board.bottom() + u16::from(areas.frame.is_some())
            );
            for y in 0..area.height {
                for x in 0..area.width {
                    assert!(!matches!(hit(area, &view, x, y), Some(Target::Pad(_))));
                }
            }
        }
        let mut terminal = Terminal::new(TestBackend::new(75, 44)).unwrap();
        terminal
            .draw(|frame| draw(frame, &game(), &view, &Theme::tokyo_night_amoled()))
            .unwrap();
        let below: String = (0..75)
            .map(|x| terminal.backend().buffer()[(x, BOARD_HEIGHT + 2)].symbol())
            .collect();
        assert!(
            below.starts_with("╭") || below.contains("─"),
            "the hint panel: {below}"
        );
        assert!(!below.contains('9'), "no pad: {below}");
        // The dense hint area has no border to cover a pad drawn by mistake.
        let mut terminal = Terminal::new(TestBackend::new(38, 45)).unwrap();
        terminal
            .draw(|frame| draw(frame, &game(), &view, &Theme::tokyo_night_amoled()))
            .unwrap();
        let below: String = (0..38)
            .map(|x| terminal.backend().buffer()[(x, 1 + BOARD_HEIGHT)].symbol())
            .collect();
        assert!(!below.contains('9'), "no pad: {below}");
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
        let first = 1 + BOARD_HEIGHT + PAD_HEIGHT;
        let rung: String = (0..DENSE.0).map(|x| b[(x, first)].symbol()).collect();
        assert!(rung.starts_with("Rung 2 of 5 "), "{rung}");
        assert!(rung.ends_with('×'), "the hint's close button: {rung}");
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
    /// A game with a bookmark, then a 5 placed in r1c3 beside the 5 in
    /// r1c1.
    fn dead_end() -> Game {
        let mut game = game();
        game.bookmark();
        go(&mut game, 2);
        game.press(5);
        game
    }

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
            ("dead end", dead_end(), View::default()),
            (
                "paused",
                game(),
                View {
                    paused: true,
                    ..View::default()
                },
            ),
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
        has("won", ("Esc", "menu"));
        has("paused", ("p", "carry on"));
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

    /// At a dead end with a bookmark the key bar offers B first; without
    /// a bookmark it can't, and keeps the usual keys.
    #[test]
    fn a_dead_end_offers_the_bookmark() {
        let keys_at = |game: &Game| keys(game, &View::default());
        assert_eq!(keys_at(&dead_end())[0], ("B", "back to bookmark"));
        let mut unmarked = game();
        go(&mut unmarked, 2);
        unmarked.press(5);
        assert_eq!(keys_at(&unmarked), keys_at(&game()));
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

    fn top_line(backend: &TestBackend) -> String {
        let buffer = backend.buffer();
        (0..buffer.area.width)
            .map(|x| buffer[(x, 0)].symbol())
            .collect()
    }

    #[test]
    fn the_status_line_shows_the_level_and_the_time() {
        let game = game().rated(Level::Medium);
        let view = View {
            time: Some(std::time::Duration::from_secs(65)),
            ..View::default()
        };
        let top = top_line(&render(&game, &view));
        assert!(top.contains(" Medium · 1:05 · ● DIGIT "), "{top}");
        let without = top_line(&render(&game, &View::default()));
        assert!(without.contains(" Medium · ● DIGIT "), "{without}");
    }

    /// In the dense layout's narrow status line an open hint beats the
    /// time, and the time beats the level.
    #[test]
    fn the_dense_status_line_keeps_the_hint_then_the_time_then_the_level() {
        fn status(game: &Game, view: &View) -> String {
            top_line(&render_dense(game, view))[TITLE.len()..]
                .trim()
                .to_string()
        }
        let mut game = game().rated(Level::Medium);
        game.next_mode();
        let time = Some(std::time::Duration::from_secs(3725));
        let plain = View {
            time,
            ..View::default()
        };
        assert_eq!(status(&game, &plain), "Medium · 1:02:05 · ● CANDIDATE");
        let hint = View {
            time,
            ..open(&game, 2)
        };
        assert_eq!(status(&game, &hint), "hint 2/5 · ● CANDIDATE");
        game.next_mode();
        game.next_mode();
        let hint = View {
            time,
            ..open(&game, 2)
        };
        assert_eq!(status(&game, &hint), "1:02:05 · hint 2/5 · ● DIGIT");
    }

    /// Hidden notes take their short name before anything is left off, and
    /// what the player turned on outlasts the level and the time.
    #[test]
    fn the_status_line_keeps_what_the_player_turned_on() {
        let game = game().rated(Level::Medium);
        let view = View {
            notes_hidden: true,
            filter: Some(Filter::Digit(5)),
            time: Some(std::time::Duration::from_secs(3725)),
            ..View::default()
        };
        let full = top_line(&render(&game, &view));
        assert!(
            full.contains("Medium · 1:02:05 · notes hidden · filter 5"),
            "{full}"
        );
        let dense = top_line(&render_dense(
            &game,
            &View {
                filter: None,
                ..view.clone()
            },
        ));
        assert!(dense.contains("notes hidden · ● DIGIT"), "{dense}");
        assert!(!dense.contains("Medium"), "{dense}");
    }

    #[test]
    fn clock_text_shows_hours_only_when_there_are_some() {
        let at = |s| clock_text(std::time::Duration::from_secs(s));
        assert_eq!(at(0), "0:00");
        assert_eq!(at(65), "1:05");
        assert_eq!(at(3725), "1:02:05");
    }

    /// Pausing hides the board in both layouts: no digit of it shows, and
    /// no hint.
    #[test]
    fn pausing_hides_the_board() {
        let game = game();
        // A hint open when the game was paused doesn't show either: not its
        // words, its rung in the status line, nor its close button, which
        // takes no click.
        let view = View {
            paused: true,
            ..open(&game, 2)
        };
        for (backend, layout) in [
            (render(&game, &view), "full"),
            (render_dense(&game, &view), "dense"),
        ] {
            let buffer = backend.buffer();
            let text: String = (0..buffer.area.height)
                .flat_map(|y| (0..buffer.area.width).map(move |x| (x, y)))
                .map(|pos| buffer[pos].symbol().to_string())
                .collect();
            assert!(text.contains("Paused"), "{layout}");
            assert!(text.contains("p carry on"), "{layout}");
            assert!(!text.contains('┼'), "{layout}: the board's lines show");
            assert!(!text.contains("53"), "{layout}");
            assert!(!text.contains("Rung"), "{layout}");
            assert!(!text.contains("hint 2/5"), "{layout}");
            assert!(!text.contains('×'), "{layout}");
        }
    }

    /// The panel's text rows, trimmed.
    fn panel_lines(backend: &TestBackend) -> Vec<String> {
        let buffer = backend.buffer();
        let top = BOARD_HEIGHT + 2 + PAD_HEIGHT;
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

    /// The win screen says the rating, the hints and reveals, and the time
    /// only if the timer is on.
    #[test]
    fn the_win_screen_says_what_the_game_took() {
        let mut game = won_game().rated(Level::Hard);
        let lines = panel_lines(&render(&game, &View::default()));
        assert_eq!(lines, ["Solved, Hard.", "No hints.", ""]);
        game.count_hint();
        game.count_hint();
        let timed = View {
            time: Some(std::time::Duration::from_secs(754)),
            ..View::default()
        };
        let lines = panel_lines(&render(&game, &timed));
        assert_eq!(lines, ["Solved, Hard.", "2 hints, 12:34.", ""]);
        let unrated = panel_lines(&render(&won_game(), &View::default()));
        assert_eq!(unrated[0], "Solved.");
    }

    #[test]
    fn a_reveal_shows_on_the_win_screen() {
        let mut game = game();
        go(&mut game, 2);
        game.reveal();
        let solution = solution();
        for cell in 0..81 {
            if game.board().get(cell).is_none() {
                go(&mut game, cell);
                game.press(solution.get(cell).unwrap());
            }
        }
        let lines = panel_lines(&render(&game, &View::default()));
        assert_eq!(lines[1], "No hints, 1 cell revealed.");
    }

    /// Each highlight switched off leaves its cells as they would be
    /// without it.
    #[test]
    fn each_highlight_can_be_switched_off() {
        let theme = Theme::tokyo_night_amoled();
        let off = |change: fn(&mut Highlights)| {
            let mut highlights = Highlights::default();
            change(&mut highlights);
            View {
                highlights,
                ..View::default()
            }
        };
        let bg = |game: &Game, view: &View, cell| render(game, view).buffer()[middle(cell)].bg;

        let mut game = game();
        go(&mut game, 2);
        game.press(5); // clashes with the clue 5 at r1c1
        go(&mut game, 0);
        assert_eq!(bg(&game, &View::default(), 14), theme.same_digit);
        assert_eq!(
            bg(&game, &off(|h| h.same_digit = false), 14),
            theme.background
        );
        assert_eq!(bg(&game, &View::default(), 36), theme.peer);
        assert_eq!(bg(&game, &off(|h| h.peer = false), 36), theme.background);

        let mut notes = self::game();
        notes.next_mode();
        go(&mut notes, 3);
        notes.press(5); // a candidate 5 in a row that holds a 5
        go(&mut notes, 40);
        let clash = |view: &View| render(&notes, view).buffer()[note_at(3, 5)].clone();
        assert_eq!(clash(&View::default()).fg, theme.conflict);
        let quiet = clash(&off(|h| h.note_clashes = false));
        assert_eq!(quiet.fg, theme.candidate);
        assert!(!quiet.modifier.contains(Modifier::REVERSED));

        let mut complete = self::game();
        let solution = solution();
        for cell in 0..81 {
            if solution.get(cell) == Some(5) && !complete.is_clue(cell) {
                go(&mut complete, cell);
                complete.press(5);
            }
        }
        let five = |view: &View| render(&complete, view).buffer()[middle(0)].fg;
        assert_eq!(five(&View::default()), theme.dimmed);
        assert_eq!(five(&off(|h| h.complete_digits = false)), theme.clue);
    }

    /// A game in progress with the longest Continue label there is.
    const PROGRESS: Option<Progress> = Some(Progress {
        level: Some(Level::Medium),
        left: 81,
    });

    fn render_menu(menu: &Menu, progress: Option<Progress>, size: (u16, u16)) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
        let theme = Theme::tokyo_night_amoled();
        let items = menu.items(progress);
        terminal
            .draw(|frame| draw_menu(frame, menu, &items, &theme))
            .unwrap();
        terminal.backend().clone()
    }

    /// The menu takes the biggest logo that fits: 6 rows, 3 rows, or the
    /// name in plain text.
    #[test]
    fn menus_in_three_window_sizes() {
        use crate::menu::Kind;
        let main = Menu::new(Kind::Main);
        insta::assert_snapshot!("menu_wide", render_menu(&main, PROGRESS, (80, 30)));
        insta::assert_snapshot!("menu_phone", render_menu(&main, PROGRESS, (38, 18)));
        let pause = Menu::new(Kind::Pause);
        let tight = menu_needed(&pause, &pause.items(None));
        insta::assert_snapshot!("menu_tight", render_menu(&pause, None, tight));
    }

    fn text_of(backend: &TestBackend) -> String {
        let buffer = backend.buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    + "\n"
            })
            .collect()
    }

    /// Every menu, with and without a game to continue, fits the size its
    /// items ask for, labels and keys whole, and a window a column or a
    /// row smaller is told that size.
    #[test]
    fn a_menu_is_as_big_as_its_items() {
        use crate::menu::Kind;
        let kinds = [
            Kind::Main,
            Kind::Pause,
            Kind::Levels { from_pause: false },
            Kind::Replace {
                level: Level::Hard,
                from_pause: false,
            },
        ];
        for kind in kinds {
            for progress in [None, PROGRESS] {
                let menu = Menu::new(kind);
                let items = menu.items(progress);
                let (width, height) = menu_needed(&menu, &items);
                let text = text_of(&render_menu(&menu, progress, (width, height)));
                let rows: Vec<&str> = text.lines().collect();
                let boxed = MenuAreas::of(Rect::new(0, 0, width, height), &menu, &items).boxed;
                let heading = menu.question().unwrap_or("sumi");
                assert!(text.contains(heading), "{kind:?}: {text}");
                for item in &items {
                    let y = rows.iter().position(|r| r.contains(&item.label()));
                    let y = y.unwrap_or_else(|| panic!("{kind:?}: {text}")) as u16;
                    assert!(boxed.y < y && y + 1 < boxed.bottom(), "{kind:?}: {text}");
                }
                // Nothing to spare: no blank row, and the box or a key row
                // as wide as the window.
                assert!(
                    rows.iter().all(|r| !r.trim().is_empty()),
                    "{kind:?}: {text}"
                );
                let widest = rows.iter().map(|r| r.trim().chars().count()).max().unwrap();
                assert!(
                    boxed.width == width || widest as u16 + 2 == width,
                    "{kind:?}: {text}"
                );
                for (key, words) in menu_keys(&menu) {
                    assert!(text.contains(&format!("{key} {words}")), "{kind:?}: {text}");
                }
                let asked = format!("{width}×{height}");
                for smaller in [(width - 1, height), (width, height - 1)] {
                    let text = text_of(&render_menu(&menu, progress, smaller));
                    assert!(text.contains(&asked), "{kind:?} {smaller:?}: {text}");
                }
            }
        }
    }

    #[test]
    fn the_logo_is_the_same_picture_at_both_scales() {
        let (small, big) = (logo(1), logo(2));
        assert_eq!((small.len(), big.len()), (3, 6));
        let width = |rows: &[String]| rows.iter().map(|r| r.chars().count()).max().unwrap();
        assert_eq!((width(&small), width(&big)), (17, 34));
    }

    fn render_settings(page: &Page, config: &Config, size: (u16, u16)) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
        let theme = Theme::tokyo_night_amoled();
        let shown = SettingsPage {
            page,
            config,
            theme_name: "Tokyo Night AMOLED",
            error: None,
            note: None,
        };
        terminal
            .draw(|frame| draw_settings(frame, &shown, &theme))
            .unwrap();
        terminal.backend().clone()
    }

    #[test]
    fn the_settings_page() {
        let page = Page::new(true);
        insta::assert_snapshot!(render_settings(
            &page,
            &Config::default(),
            settings_needed()
        ));
    }

    /// The settings page's widest row fits the width it asks for.
    #[test]
    fn every_settings_row_fits_the_page() {
        let theme = Theme::tokyo_night_amoled();
        let config = Config {
            marks_to_candidates: crate::config::MarksToCandidates::Merge,
            ..Config::default()
        };
        let (_, inner, _) =
            settings_areas(Rect::new(0, 0, settings_needed().0, settings_needed().1));
        for name in ["Tokyo Night AMOLED", "A user theme with a very long name"] {
            for selected in 0..crate::settings::Setting::all().len() {
                let page = Page {
                    selected,
                    from_pause: false,
                };
                let shown = SettingsPage {
                    page: &page,
                    config: &config,
                    theme_name: name,
                    error: None,
                    note: None,
                };
                for line in settings_lines(&shown, &theme) {
                    assert!(line.width() <= usize::from(inner.width), "{line}");
                }
            }
        }
        assert_eq!(
            shorten("Tokyo Night AMOLED", VALUE_WIDTH),
            "Tokyo Night AMOLED"
        );
        assert_eq!(shorten("abcdef", 4), "abc…");
    }

    /// The key bar says what t does under the setting in force.
    #[test]
    fn the_key_bar_says_what_t_does() {
        let mut game = game();
        game.next_mode();
        game.next_mode();
        let t = |game: &Game| {
            keys(game, &View::default())
                .into_iter()
                .find(|(key, _)| *key == "t")
                .unwrap()
                .1
        };
        assert_eq!(t(&game), "fill, drop marks");
        game.set_candidates_from_marks(CandidatesFromMarks::Merge);
        assert_eq!(t(&game), "add to candidates");
    }

    /// With the key bar hidden, no keys show, except at the filter's prompt.
    #[test]
    fn the_key_bar_can_be_hidden() {
        let game = game();
        let hidden = View {
            key_bar_hidden: true,
            ..View::default()
        };
        assert_eq!(shown_keys(&game, &hidden), Vec::new());
        let prompt = View {
            awaiting_filter: true,
            ..hidden
        };
        assert!(!shown_keys(&game, &prompt).is_empty());
        let bottom: String = (0..FRAME_WIDTH)
            .map(|x| {
                render(
                    &game,
                    &View {
                        key_bar_hidden: true,
                        ..View::default()
                    },
                )
                .buffer()[(x, BOARD_HEIGHT + 1)]
                    .symbol()
                    .to_string()
            })
            .collect();
        assert!(!bottom.contains("quit"), "{bottom}");
        assert!(
            bottom.trim_matches(['╰', '╯']).chars().all(|c| c == '─'),
            "{bottom}"
        );
    }

    /// Every spot on screen where the buffer holds `symbol`.
    fn find(backend: &TestBackend, symbol: &str) -> Vec<(u16, u16)> {
        let area = backend.buffer().area;
        (0..area.height)
            .flat_map(|y| (0..area.width).map(move |x| (x, y)))
            .filter(|&pos| backend.buffer()[pos].symbol() == symbol)
            .collect()
    }

    fn full_area() -> Rect {
        Rect::new(0, 0, NEEDED.0, NEEDED.1)
    }

    fn dense_area() -> Rect {
        Rect::new(0, 0, DENSE.0, DENSE.1)
    }

    /// A click on any cell lands on that cell. In the full layout a click on
    /// a note's spot lands on that note, and the middle is the 5's spot.
    #[test]
    fn a_click_lands_on_the_cell_or_note_drawn_under_it() {
        let view = View::default();
        for cell in 0..81 {
            let (x, y) = middle(cell);
            assert_eq!(
                hit(full_area(), &view, x - 3, y - 1),
                Some(Target::Cell(cell))
            );
            assert_eq!(hit(full_area(), &view, x, y), Some(Target::Note(cell, 5)));
            for digit in 1..=9 {
                let (nx, ny) = note_at(cell, digit);
                assert_eq!(
                    hit(full_area(), &view, nx, ny),
                    Some(Target::Note(cell, digit))
                );
            }
            let (dx, dy) = dense_cell(cell);
            for (ox, oy) in [(0, 0), (1, 1), (2, 2)] {
                assert_eq!(
                    hit(dense_area(), &view, dx + ox, dy + oy),
                    Some(Target::Cell(cell))
                );
            }
        }
        // The lines between cells take no click.
        let (x, y) = middle(40);
        assert_eq!(hit(full_area(), &view, x - 4, y), None);
    }

    /// A click on a digit of the pad, or the count under it, lands on that
    /// digit, in both layouts.
    #[test]
    fn a_click_on_the_pad_lands_on_its_digit() {
        let game = game();
        let view = View::default();
        for (backend, area) in [
            (render(&game, &view), full_area()),
            (render_dense(&game, &view), dense_area()),
        ] {
            // The pad's row: the only one whose text is the nine digits.
            let buffer = backend.buffer();
            let y = (0..buffer.area.height)
                .find(|&y| {
                    let text: String = (0..buffer.area.width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                        .split_whitespace()
                        .collect();
                    text == "123456789"
                })
                .unwrap();
            for digit in 1..=9u8 {
                let (x, _) = *find(&backend, &digit.to_string())
                    .iter()
                    .find(|&&(_, at)| at == y)
                    .unwrap();
                assert_eq!(hit(area, &view, x, y), Some(Target::Pad(digit)), "{digit}");
                assert_eq!(
                    hit(area, &view, x, y + 1),
                    Some(Target::Pad(digit)),
                    "{digit}"
                );
            }
        }
    }

    /// While paused, the hint's close button takes no click.
    #[test]
    fn a_paused_hint_takes_no_click() {
        let game = game();
        let shown = open(&game, 2);
        let x = find(&render(&game, &shown), "×")[0];
        let paused = View {
            paused: true,
            ..shown
        };
        assert_eq!(hit(full_area(), &paused, x.0, x.1), None);
    }

    /// The hint's close button is where × is drawn, and only with a hint open.
    #[test]
    fn a_click_on_the_close_button_closes_the_hint() {
        let game = game();
        let open = open(&game, 2);
        for (backend, area) in [
            (render(&game, &open), full_area()),
            (render_dense(&game, &open), dense_area()),
        ] {
            let spots = find(&backend, "×");
            assert_eq!(spots.len(), 1);
            let (x, y) = spots[0];
            assert_eq!(hit(area, &open, x, y), Some(Target::CloseHint));
            assert_eq!(hit(area, &View::default(), x, y), None);
        }
    }

    /// The key list and the pause screen cover the board and the pad: a
    /// click there lands on nothing.
    #[test]
    fn covered_cells_take_no_clicks() {
        for view in [
            View {
                help: true,
                ..View::default()
            },
            View {
                paused: true,
                ..View::default()
            },
        ] {
            let (x, y) = middle(40);
            assert_eq!(hit(full_area(), &view, x, y), None);
            let (dx, dy) = dense_cell(40);
            assert_eq!(hit(dense_area(), &view, dx, dy), None);
        }
    }

    /// A click on a menu item's label lands on that item, at every logo
    /// size.
    #[test]
    fn a_click_on_a_menu_item_lands_on_it() {
        let menu = Menu::new(crate::menu::Kind::Pause);
        let items = menu.items(None);
        for size in [(80, 30), (38, 18), menu_needed(&menu, &items)] {
            let backend = render_menu(&menu, None, size);
            let area = Rect::new(0, 0, size.0, size.1);
            for (i, item) in items.iter().enumerate() {
                let label = item.label();
                let spot = find(&backend, &label[..1])
                    .into_iter()
                    .find(|&(x, y)| {
                        let rest: String = (x..x + label.len() as u16)
                            .map(|x| backend.buffer()[(x, y)].symbol())
                            .collect();
                        rest == label
                    })
                    .unwrap();
                assert_eq!(
                    menu_hit(area, &menu, &items, spot.0, spot.1),
                    Some(i),
                    "{size:?}"
                );
                // The whole row inside the box takes the click, not only the
                // label.
                let areas = MenuAreas::of(area, &menu, &items);
                let edge = areas.boxed.x + 1;
                assert_eq!(
                    menu_hit(area, &menu, &items, edge, spot.1),
                    Some(i),
                    "{size:?}"
                );
            }
        }
    }

    /// A save error's reason shows, however long the path before it, and
    /// without one the page says when the terminal overrides the theme.
    #[test]
    fn the_settings_page_says_why_a_change_wasnt_saved() {
        let theme = Theme::tokyo_night_amoled();
        let page = Page::new(false);
        let config = Config::default();
        let error = "/home/someone/.config/sumi/config.toml: Permission denied (os error 13)";
        let text = |error, note| {
            let shown = SettingsPage {
                page: &page,
                config: &config,
                theme_name: "Nord",
                error,
                note,
            };
            let mut terminal =
                Terminal::new(TestBackend::new(settings_needed().0, settings_needed().1)).unwrap();
            terminal
                .draw(|frame| draw_settings(frame, &shown, &theme))
                .unwrap();
            let buffer = terminal.backend().buffer().clone();
            (0..buffer.area.height)
                .map(|y| {
                    (0..buffer.area.width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                })
                .map(|row| {
                    row.trim_matches(|c| c == ' ' || c == '│')
                        .trim()
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        let shown = text(Some(error), None);
        assert!(shown.contains("Permission denied"), "{shown}");
        assert!(text(None, Some("This terminal shows 16 colours.")).contains("16 colours"));
    }

    /// The page's rows, the frame and the key bar fit the height it asks
    /// for.
    #[test]
    fn the_settings_page_fits_the_size_it_asks_for() {
        let area = Rect::new(0, 0, settings_needed().0, settings_needed().1);
        let (boxed, inner, keys) = settings_areas(area);
        assert_eq!(inner.height as usize, settings_rows().len() + 2);
        assert!(boxed.bottom() <= keys.y && keys.bottom() <= area.bottom());
        assert_eq!(
            boxed.y + keys.bottom() - area.bottom(),
            boxed.y,
            "no room to spare"
        );
    }

    /// While a completed house pulses, its cells are lit; without colour,
    /// reversed.
    #[test]
    fn a_pulse_lights_the_house() {
        use crate::animation::{Animation, Kind};
        let theme = Theme::tokyo_night_amoled();
        let game = game();
        let view = View {
            animations: vec![Animation::new(Kind::House(sumi_core::grid::House::Row(8)))],
            ..View::default()
        };
        let backend = render(&game, &view);
        assert_eq!(backend.buffer()[middle(76)].bg, theme.same_digit);
        assert_ne!(backend.buffer()[middle(67)].bg, theme.same_digit);
        let plain = render_in(&game, &view, &Theme::monochrome());
        assert!(
            plain.buffer()[middle(76)]
                .modifier
                .contains(Modifier::REVERSED)
        );
    }

    /// Where two animations light a cell, the stronger light shows.
    #[test]
    fn the_strongest_light_wins() {
        use crate::animation::{Animation, Kind};
        use sumi_core::grid::House;
        let theme = Theme::tokyo_night_amoled();
        let faint = Animation {
            kind: Kind::House(House::Row(8)),
            elapsed: std::time::Duration::from_millis(200),
        };
        let strong = Animation::new(Kind::House(House::Col(4)));
        for animations in [vec![faint, strong], vec![strong, faint]] {
            let view = View {
                animations,
                ..View::default()
            };
            let backend = render(&game(), &view);
            assert_eq!(backend.buffer()[middle(76)].bg, theme.same_digit);
            // r9c1 is in the faint row only: the faint light shows there.
            assert_eq!(backend.buffer()[middle(72)].bg, theme.hint_area);
        }
    }

    /// A click on a setting's label lands on that setting.
    #[test]
    fn a_click_on_a_setting_lands_on_it() {
        let page = Page::new(false);
        let backend = render_settings(&page, &Config::default(), settings_needed());
        let area = Rect::new(0, 0, settings_needed().0, settings_needed().1);
        for (i, setting) in Setting::all().into_iter().enumerate() {
            let label = setting.label();
            let spot = find(&backend, &label[..1])
                .into_iter()
                .filter(|&(x, _)| x + label.len() as u16 <= area.width)
                .find(|&(x, y)| {
                    let rest: String = (x..x + label.len() as u16)
                        .map(|x| backend.buffer()[(x, y)].symbol())
                        .collect();
                    rest == label
                })
                .unwrap();
            assert_eq!(settings_hit(area, spot.0, spot.1), Some(i), "{label}");
        }
        let heading = find(&backend, "G")[0];
        assert_eq!(settings_hit(area, heading.0, heading.1), None, "Gameplay");
    }
}
