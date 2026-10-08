//! The interactive game: one loop that draws, waits for an event and hands it
//! to the session.

use std::io;
use std::time::{Duration, Instant};

use rand::rngs::Xoshiro256PlusPlus;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use ratatui::crossterm::execute;
use sumi_core::bank;
use sumi_core::generate::{LevelGenerator, generate_solvable};
use sumi_core::hint::Hint;
use sumi_core::technique::Level;

use crate::config::Config;
use crate::game::Game;
use crate::input::{Action, FilterKey, action, filter_key};
use crate::menu::{Choice, Item, Kind, Menu};
use crate::theme::Theme;
use crate::ui::{self, Filter, OpenHint, View};

/// Plays puzzles of `level`, or opens the main menu without one. Fails
/// without changing the terminal when there is none to draw on.
pub fn run(level: Option<Level>) -> io::Result<()> {
    let (config, theme, warnings) = start();
    let mut terminal = ratatui::try_init()
        .map_err(|err| io::Error::new(err.kind(), format!("can't open the terminal: {err}")))?;
    // With bracketed paste on, pasted text arrives as one event that the game
    // ignores, instead of as keys that could place digits or quit.
    let result = execute!(io::stdout(), EnableBracketedPaste)
        .and_then(|()| play(&mut terminal, level, &config, &theme));
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    // The screen hid them while the game ran.
    for line in warnings {
        eprintln!("sumi: {line}");
    }
    result
}

/// The config, the theme the player asked for if the environment allows it,
/// and a line for each file that couldn't be read.
fn start() -> (Config, Theme, Vec<String>) {
    let dir = Config::dir();
    let (config, warning) = Config::load(dir.as_ref());
    let (themes, warnings) = Theme::load_all(dir.map(|d| d.join("themes")).as_deref());
    let warnings: Vec<String> = warning.into_iter().chain(warnings).collect();
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let truecolor = std::env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit");
    let theme = Theme::choose(&themes, config.theme.as_deref(), no_color, truecolor);
    (config, theme, warnings)
}

fn play(
    terminal: &mut DefaultTerminal,
    level: Option<Level>,
    config: &Config,
    theme: &Theme,
) -> io::Result<()> {
    let mut app = App::new(rand::make_rng(), level, config.clone(), Instant::now);
    loop {
        if let Some(session) = &mut app.session {
            session.view.time = session.time();
        }
        let mut shown = true;
        terminal.draw(|frame| {
            shown = app.fits(frame.area());
            match (&app.screen, &app.session) {
                (Screen::Menu(menu), session) => {
                    ui::draw_menu(frame, menu, &menu.items(session.is_some()), theme);
                }
                (Screen::Game, Some(session)) => {
                    ui::draw(frame, &session.game, &session.view, theme);
                }
                (Screen::Game, None) => unreachable!("the game screen shows a game"),
            }
        })?;
        // A running timer is redrawn as its seconds change; otherwise the
        // loop sleeps until a key arrives.
        let ticking = app.screen == Screen::Game
            && app
                .session
                .as_ref()
                .is_some_and(|s| s.view.time.is_some() && s.clock_running());
        let event = match app.session.as_ref().and_then(|s| s.view.time) {
            Some(time) if ticking => {
                let to_next_second = Duration::from_millis(1000 - u64::from(time.subsec_millis()));
                if !event::poll(to_next_second)? {
                    continue;
                }
                event::read()?
            }
            _ => event::read()?,
        };
        if app.handle(event, shown) == Flow::Quit {
            return Ok(());
        }
    }
}

/// What a key in the game asks of the app around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Quit,
    /// Esc with nothing to close: the pause menu.
    Menu,
    /// Enter after a win: another puzzle of the same level.
    NewGame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Game,
    Menu(Menu),
}

/// The screen showing, the game in progress if there is one, and where new
/// puzzles come from.
struct App {
    screen: Screen,
    session: Option<Session>,
    rng: Xoshiro256PlusPlus,
    /// The level new games are of; `None` is any level sumi can finish.
    level: Option<Level>,
    config: Config,
    /// Where the time comes from; tests set their own.
    clock: fn() -> Instant,
}

impl App {
    /// A game of `level` straight away, or the main menu without one.
    fn new(
        rng: Xoshiro256PlusPlus,
        level: Option<Level>,
        config: Config,
        clock: fn() -> Instant,
    ) -> App {
        let mut app = App {
            screen: Screen::Menu(Menu::new(Kind::Main)),
            session: None,
            rng,
            level,
            config,
            clock,
        };
        if level.is_some() {
            app.start(level);
        }
        app
    }

    /// Whether the screen showing can be drawn in a window of this size.
    fn fits(&self, area: ratatui::layout::Rect) -> bool {
        match self.screen {
            Screen::Game => ui::fits(area),
            Screen::Menu(_) => ui::menu_fits(area),
        }
    }

    /// Starts a game of `level`. Hidden candidates carry over from the last
    /// game; the rest of its view doesn't.
    fn start(&mut self, level: Option<Level>) {
        self.level = level;
        let notes_hidden = self.session.as_ref().is_some_and(|s| s.view.notes_hidden);
        let game = new_game(&mut self.rng, level);
        let mut session = Session::new(game, &self.config, self.clock);
        session.view.notes_hidden = notes_hidden;
        self.session = Some(session);
        self.screen = Screen::Game;
    }

    fn resume(&mut self) {
        if let Some(session) = &mut self.session {
            session.game.start_clock((self.clock)());
        }
        self.screen = Screen::Game;
    }

    fn handle(&mut self, event: Event, shown: bool) -> Flow {
        let Screen::Menu(menu) = &mut self.screen else {
            let session = self.session.as_mut().expect("the game screen shows a game");
            return match session.handle(event, shown) {
                Flow::NewGame => {
                    self.start(self.level);
                    Flow::Continue
                }
                Flow::Menu => {
                    session.game.stop_clock((self.clock)());
                    self.screen = Screen::Menu(Menu::new(Kind::Pause));
                    Flow::Continue
                }
                flow => flow,
            };
        };
        let Some(key) = press(event) else {
            return Flow::Continue;
        };
        let action = action(key);
        if is_ctrl_c(key) || action == Some(Action::Quit) {
            return Flow::Quit;
        }
        let (Some(action), true) = (action, shown) else {
            return Flow::Continue;
        };
        let kind = menu.kind;
        match menu.handle(action, self.session.is_some()) {
            None => {}
            Some(Choice::Back) => match kind {
                Kind::Levels { from_pause: true } => {
                    self.screen = Screen::Menu(Menu::new(Kind::Pause))
                }
                Kind::Levels { from_pause: false } => {
                    self.screen = Screen::Menu(Menu::new(Kind::Main))
                }
                Kind::Pause => self.resume(),
                Kind::Main => {}
            },
            Some(Choice::Chosen(item)) => match item {
                Item::Continue | Item::Resume => self.resume(),
                Item::NewGame => {
                    let from_pause = kind == Kind::Pause;
                    self.screen = Screen::Menu(Menu::new(Kind::Levels { from_pause }));
                }
                Item::Level(level) => self.start(Some(level)),
                Item::MainMenu => self.screen = Screen::Menu(Menu::new(Kind::Main)),
                Item::Quit => return Flow::Quit,
            },
        }
        Flow::Continue
    }
}

/// The key a press event carries. Terminals that report releases and
/// repeats as separate events would otherwise apply every key twice.
fn press(event: Event) -> Option<KeyEvent> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => Some(key),
        _ => None,
    }
}

/// One game: the puzzle and the player's work, and how it is shown.
struct Session {
    game: Game,
    view: View,
    /// Show the time played.
    timer: bool,
    /// Where the time comes from; tests set their own.
    clock: fn() -> Instant,
}

impl Session {
    /// Starts `game`, with its clock running.
    fn new(mut game: Game, config: &Config, clock: fn() -> Instant) -> Session {
        game.start_clock(clock());
        Session {
            game,
            view: View {
                highlights: config.highlights,
                ..View::default()
            },
            timer: config.timer,
            clock,
        }
    }

    /// The time to show, if the timer is on.
    fn time(&self) -> Option<Duration> {
        self.timer.then(|| self.game.elapsed((self.clock)()))
    }

    /// Whether the time played is going up: not paused, and not won.
    fn clock_running(&self) -> bool {
        !self.view.paused && !self.game.is_won()
    }

    /// `board_shown` is false while the window is too small to draw the
    /// board; keys then do nothing but quit, so nothing changes unseen, the
    /// view included.
    fn handle(&mut self, event: Event, board_shown: bool) -> Flow {
        let Some(key) = press(event) else {
            return Flow::Continue;
        };
        if is_ctrl_c(key) {
            return Flow::Quit;
        }
        if !board_shown {
            return if action(key) == Some(Action::Quit) {
                Flow::Quit
            } else {
                Flow::Continue
            };
        }
        let flow = self.dispatch(key);
        if self.game.is_won() {
            self.game.stop_clock((self.clock)());
        }
        // A hint is about the board it was worked out from: any change to
        // the board or the notes closes it, so Enter never applies a step to
        // a board it wasn't found on.
        let revision = self.game.revision();
        if self
            .view
            .hint
            .as_ref()
            .is_some_and(|open| open.revision != revision)
        {
            self.view.hint = None;
        }
        // A check's mark belongs to the digit it found, not to the cell.
        let board = self.game.board();
        self.view
            .checked
            .retain(|&(cell, digit)| board.get(cell) == Some(digit));
        flow
    }

    fn dispatch(&mut self, key: KeyEvent) -> Flow {
        self.view.notice = None;
        if self.view.awaiting_filter {
            self.view.awaiting_filter = false;
            match filter_key(key) {
                Some(FilterKey::Digit(d)) => self.view.filter = Some(Filter::Digit(d)),
                Some(FilterKey::TwoCandidates) => self.view.filter = Some(Filter::TwoCandidates),
                Some(FilterKey::Off) => self.view.filter = None,
                None => {}
            }
            return Flow::Continue;
        }

        let Some(action) = action(key) else {
            return Flow::Continue;
        };
        // The key list and the pause screen cover the board, so nothing
        // under them acts.
        if self.view.help {
            match action {
                Action::Quit => return Flow::Quit,
                Action::Help | Action::Close => self.view.help = false,
                _ => {}
            }
            return Flow::Continue;
        }
        if self.view.paused {
            match action {
                Action::Quit => return Flow::Quit,
                Action::Help => self.view.help = true,
                Action::Pause => {
                    self.view.paused = false;
                    self.game.start_clock((self.clock)());
                }
                _ => {}
            }
            return Flow::Continue;
        }
        let game = &mut self.game;
        match action {
            Action::Quit => return Flow::Quit,
            Action::Help => self.view.help = true,
            Action::Confirm if game.is_won() => return Flow::NewGame,
            // Once solved, the board and the view stay as they are until the
            // next game: Enter and quitting are the only keys that act.
            _ if game.is_won() => {}
            Action::Pause => {
                self.view.paused = true;
                game.stop_clock((self.clock)());
            }
            Action::Step(direction) => game.step(direction),
            Action::Jump(direction) => game.jump(direction),
            Action::Digit(digit) => game.press(digit),
            Action::Clear => game.clear(),
            Action::NextMode => game.next_mode(),
            Action::FillCandidates => game.fill_candidates(),
            Action::RefillCandidates => game.refill_candidates(),
            Action::MarksToCandidates => game.marks_to_candidates(),
            Action::ClearLayer => game.clear_notes(Some(game.current_layer())),
            Action::ClearAllNotes => game.clear_notes(None),
            Action::ToggleNotes => self.view.notes_hidden = !self.view.notes_hidden,
            Action::Filter => self.view.awaiting_filter = true,
            Action::Undo => game.undo(),
            Action::Redo => game.redo(),
            Action::Hint => self.climb_hint(),
            // Esc closes what is open, then clears check marks, then opens
            // the pause menu.
            Action::Close => {
                if self.view.hint.take().is_none() {
                    if self.view.checked.is_empty() {
                        return Flow::Menu;
                    }
                    self.view.checked.clear();
                }
            }
            Action::Check => {
                self.view.checked = game.wrong_digits();
                self.view.notice = self.view.checked.is_empty().then_some("no mistakes");
            }
            Action::Reveal => game.reveal(),
            Action::Confirm => self.apply_hint(),
        }
        Flow::Continue
    }
}

impl Session {
    /// Opens a hint at the game's starting rung, or moves an open one up a
    /// rung, stopping at the last.
    fn climb_hint(&mut self) {
        match &mut self.view.hint {
            Some(open) => open.rung = (open.rung + 1).min(open.hint.rungs()),
            None => {
                self.game.count_hint();
                let hint = self.game.hint();
                let rung = self.game.hint_start().clamp(1, hint.rungs());
                self.view.hint = Some(OpenHint {
                    hint,
                    rung,
                    revision: self.game.revision(),
                });
            }
        }
    }

    /// At the open hint's last rung, does its step, or takes the cursor to
    /// the mistake it names.
    fn apply_hint(&mut self) {
        let Some(open) = self
            .view
            .hint
            .as_ref()
            .filter(|open| open.rung == open.hint.rungs())
        else {
            return;
        };
        match &open.hint {
            Hint::Mistake(cell) | Hint::StruckOut(cell) => self.game.move_to(*cell),
            hint => {
                if let Some(step) = hint.step().cloned() {
                    self.game.apply_hint(&step);
                    self.view.hint = None;
                }
            }
        }
    }
}

/// A game of `level`, or of any level sumi can finish, with its rating.
/// Easy and Medium puzzles are generated on the spot; Hard and Expert ones
/// come from the bank, which says why.
fn new_game(rng: &mut Xoshiro256PlusPlus, level: Option<Level>) -> Game {
    let (puzzle, level) = match level {
        None => {
            let (puzzle, grade) = generate_solvable(rng);
            (puzzle, grade.level)
        }
        Some(level @ (Level::Hard | Level::Expert)) => {
            let puzzle = bank::pick(rng, level).expect("the bank holds Hard and Expert puzzles");
            (puzzle, level)
        }
        Some(level) => (LevelGenerator::new(level).next(rng).0, level),
    };
    Game::new(puzzle).rated(level)
}

fn is_ctrl_c(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use rand::SeedableRng;
    use sumi_core::grade::grade;

    fn session() -> Session {
        Session::new(fixtures::game(), &Config::default(), now)
    }

    thread_local! {
        static NOW: std::cell::Cell<Instant> = std::cell::Cell::new(Instant::now());
    }

    /// The clock tests run on: it moves only when `advance` moves it.
    fn now() -> Instant {
        NOW.with(std::cell::Cell::get)
    }

    fn advance(seconds: u64) {
        NOW.with(|now| now.set(now.get() + Duration::from_secs(seconds)));
    }

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    /// p pauses: the board is hidden, the clock stops, and keys meant for
    /// the board do nothing until p or Space resumes.
    #[test]
    fn pausing_hides_the_board_and_stops_the_clock() {
        let mut s = session();
        type_keys(&mut s, "ll"); // r1c3, empty
        advance(10);
        type_keys(&mut s, "p");
        assert!(s.view.paused);
        advance(30);
        let before = (s.game.revision(), s.game.cursor());
        type_keys(&mut s, "4j?");
        assert_eq!((s.game.revision(), s.game.cursor()), before);
        assert_eq!(rung(&s), None);
        type_keys(&mut s, " ");
        assert!(!s.view.paused);
        advance(5);
        assert_eq!(s.game.elapsed(now()), secs(15));
        assert_eq!(type_keys(&mut s, "pq"), Flow::Quit, "q quits while paused");
    }

    #[test]
    fn the_clock_stops_at_the_win_and_a_new_game_starts_it_again() {
        let mut app = app_with(fixtures::game());
        advance(7);
        win(game_of(&mut app));
        advance(100);
        assert_eq!(game_of(&mut app).game.elapsed(now()), secs(7));
        app.handle(key(KeyCode::Enter), true);
        advance(3);
        assert_eq!(game_of(&mut app).game.elapsed(now()), secs(3));
    }

    /// The time is there to show only with the timer on.
    #[test]
    fn the_time_shows_only_with_the_timer_on() {
        let mut s = session();
        advance(4);
        assert_eq!(s.time(), None);
        s.timer = true;
        assert_eq!(s.time(), Some(secs(4)));
    }

    /// Each hint opened counts once, however many rungs it is climbed.
    #[test]
    fn each_hint_opened_counts_once() {
        let mut s = session();
        type_keys(&mut s, "???");
        assert_eq!(s.game.hints_used(), 1);
        s.handle(key(KeyCode::Esc), true);
        type_keys(&mut s, "?");
        assert_eq!(s.game.hints_used(), 2);
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn ctrl(c: char) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
    }

    fn type_keys(s: &mut Session, keys: &str) -> Flow {
        for c in keys.chars() {
            if s.handle(key(KeyCode::Char(c)), true) == Flow::Quit {
                return Flow::Quit;
            }
        }
        Flow::Continue
    }

    #[test]
    fn ctrl_c_quits_even_while_waiting_for_the_filter_key() {
        let mut s = session();
        type_keys(&mut s, "f");
        assert_eq!(s.handle(ctrl('c'), true), Flow::Quit);
    }

    #[test]
    fn the_key_after_f_sets_the_filter_and_is_used_up() {
        let mut s = session();
        type_keys(&mut s, "f4");
        assert_eq!(s.view.filter, Some(Filter::Digit(4)));
        type_keys(&mut s, "fb");
        assert_eq!(s.view.filter, Some(Filter::TwoCandidates));
        type_keys(&mut s, "fl"); // not a filter key: cancels, doesn't move
        assert_eq!(s.game.cursor(), 0);
        type_keys(&mut s, "f0");
        assert_eq!(s.view.filter, None);
    }

    #[test]
    fn keys_do_nothing_but_quit_while_the_board_is_hidden() {
        let mut s = session();
        for c in "lla4".chars() {
            s.handle(key(KeyCode::Char(c)), false);
        }
        assert_eq!(s.game.cursor(), 0);
        assert_eq!(s.game.candidates(2), sumi_core::digits::Digits::NONE);
        assert_eq!(s.handle(key(KeyCode::Char('q')), false), Flow::Quit);
    }

    fn rung(s: &Session) -> Option<usize> {
        s.view.hint.as_ref().map(|open| open.rung)
    }

    #[test]
    fn the_hint_key_opens_the_ladder_and_climbs_a_rung_a_press() {
        let mut s = session();
        assert_eq!(rung(&s), None);
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(1));
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(2));
        type_keys(&mut s, "????");
        assert_eq!(rung(&s), Some(5), "the top rung is the last");
    }

    #[test]
    fn moving_leaves_the_hint_open_and_typing_or_esc_closes_it() {
        let mut s = session();
        type_keys(&mut s, "?ljHn");
        assert_eq!(rung(&s), Some(1), "moving and changing mode keep it");
        s.handle(key(KeyCode::Esc), true);
        assert_eq!(rung(&s), None);

        type_keys(&mut s, "?4");
        assert_eq!(rung(&s), None, "a digit closes it");
        type_keys(&mut s, "?u");
        assert_eq!(rung(&s), None, "so does undo");
        type_keys(&mut s, "?a");
        assert_eq!(rung(&s), None, "and fill");
    }

    #[test]
    fn a_closed_hint_starts_again_at_the_first_rung() {
        let mut s = session();
        type_keys(&mut s, "???");
        s.handle(key(KeyCode::Esc), true);
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(1));
    }

    #[test]
    fn enter_applies_the_hint_at_the_last_rung_and_not_before() {
        let mut s = session();
        type_keys(&mut s, "??");
        let before = s.game.board().filled();
        s.handle(key(KeyCode::Enter), true);
        assert_eq!(s.game.board().filled(), before);
        assert_eq!(rung(&s), Some(2));

        type_keys(&mut s, "???");
        assert_eq!(rung(&s), Some(5));
        s.handle(key(KeyCode::Enter), true);
        assert_eq!(s.game.board().filled(), before + 1);
        assert_eq!(rung(&s), None);
    }

    #[test]
    fn hint_start_skips_the_early_rungs() {
        let mut s = session();
        s.game.set_hint_start(3);
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(3));
        s.handle(key(KeyCode::Esc), true);
        s.game.set_hint_start(5);
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(5));
    }

    #[test]
    fn a_short_ladder_opens_at_its_last_rung_when_hint_start_is_past_it() {
        let mut s = session();
        s.game.set_hint_start(5);
        type_keys(&mut s, "ll"); // r1c3
        type_keys(&mut s, "1"); // a wrong digit: the answer is 4
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(3));
    }

    /// `i` opens the list of keys. While it shows, keys meant for the board
    /// do nothing; `i` or Esc closes it, and q still quits.
    #[test]
    fn the_key_list_opens_and_keys_under_it_do_nothing() {
        let mut s = session();
        type_keys(&mut s, "ll"); // r1c3, empty
        type_keys(&mut s, "i");
        assert!(s.view.help);
        let before = (s.game.revision(), s.game.cursor());
        type_keys(&mut s, "4ja?f");
        assert_eq!((s.game.revision(), s.game.cursor()), before);
        assert_eq!(rung(&s), None);
        assert!(!s.view.awaiting_filter);
        type_keys(&mut s, "i");
        assert!(!s.view.help);
        type_keys(&mut s, "i");
        s.handle(key(KeyCode::Esc), true);
        assert!(!s.view.help);
        assert_eq!(type_keys(&mut s, "iq"), Flow::Quit);
    }

    /// The won screen offers the key list, so `i` opens it there too.
    #[test]
    fn the_key_list_opens_after_a_win() {
        let mut s = session();
        let solution = fixtures::solution();
        for cell in 0..81 {
            if !s.game.is_clue(cell) {
                s.game.move_to(cell);
                s.game.press(solution.get(cell).unwrap());
            }
        }
        assert!(s.game.is_won());
        type_keys(&mut s, "i");
        assert!(s.view.help);
    }

    /// Esc with the key list over a hint closes only the list.
    #[test]
    fn esc_closes_the_key_list_before_the_hint() {
        let mut s = session();
        type_keys(&mut s, "??i");
        s.handle(key(KeyCode::Esc), true);
        assert!(!s.view.help);
        assert_eq!(rung(&s), Some(2));
    }

    /// A mistake's last rung names the cell, and Enter takes the cursor there
    /// with the hint still open; before the last rung Enter does nothing.
    #[test]
    fn enter_on_a_mistakes_last_rung_goes_to_its_cell() {
        let mut s = session();
        type_keys(&mut s, "ll1jj??"); // a wrong 1 at r1c3, then away
        s.handle(key(KeyCode::Enter), true);
        assert_eq!(s.game.cursor(), 20);
        type_keys(&mut s, "?");
        assert_eq!(rung(&s), Some(3));
        s.handle(key(KeyCode::Enter), true);
        assert_eq!(s.game.cursor(), 2);
        assert_eq!(rung(&s), Some(3));
    }

    #[test]
    fn check_marks_wrong_digits_and_says_when_there_are_none() {
        let mut s = session();
        type_keys(&mut s, "c");
        assert_eq!(s.view.notice, Some("no mistakes"));
        assert!(s.view.checked.is_empty());

        type_keys(&mut s, "ll1"); // r1c3 gets a 1; its answer is 4
        type_keys(&mut s, "c");
        assert_eq!(s.view.notice, None);
        assert_eq!(s.view.checked, [(2, 1)]);

        s.handle(key(KeyCode::Esc), true);
        assert!(s.view.checked.is_empty(), "Esc clears the marks");
    }

    #[test]
    fn reveal_writes_the_answer_closes_the_hint_and_undoes() {
        let mut s = session();
        type_keys(&mut s, "ll?r");
        assert_eq!(s.game.board().get(2), Some(4));
        assert_eq!(rung(&s), None);
        type_keys(&mut s, "u");
        assert_eq!(s.game.board().get(2), None);
    }

    #[test]
    fn nothing_but_quit_acts_while_the_board_is_hidden_not_even_the_view() {
        let mut s = session();
        type_keys(&mut s, "f");
        s.handle(key(KeyCode::Char('3')), false);
        assert_eq!(s.view.filter, None, "the filter key was not seen");

        let mut s = session();
        type_keys(&mut s, "?c");
        let before = s.view.clone();
        for c in "1u?a".chars() {
            s.handle(key(KeyCode::Char(c)), false);
        }
        s.handle(key(KeyCode::Esc), false);
        assert_eq!(s.view, before);
        assert_eq!(s.handle(key(KeyCode::Char('q')), false), Flow::Quit);
    }

    #[test]
    fn looking_around_filtering_and_checking_leave_a_hint_open() {
        for keys in ["v", "f3", "c", "n", "H"] {
            let mut s = session();
            type_keys(&mut s, "?");
            type_keys(&mut s, keys);
            assert_eq!(rung(&s), Some(1), "{keys}");
        }
    }

    #[test]
    fn esc_closes_the_hint_before_it_clears_the_check_marks() {
        let mut s = session();
        type_keys(&mut s, "ll1c?");
        s.handle(key(KeyCode::Esc), true);
        assert_eq!(rung(&s), None);
        assert_eq!(s.view.checked, [(2, 1)]);
        s.handle(key(KeyCode::Esc), true);
        assert!(s.view.checked.is_empty());
    }

    #[test]
    fn a_notice_goes_with_the_next_key_even_one_that_does_nothing() {
        let mut s = session();
        type_keys(&mut s, "c");
        assert_eq!(s.view.notice, Some("no mistakes"));
        type_keys(&mut s, "z");
        assert_eq!(s.view.notice, None);
    }

    #[test]
    fn a_check_mark_goes_when_its_digit_does_and_stays_gone() {
        let mut s = session();
        type_keys(&mut s, "ll1c"); // r1c3 = 1, wrong
        type_keys(&mut s, "4"); // right
        assert!(s.view.checked.is_empty());
        type_keys(&mut s, "1"); // wrong again, but nobody checked this one
        assert!(s.view.checked.is_empty());
    }

    #[test]
    fn a_new_game_starts_with_a_clean_view_but_keeps_hidden_candidates_and_switches() {
        let config = Config {
            highlights: crate::config::Highlights {
                peer: false,
                ..Default::default()
            },
            ..Config::default()
        };
        let mut app = App::new(Xoshiro256PlusPlus::seed_from_u64(1), None, config, now);
        app.session = Some(Session::new(fixtures::game(), &app.config, now));
        app.screen = Screen::Game;
        let s = game_of(&mut app);
        type_keys(s, "v");
        type_keys(s, "ll1c");
        type_keys(s, "f3");
        type_keys(s, "?");
        win(s);
        app.handle(key(KeyCode::Enter), true);
        assert_eq!(
            game_of(&mut app).view,
            View {
                notes_hidden: true,
                highlights: crate::config::Highlights {
                    peer: false,
                    ..Default::default()
                },
                ..View::default()
            }
        );
    }

    /// What closes a hint is the game changing, however it changed, not
    /// which key was pressed.
    #[test]
    fn a_hint_closes_when_the_game_changes_and_only_then() {
        let mut s = session();
        type_keys(&mut s, "?");
        s.game.press(5); // r1c1 is a clue: nothing changes
        type_keys(&mut s, "l");
        assert_eq!(rung(&s), Some(1));
        s.game.press(4); // r1c2 is a clue too; move to r1c3, which is empty
        type_keys(&mut s, "l");
        s.game.press(4);
        type_keys(&mut s, "l");
        assert_eq!(rung(&s), None, "a change made any way closes it");
    }

    #[test]
    fn pasted_text_is_ignored() {
        let mut s = session();
        assert_eq!(s.handle(Event::Paste("ll4q".into()), true), Flow::Continue);
        assert_eq!(s.game.cursor(), 0);
    }

    #[test]
    fn releases_do_not_act() {
        let mut s = session();
        let mut release = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        s.handle(Event::Key(release), true);
        assert_eq!(s.game.cursor(), 0);
    }

    /// Fills in the fixture puzzle's solution with keys.
    fn win(s: &mut Session) {
        let solution = fixtures::solution();
        let empty: Vec<usize> = (0..81).filter(|&c| !s.game.is_clue(c)).collect();
        for cell in empty {
            while s.game.cursor() != cell {
                let step = if cell % 9 == s.game.cursor() % 9 {
                    "j"
                } else {
                    "l"
                };
                type_keys(s, step);
            }
            let digit = solution.get(cell).unwrap();
            type_keys(s, &digit.to_string());
        }
        assert!(s.game.is_won());
    }

    #[test]
    fn after_a_win_only_enter_and_quit_act_and_hiding_notes_carries_over() {
        let mut s = session();
        type_keys(&mut s, "v");
        win(&mut s);

        type_keys(&mut s, "uhf3");
        assert!(s.game.is_won());
        assert_eq!(s.view.filter, None);

        assert_eq!(s.handle(key(KeyCode::Enter), true), Flow::NewGame);
    }

    #[test]
    fn every_game_is_of_the_level_asked_for() {
        for level in [Level::Medium, Level::Hard] {
            let mut app = App::new(
                Xoshiro256PlusPlus::seed_from_u64(2),
                Some(level),
                Config::default(),
                now,
            );
            let s = game_of(&mut app);
            assert_eq!(s.game.level(), Some(level));
            assert_eq!(grade(s.game.board()).unwrap().level, level);
            s.game = fixtures::game();
            win(s);
            app.handle(key(KeyCode::Enter), true);
            let s = game_of(&mut app);
            assert_eq!(s.game.level(), Some(level));
            assert_eq!(grade(s.game.board()).unwrap().level, level);
        }
    }

    /// An app playing `game`.
    fn app_with(game: Game) -> App {
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(1),
            None,
            Config::default(),
            now,
        );
        app.session = Some(Session::new(game, &app.config, now));
        app.screen = Screen::Game;
        app
    }

    fn game_of(app: &mut App) -> &mut Session {
        app.session.as_mut().unwrap()
    }

    fn press_keys(app: &mut App, keys: &[KeyCode]) -> Flow {
        for &code in keys {
            if app.handle(key(code), true) == Flow::Quit {
                return Flow::Quit;
            }
        }
        Flow::Continue
    }

    fn menu_kind(app: &App) -> Option<Kind> {
        match &app.screen {
            Screen::Menu(menu) => Some(menu.kind),
            Screen::Game => None,
        }
    }

    /// Bare sumi opens the main menu with no game; New game and a level
    /// start one, Medium unless another is picked.
    #[test]
    fn the_main_menu_starts_a_game_of_the_level_picked() {
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(3),
            None,
            Config::default(),
            now,
        );
        assert_eq!(menu_kind(&app), Some(Kind::Main));
        assert!(app.session.is_none());
        press_keys(&mut app, &[KeyCode::Enter]);
        assert_eq!(menu_kind(&app), Some(Kind::Levels { from_pause: false }));
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Main));
        press_keys(
            &mut app,
            &[KeyCode::Enter, KeyCode::Char('k'), KeyCode::Enter],
        );
        assert_eq!(app.screen, Screen::Game);
        assert_eq!(game_of(&mut app).game.level(), Some(Level::Easy));
    }

    /// Esc with nothing open opens the pause menu and stops the clock; Esc
    /// or Resume goes back to the game and starts it again.
    #[test]
    fn esc_opens_the_pause_menu_which_stops_the_clock() {
        let mut app = app_with(fixtures::game());
        advance(5);
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Pause));
        advance(60);
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(app.screen, Screen::Game);
        advance(1);
        press_keys(&mut app, &[KeyCode::Esc, KeyCode::Enter]);
        assert_eq!(app.screen, Screen::Game, "Resume is first");
        assert_eq!(game_of(&mut app).game.elapsed(now()), secs(6));
    }

    /// From the pause menu, the main menu offers Continue, which goes back
    /// to the same game.
    #[test]
    fn the_main_menu_continues_the_game_in_progress() {
        let mut app = app_with(fixtures::game());
        type_keys(game_of(&mut app), "ll4");
        let revision = game_of(&mut app).game.revision();
        press_keys(
            &mut app,
            &[KeyCode::Esc, KeyCode::Char('k'), KeyCode::Enter],
        );
        assert_eq!(menu_kind(&app), Some(Kind::Main));
        press_keys(&mut app, &[KeyCode::Enter]);
        assert_eq!(app.screen, Screen::Game);
        assert_eq!(game_of(&mut app).game.revision(), revision);
    }

    /// New game from the pause menu lists the levels; Esc goes back to the
    /// pause menu, not the main one.
    #[test]
    fn the_level_list_goes_back_where_it_came_from() {
        let mut app = app_with(fixtures::game());
        press_keys(
            &mut app,
            &[KeyCode::Esc, KeyCode::Char('j'), KeyCode::Enter],
        );
        assert_eq!(menu_kind(&app), Some(Kind::Levels { from_pause: true }));
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Pause));
    }

    #[test]
    fn q_quits_from_any_menu_and_keys_do_nothing_in_a_window_too_small() {
        let mut app = app_with(fixtures::game());
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(app.handle(key(KeyCode::Enter), false), Flow::Continue);
        assert_eq!(menu_kind(&app), Some(Kind::Pause));
        assert_eq!(press_keys(&mut app, &[KeyCode::Char('q')]), Flow::Quit);
    }
}
