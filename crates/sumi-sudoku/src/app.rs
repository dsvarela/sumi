//! The interactive game: one loop that draws, waits for an event and hands it
//! to the session.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use rand::rngs::Xoshiro256PlusPlus;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::Rect;
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use sumi_core::bank;
use sumi_core::generate::LevelGenerator;
use sumi_core::grid::{CELLS, House};
use sumi_core::hint::Hint;
use sumi_core::technique::Level;

use crate::animation::{Animation, Kind as AnimationKind, complete_houses};
use crate::config::{Config, MarksToCandidates};
use crate::game::{CandidatesFromMarks, Game, Mode};
use crate::input::{Action, FilterKey, action, filter_key};
use crate::menu::{Choice, Item, Kind, Menu, Progress};
use crate::settings::{Choice as SettingsChoice, Page, Setting};
use crate::store;
use crate::theme::{self, Theme};
use crate::ui::{self, Filter, OpenHint, Target, View};

/// What sumi opens on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    Menu,
    /// A new game of a level.
    Level(Level),
    /// The saved game.
    Continue,
}

/// Opens on `start`. Fails without changing the terminal when there is
/// none to draw on, or no saved game to continue.
pub fn run(start: Start) -> io::Result<()> {
    // Drawing into a file would leave the terminal raw and blank.
    if !io::stdout().is_terminal() {
        return Err(io::Error::other(
            "can't draw: standard output isn't a terminal",
        ));
    }
    let (config, setup, saved, warnings) = setup();
    if start == Start::Continue && saved.is_none() {
        for line in &warnings {
            eprintln!("sumi: {line}");
        }
        return Err(io::Error::other("there's no saved game to continue"));
    }
    let mut terminal = ratatui::try_init()
        .map_err(|err| io::Error::new(err.kind(), format!("can't open the terminal: {err}")))?;
    // ratatui's own hook leaves raw mode and the alternate screen; this one
    // first turns off what sumi turned on.
    let restore = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableBracketedPaste);
        let _ = mouse(false);
        restore(info);
    }));
    let mut app = App::new(rand::make_rng(), start, saved, config, setup, Instant::now);
    app.warnings = warnings;
    if let Some(dir) = app.setup.data.clone() {
        let slot = Arc::clone(&app.slot);
        let signals = Signals::new([SIGTERM, SIGHUP, SIGINT])?;
        std::thread::spawn(move || on_signals(signals, &slot, &dir));
    }
    // With bracketed paste on, pasted text arrives as one event that the game
    // ignores, instead of as keys that could place digits or quit.
    let result =
        execute!(io::stdout(), EnableBracketedPaste).and_then(|()| play(&mut terminal, &mut app));
    app.save(true);
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    let _ = mouse(false);
    ratatui::restore();
    // The screen hid them while the game ran.
    for line in &app.warnings {
        eprintln!("sumi: {line}");
    }
    result
}

/// Saves the game again and puts the terminal back when sumi is told to
/// stop, as when its terminal closes, then stops as the signal asks. The
/// main loop can't do it: a closed terminal can leave it reading nothing
/// forever.
fn on_signals(mut signals: Signals, slot: &Slot, dir: &Path) {
    if let Some(signal) = signals.forever().next() {
        // The lock is kept, so the loop writes nothing more.
        let slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(game) = &*slot {
            let _ = store::save_game(dir, &game.to_json(Instant::now()));
        }
        let _ = execute!(io::stdout(), DisableBracketedPaste);
        let _ = mouse(false);
        ratatui::restore();
        let _ = signal_hook::low_level::emulate_default_handler(signal);
        std::process::exit(128 + signal);
    }
}

/// The game in progress as last saved, shared with the thread that saves
/// it again on a signal. The file is written under its lock, so the two
/// never write it at once. `None` once there is nothing to save.
type Slot = Arc<Mutex<Option<Game>>>;

/// Turns the terminal's mouse reporting on or off: presses and releases
/// only (1000), in SGR form (1006). Unlike crossterm's EnableMouseCapture it
/// leaves motion unreported, which sumi doesn't use and which would wake
/// the loop on every move. Off turns every mode off, whatever turned it on.
fn mouse(on: bool) -> io::Result<()> {
    let codes = if on {
        "\x1b[?1000h\x1b[?1006h"
    } else {
        "\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1015l\x1b[?1006l"
    };
    let mut out = io::stdout();
    out.write_all(codes.as_bytes())?;
    out.flush()
}

/// What the app needs from outside it: where the config and the saved
/// game live, the themes found, and what the terminal allows.
struct Setup {
    dir: Option<PathBuf>,
    data: Option<PathBuf>,
    themes: Vec<(String, Theme)>,
    no_color: bool,
    truecolor: bool,
}

/// The config, the setup, the saved game, and a line for each file that
/// couldn't be read.
fn setup() -> (Config, Setup, Option<Game>, Vec<String>) {
    let dir = Config::dir();
    let data = store::dir();
    let (config, warning) = Config::load(dir.as_ref());
    let (themes, warnings) = Theme::load_all(dir.as_ref().map(|d| d.join("themes")).as_deref());
    let mut warnings: Vec<String> = warning.into_iter().chain(warnings).collect();
    if let Some(id) = &config.theme
        && !themes.iter().any(|(known, _)| known == id)
    {
        warnings.push(format!(
            "theme {id:?} isn't a theme; using {}",
            theme::DEFAULT
        ));
    }
    let saved = data
        .as_deref()
        .map(store::load_game)
        .transpose()
        .unwrap_or_else(|err| {
            warnings.push(format!("couldn't read the saved game: {err}"));
            None
        })
        .flatten();
    let setup = Setup {
        dir,
        data,
        themes,
        no_color: std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()),
        truecolor: std::env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit"),
    };
    (config, setup, saved, warnings)
}

fn play(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    let mut capturing = false;
    loop {
        // Mouse reporting follows the setting. With it on, the scroll wheel
        // arrives as mouse events, which do nothing, rather than as the
        // arrow keys terminals send for it in a full-screen program.
        if app.config.mouse != capturing {
            capturing = app.config.mouse;
            mouse(capturing)?;
        }
        if let Some(session) = &mut app.session {
            session.view.time = session.time();
            session.tick(Instant::now());
        }
        let mut shown = true;
        terminal.draw(|frame| {
            app.area = frame.area();
            shown = app.fits(frame.area());
            let theme = &app.theme;
            let progress = app.progress();
            match (&app.screen, &app.session) {
                (Screen::Menu(menu), _) => {
                    ui::draw_menu(frame, menu, &menu.items(progress), theme);
                }
                (Screen::Settings(page), _) => {
                    let page = ui::SettingsPage {
                        page,
                        config: &app.config,
                        theme_name: &app.theme_name(),
                        error: app.save_error.as_deref(),
                        note: app.theme_note(),
                    };
                    ui::draw_settings(frame, &page, theme);
                }
                (Screen::Game, Some(session)) => {
                    ui::draw(frame, &session.game, &session.view, theme);
                }
                (Screen::Game, None) => unreachable!("the game screen shows a game"),
            }
        })?;
        if app.screen == Screen::Game
            && let Some(session) = &mut app.session
        {
            session.set_shown(shown);
        }
        // A running timer is redrawn as its seconds change; otherwise the
        // loop sleeps until a key arrives.
        let ticking = app.screen == Screen::Game
            && app
                .session
                .as_ref()
                .is_some_and(|s| s.view.time.is_some() && s.game.clock_is_running());
        let animating = app.screen == Screen::Game
            && app
                .session
                .as_ref()
                .is_some_and(|s| !s.view.animations.is_empty());
        let event = match app.session.as_ref().and_then(|s| s.view.time) {
            // About sixty frames a second while something moves.
            _ if animating => {
                if !event::poll(Duration::from_millis(16))? {
                    continue;
                }
                event::read()?
            }
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
    Settings(Page),
}

/// The screen showing, the game in progress if there is one, and where new
/// puzzles come from.
struct App {
    screen: Screen,
    session: Option<Session>,
    rng: Xoshiro256PlusPlus,
    /// The level Enter starts after a win: the last one asked for, Medium
    /// until one is.
    level: Level,
    config: Config,
    setup: Setup,
    /// The theme drawn with: the config's, unless the terminal overrides it.
    theme: Theme,
    /// Where the time comes from; tests set their own.
    clock: fn() -> Instant,
    /// The window's size when it was last drawn, which clicks are read
    /// against.
    area: Rect,
    /// Lines for stderr once the screen is back, and the last setting that
    /// couldn't be saved.
    warnings: Vec<String>,
    save_error: Option<String>,
    /// What the saved game was last written from, so a key that changes
    /// nothing kept writes nothing.
    saved: Option<Saved>,
    slot: Slot,
}

/// What a saved game holds that a key can change, short of the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Saved {
    revision: u64,
    cursor: sumi_core::grid::Cell,
    mode: Mode,
    hints_used: u32,
    clock_running: bool,
}

impl App {
    /// Opens on `start`. `saved`, the saved game, is the game in progress
    /// until another starts.
    fn new(
        rng: Xoshiro256PlusPlus,
        start: Start,
        saved: Option<Game>,
        config: Config,
        setup: Setup,
        clock: fn() -> Instant,
    ) -> App {
        let theme = Theme::choose(
            &setup.themes,
            config.theme.as_deref(),
            setup.no_color,
            setup.truecolor,
        );
        let mut app = App {
            screen: Screen::Menu(Menu::new(Kind::Main)),
            session: None,
            rng,
            level: saved
                .as_ref()
                .and_then(Game::level)
                .unwrap_or(Level::Medium),
            config,
            setup,
            theme,
            clock,
            area: Rect::default(),
            warnings: Vec::new(),
            save_error: None,
            saved: None,
            slot: Slot::default(),
        };
        if let Some(game) = saved {
            let mut session = Session::new(game, &app.config, clock);
            session.game.stop_clock(clock());
            session.show_last_move();
            app.saved = Some(session.saved());
            *app.slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(session.game.clone());
            app.session = Some(session);
        }
        match start {
            Start::Menu => {}
            Start::Level(level) => app.start(level),
            Start::Continue => app.resume(),
        }
        app.save(false);
        app
    }

    /// The game there is to continue, one started and not won, as the
    /// main menu shows it.
    fn progress(&self) -> Option<Progress> {
        let game = &self.session.as_ref()?.game;
        (!game.is_won()).then(|| Progress {
            level: game.level(),
            left: CELLS - game.board().filled(),
        })
    }

    /// Writes the game in progress to the saved game, if anything kept has
    /// changed since it was last written, or with `always`, as on quitting,
    /// which keeps the time. A won game's save is removed. A failure is
    /// shown on the status line and named on stderr after quitting.
    fn save(&mut self, always: bool) {
        let (Some(dir), Some(session)) = (&self.setup.data, &mut self.session) else {
            return;
        };
        let now = Some(session.saved());
        if now == self.saved && !always {
            return;
        }
        let mut slot = self.slot.lock().unwrap_or_else(PoisonError::into_inner);
        let won = session.game.is_won();
        let result = if won {
            store::remove_game(dir)
        } else {
            store::save_game(dir, &session.game.to_json((self.clock)()))
        };
        *slot = (!won).then(|| session.game.clone());
        match result {
            Ok(()) => self.saved = now,
            Err(err) => {
                session.view.notice = Some("game not saved");
                let line = format!("couldn't save the game: {err}");
                if !self.warnings.contains(&line) {
                    self.warnings.push(line);
                }
            }
        }
    }

    /// Whether the screen showing can be drawn in a window of this size.
    fn fits(&self, area: Rect) -> bool {
        match &self.screen {
            Screen::Game => ui::fits(area),
            Screen::Menu(menu) => ui::menu_fits(area, menu, &menu.items(self.progress())),
            Screen::Settings(_) => ui::settings_fit(area),
        }
    }

    /// The name of the theme the config asks for, which the terminal may
    /// override.
    fn theme_name(&self) -> String {
        Theme::choose(
            &self.setup.themes,
            self.config.theme.as_deref(),
            false,
            true,
        )
        .name
    }

    /// Why the theme chosen isn't the one drawn, when the terminal decides.
    fn theme_note(&self) -> Option<&'static str> {
        if self.setup.no_color {
            Some("NO_COLOR is set, so no theme's colours show here.")
        } else if !self.setup.truecolor {
            Some("This terminal shows 16 colours, not the theme's.")
        } else {
            None
        }
    }

    /// Changes `setting`, saves it, and puts it into effect at once.
    fn change(&mut self, setting: Setting, forward: bool) {
        let ids: Vec<String> = self.setup.themes.iter().map(|(id, _)| id.clone()).collect();
        setting.change(&mut self.config, &ids, forward);
        self.save_error = None;
        if let Some(dir) = &self.setup.dir
            && let Err(err) = Config::save(dir, setting.key(), setting.value(&self.config))
        {
            let line = format!("couldn't save a setting: {err}");
            if !self.warnings.contains(&line) {
                self.warnings.push(line);
            }
            self.save_error = Some(err);
        }
        self.theme = Theme::choose(
            &self.setup.themes,
            self.config.theme.as_deref(),
            self.setup.no_color,
            self.setup.truecolor,
        );
        if let Some(session) = &mut self.session {
            session.configure(&self.config);
        }
    }

    /// Starts a game of `level`. Hidden candidates carry over from the last
    /// game; the rest of its view doesn't.
    fn start(&mut self, level: Level) {
        self.level = level;
        let notes_hidden = self.session.as_ref().is_some_and(|s| s.view.notes_hidden);
        let game = new_game(&mut self.rng, level);
        let mut session = Session::new(game, &self.config, self.clock);
        session.view.notes_hidden = notes_hidden;
        self.session = Some(session);
        self.screen = Screen::Game;
    }

    fn resume(&mut self) {
        if let Some(session) = &mut self.session
            && session.clock_running()
        {
            session.game.start_clock((self.clock)());
        }
        self.screen = Screen::Game;
    }

    /// `shown` is false while the window is too small for the screen;
    /// then only quitting acts. Whatever the event changed in the game is
    /// saved.
    fn handle(&mut self, event: Event, shown: bool) -> Flow {
        let flow = self.react(event, shown);
        self.save(false);
        flow
    }

    fn react(&mut self, event: Event, shown: bool) -> Flow {
        if let Event::Mouse(mouse) = event {
            return if shown {
                self.click(mouse)
            } else {
                Flow::Continue
            };
        }
        if self.screen == Screen::Game {
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
        }
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
        self.menu_action(action)
    }

    /// A key on a menu or the settings page.
    fn menu_action(&mut self, action: Action) -> Flow {
        let progress = self.progress();
        match &mut self.screen {
            Screen::Game => {}
            Screen::Settings(page) => {
                let from_pause = page.from_pause;
                match page.handle(action) {
                    None => {}
                    Some(SettingsChoice::Change(setting, forward)) => self.change(setting, forward),
                    Some(SettingsChoice::Back) => {
                        let kind = if from_pause { Kind::Pause } else { Kind::Main };
                        self.screen = Screen::Menu(Menu::new(kind));
                    }
                }
            }
            Screen::Menu(menu) => {
                let kind = menu.kind;
                match menu.handle(action, progress) {
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
                        Item::Continue(_) | Item::Resume => self.resume(),
                        Item::NewGame => {
                            let from_pause = kind == Kind::Pause;
                            self.screen = Screen::Menu(Menu::new(Kind::Levels { from_pause }));
                        }
                        Item::Level(level) => self.start(level),
                        Item::MainMenu => self.screen = Screen::Menu(Menu::new(Kind::Main)),
                        Item::Settings => {
                            self.screen = Screen::Settings(Page::new(kind == Kind::Pause));
                        }
                        Item::Quit => return Flow::Quit,
                    },
                }
            }
        }
        Flow::Continue
    }

    /// A left click or tap acts on what is drawn under it: on a menu or the
    /// settings page it chooses, as Enter does. Scrolling and other buttons
    /// do nothing, so a swipe on a phone never moves the cursor.
    fn click(&mut self, mouse: MouseEvent) -> Flow {
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return Flow::Continue;
        }
        let (x, y) = (mouse.column, mouse.row);
        let progress = self.progress();
        match &mut self.screen {
            Screen::Game => {
                let session = self.session.as_mut().expect("the game screen shows a game");
                session.click(ui::hit(self.area, &session.view, x, y));
                Flow::Continue
            }
            Screen::Menu(menu) => {
                let items = menu.items(progress);
                match ui::menu_hit(self.area, menu, &items, x, y) {
                    Some(i) => {
                        menu.selected = i;
                        self.menu_action(Action::Confirm)
                    }
                    None => Flow::Continue,
                }
            }
            Screen::Settings(page) => match ui::settings_hit(self.area, x, y) {
                Some(i) => {
                    page.selected = i;
                    self.menu_action(Action::Confirm)
                }
                None => Flow::Continue,
            },
        }
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
    animations: bool,
    /// The houses complete after the last key or click, so the next can
    /// tell which it completed.
    complete: [bool; 27],
    /// When the animations were last moved on.
    last_tick: Instant,
    /// Where the time comes from; tests set their own.
    clock: fn() -> Instant,
}

impl Session {
    /// Starts `game`, with its clock running.
    fn new(mut game: Game, config: &Config, clock: fn() -> Instant) -> Session {
        game.start_clock(clock());
        let complete = complete_houses(game.board());
        let mut session = Session {
            game,
            view: View::default(),
            timer: config.timer,
            animations: config.animations,
            complete,
            last_tick: clock(),
            clock,
        };
        session.configure(config);
        session
    }

    /// Puts the settings in `config` into effect.
    fn configure(&mut self, config: &Config) {
        self.timer = config.timer;
        self.animations = config.animations;
        if !config.animations {
            self.view.animations.clear();
        }
        self.view.highlights = config.highlights;
        self.view.key_bar_hidden = !config.key_bar;
        self.game.set_hint_start(config.hint_start);
        self.game
            .set_candidates_from_marks(match config.marks_to_candidates {
                MarksToCandidates::Fill => CandidatesFromMarks::Fill,
                MarksToCandidates::Merge => CandidatesFromMarks::Merge,
            });
    }

    fn start_animation(&mut self, kind: AnimationKind) {
        if self.view.animations.is_empty() {
            self.last_tick = (self.clock)();
        }
        self.view.animations.push(Animation::new(kind));
    }

    /// Moves the animations on to `now`, dropping those that are over.
    fn tick(&mut self, now: Instant) {
        let step = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        for animation in &mut self.view.animations {
            animation.elapsed += step;
        }
        self.view.animations.retain(|a| !a.done());
    }

    /// Puts the cursor on the last move and shades the cells it changed,
    /// until the next key or click.
    fn show_last_move(&mut self) {
        if let Some((cursor, cells)) = self.game.last_move() {
            self.game.move_to(cursor);
            self.view.last_move = cells;
        }
    }

    fn saved(&self) -> Saved {
        Saved {
            revision: self.game.revision(),
            cursor: self.game.cursor(),
            mode: self.game.mode(),
            hints_used: self.game.hints_used(),
            clock_running: self.game.clock_is_running(),
        }
    }

    /// The time to show, if the timer is on.
    fn time(&self) -> Option<Duration> {
        self.timer.then(|| self.game.elapsed((self.clock)()))
    }

    /// Whether the time played should be going up: not paused, and not won.
    fn clock_running(&self) -> bool {
        !self.view.paused && !self.game.is_won()
    }

    /// Stops the clock while the window is too small to show the board,
    /// and starts it again once the board shows, unless paused or won.
    fn set_shown(&mut self, shown: bool) {
        let now = (self.clock)();
        if !shown {
            self.game.stop_clock(now);
        } else if self.clock_running() {
            self.game.start_clock(now);
        }
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
        self.settle();
        flow
    }

    /// A click on `target`, or on nothing. It acts as the keys would: a
    /// cell moves the cursor there, a note's spot toggles that note in the
    /// note modes, and a pad digit is pressed, or picks the filter after
    /// `f`. Any click, even on nothing, ends the prompt after `f` and clears
    /// the notice, as any key does.
    fn click(&mut self, target: Option<Target>) {
        self.view.notice = None;
        self.view.last_move.clear();
        let filtering = std::mem::take(&mut self.view.awaiting_filter);
        let Some(target) = target else {
            return;
        };
        match target {
            Target::CloseHint => self.view.hint = None,
            _ if self.game.is_won() => {}
            Target::Pad(digit) if filtering => self.view.filter = Some(Filter::Digit(digit)),
            Target::Pad(digit) => self.game.press(digit),
            Target::Cell(cell) => self.game.move_to(cell),
            Target::Note(cell, digit) => {
                self.game.move_to(cell);
                if self.game.mode() != Mode::Digit {
                    self.game.press(digit);
                }
            }
        }
        self.settle();
    }

    /// What follows any key or click: a house it completed pulses, or the
    /// board waves at a win, and the clock stops; a hint closes when the
    /// game has changed, and check marks go with their digits.
    fn settle(&mut self) {
        // A win's wave stands in for the pulses of the houses it completed.
        let complete = complete_houses(self.game.board());
        if self.animations {
            if self.game.is_won() && !self.complete.iter().all(|&c| c) {
                self.view.animations.clear();
                self.start_animation(AnimationKind::Win);
            } else {
                for (i, house) in House::ALL.into_iter().enumerate() {
                    if complete[i] && !self.complete[i] {
                        self.start_animation(AnimationKind::House(house));
                    }
                }
            }
        }
        self.complete = complete;
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
    }

    fn dispatch(&mut self, key: KeyEvent) -> Flow {
        self.view.notice = None;
        self.view.last_move.clear();
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
            Action::Close if game.is_won() => return Flow::Menu,
            // Once solved, the board and the view stay as they are: Enter
            // (a new game), Esc (the menu), i and quitting are the keys that
            // act.
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

/// A game of `level`, with its rating. Easy and Medium puzzles are
/// generated on the spot; Hard and Expert ones come from the bank, which
/// says why.
fn new_game(rng: &mut Xoshiro256PlusPlus, level: Level) -> Game {
    let puzzle = match level {
        Level::Hard | Level::Expert => {
            bank::pick(rng, level).expect("the bank holds Hard and Expert puzzles")
        }
        _ => LevelGenerator::new(level).next(rng).0,
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

    /// Fills the empty cells of `house` with their answers, the last one
    /// last.
    fn complete(s: &mut Session, house: House) {
        let solution = fixtures::solution();
        for cell in house.cells() {
            if s.game.board().get(cell).is_none() {
                s.game.move_to(cell);
                type_keys(s, &solution.get(cell).unwrap().to_string());
            }
        }
    }

    fn animating(s: &Session) -> Vec<AnimationKind> {
        s.view.animations.iter().map(|a| a.kind).collect()
    }

    /// Completing a row starts its pulse, and the pulse is gone once its
    /// time is up.
    #[test]
    fn completing_a_house_pulses_it_once() {
        let mut s = session();
        complete(&mut s, House::Row(0));
        assert!(animating(&s).contains(&AnimationKind::House(House::Row(0))));
        let started = animating(&s).len();
        type_keys(&mut s, "j");
        assert_eq!(animating(&s).len(), started, "moving starts nothing");
        advance(1);
        s.tick(now());
        assert_eq!(animating(&s), []);
    }

    /// Switching animations off stops the ones running.
    #[test]
    fn switching_animations_off_stops_them() {
        let mut s = session();
        complete(&mut s, House::Row(0));
        assert!(!animating(&s).is_empty());
        s.configure(&Config {
            animations: false,
            ..Config::default()
        });
        assert_eq!(animating(&s), []);
    }

    #[test]
    fn a_win_starts_the_wave_and_no_pulses() {
        let mut s = session();
        win(&mut s);
        assert_eq!(animating(&s), [AnimationKind::Win]);
    }

    #[test]
    fn nothing_moves_with_animations_off() {
        let config = Config {
            animations: false,
            ..Config::default()
        };
        let mut s = Session::new(fixtures::game(), &config, now);
        complete(&mut s, House::Row(0));
        win(&mut s);
        assert_eq!(animating(&s), []);
    }

    /// The clock stops while the window is too small to show the board, and
    /// starts again when it shows, unless the game is paused.
    #[test]
    fn the_clock_stops_while_the_board_cant_be_seen() {
        let mut s = session();
        advance(2);
        s.set_shown(false);
        advance(30);
        s.set_shown(true);
        advance(1);
        assert_eq!(s.game.elapsed(now()), secs(3));
        type_keys(&mut s, "p");
        s.set_shown(false);
        s.set_shown(true);
        advance(10);
        assert_eq!(s.game.elapsed(now()), secs(3), "paused stays paused");
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
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(1),
            Start::Menu,
            None,
            config,
            setup(),
            now,
        );
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
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sumi-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// A move is saved as it is made, a key that changes nothing kept
    /// writes nothing, and a win forgets the game.
    #[test]
    fn every_move_is_saved_and_a_win_forgets_the_game() {
        let dir = scratch("autosave");
        let path = dir.join("game.json");
        let mut app = app_with(fixtures::game());
        app.setup.data = Some(dir.clone());
        press_keys(&mut app, &[KeyCode::Char('l'), KeyCode::Char('l')]);
        press_keys(&mut app, &[KeyCode::Char('4')]);
        let saved = Game::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!((saved.board().get(2), saved.cursor()), (Some(4), 2));
        std::fs::remove_file(&path).unwrap();
        press_keys(&mut app, &[KeyCode::Char('Z')]);
        assert!(!path.exists(), "nothing kept changed");
        press_keys(&mut app, &[KeyCode::Char('n')]);
        assert!(path.exists(), "the mode is kept");
        press_keys(&mut app, &[KeyCode::Char('n'), KeyCode::Char('n')]);
        let shared = app.slot.lock().unwrap().as_ref().map(Game::board).copied();
        assert_eq!(
            shared.and_then(|b| b.get(2)),
            Some(4),
            "the signal thread's copy"
        );
        win(game_of(&mut app));
        press_keys(&mut app, &[KeyCode::Char('Z')]);
        assert!(!path.exists());
        assert!(
            app.slot.lock().unwrap().is_none(),
            "nothing to save after a win"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A save that can't be written says so on the status line, and once
    /// on stderr however many keys fail.
    #[test]
    fn a_failed_save_says_so() {
        let dir = scratch("unsaved");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("file"), "").unwrap();
        let mut app = app_with(fixtures::game());
        app.setup.data = Some(dir.join("file").join("sumi"));
        press_keys(&mut app, &[KeyCode::Char('l'), KeyCode::Char('l')]);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(game_of(&mut app).view.notice, Some("game not saved"));
        assert_eq!(app.warnings.len(), 1, "{:?}", app.warnings);
    }

    /// A saved game waits under the main menu, its clock stopped, until
    /// Continue; sumi continue opens it straight away. A won game is
    /// nothing to continue.
    #[test]
    fn a_saved_game_is_continued_from_the_main_menu_or_at_once() {
        let open = |start| {
            let mut game = fixtures::game();
            game.move_to(2);
            game.press(1);
            game.move_to(80);
            App::new(
                Xoshiro256PlusPlus::seed_from_u64(1),
                start,
                Some(game),
                Config::default(),
                setup(),
                now,
            )
        };
        let mut app = open(Start::Menu);
        assert_eq!(menu_kind(&app), Some(Kind::Main));
        assert!(
            app.slot.lock().unwrap().is_some(),
            "a signal before any key saves it"
        );
        // The last move, 1 in r1c3, is shown until the first key.
        assert_eq!(game_of(&mut app).view.last_move, [2]);
        assert_eq!(game_of(&mut app).game.cursor(), 2);
        assert!(!game_of(&mut app).game.clock_is_running());
        press_keys(&mut app, &[KeyCode::Enter]);
        assert_eq!(app.screen, Screen::Game);
        assert!(game_of(&mut app).game.clock_is_running());
        assert_eq!(game_of(&mut app).game.board().get(0), Some(5));
        assert_eq!(game_of(&mut app).view.last_move, [2]);
        press_keys(&mut app, &[KeyCode::Char('Z')]);
        assert!(game_of(&mut app).view.last_move.is_empty());

        let mut app = open(Start::Continue);
        assert_eq!(app.screen, Screen::Game);
        game_of(&mut app).click(None);
        assert!(
            game_of(&mut app).view.last_move.is_empty(),
            "a click is a key"
        );
        assert!(game_of(&mut app).game.clock_is_running());

        win(game_of(&mut app));
        press_keys(&mut app, &[KeyCode::Esc, KeyCode::Down, KeyCode::Down]);
        press_keys(&mut app, &[KeyCode::Down, KeyCode::Enter]);
        let Screen::Menu(menu) = &app.screen else {
            panic!("the main menu")
        };
        assert_eq!(menu.items(app.progress()).first(), Some(&Item::NewGame));
    }

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
    fn after_a_win_keys_meant_for_the_board_do_nothing() {
        let mut s = session();
        win(&mut s);
        let before = (s.game.revision(), s.game.cursor(), s.view.clone());
        type_keys(&mut s, "uhf3vn?");
        assert!(s.game.is_won());
        assert_eq!((s.game.revision(), s.game.cursor(), s.view.clone()), before);
        assert_eq!(s.handle(key(KeyCode::Enter), true), Flow::NewGame);
    }

    #[test]
    fn every_game_is_of_the_level_asked_for() {
        for level in [Level::Medium, Level::Hard] {
            let mut app = App::new(
                Xoshiro256PlusPlus::seed_from_u64(2),
                Start::Level(level),
                None,
                Config::default(),
                setup(),
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

    /// The built-in themes, no config directory to save to, and a terminal
    /// with 24-bit colour.
    fn setup() -> Setup {
        Setup {
            dir: None,
            data: None,
            themes: Theme::load_all(None).0,
            no_color: false,
            truecolor: true,
        }
    }

    /// An app playing `game`.
    fn app_with(game: Game) -> App {
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(1),
            Start::Menu,
            None,
            Config::default(),
            setup(),
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
            Screen::Game | Screen::Settings(_) => None,
        }
    }

    /// Bare sumi opens the main menu with no game; New game and a level
    /// start one, Medium unless another is picked.
    #[test]
    fn the_main_menu_starts_a_game_of_the_level_picked() {
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(3),
            Start::Menu,
            None,
            Config::default(),
            setup(),
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
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(3),
            Start::Menu,
            None,
            Config::default(),
            setup(),
            now,
        );
        press_keys(&mut app, &[KeyCode::Enter, KeyCode::Enter]);
        assert_eq!(game_of(&mut app).game.level(), Some(Level::Medium));
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
    /// After a win Esc opens the pause menu, so another level or the
    /// settings are in reach; Resume goes back to the won board with the
    /// clock still stopped.
    #[test]
    fn esc_after_a_win_opens_the_menu_and_resume_keeps_the_clock_stopped() {
        let mut app = app_with(fixtures::game());
        advance(4);
        win(game_of(&mut app));
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Pause));
        press_keys(&mut app, &[KeyCode::Enter]);
        assert_eq!(app.screen, Screen::Game);
        advance(50);
        assert_eq!(game_of(&mut app).game.elapsed(now()), secs(4));
    }

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

    /// Opens the settings from the pause menu: Esc, down twice, Enter.
    fn open_settings(app: &mut App) {
        press_keys(
            app,
            &[
                KeyCode::Esc,
                KeyCode::Char('j'),
                KeyCode::Char('j'),
                KeyCode::Enter,
            ],
        );
        assert_eq!(app.screen, Screen::Settings(Page::new(true)));
    }

    /// A change on the settings page takes effect in the game at once, and
    /// Esc goes back to the menu the page was opened from.
    #[test]
    fn settings_take_effect_at_once() {
        let mut app = app_with(fixtures::game());
        open_settings(&mut app);
        let change = |app: &mut App, setting: Setting| {
            while !matches!(&app.screen, Screen::Settings(page) if page.setting() == setting) {
                press_keys(app, &[KeyCode::Char('j')]);
            }
            press_keys(app, &[KeyCode::Char('l')]);
        };
        change(&mut app, Setting::Timer);
        assert!(app.config.timer);
        assert!(game_of(&mut app).timer);
        change(&mut app, Setting::Mouse);
        assert!(!app.config.mouse);
        change(&mut app, Setting::HintStart);
        assert_eq!(game_of(&mut app).game.hint_start(), 2);
        change(&mut app, Setting::MarksToCandidates);
        assert_eq!(app.config.marks_to_candidates, MarksToCandidates::Merge);
        assert_eq!(
            game_of(&mut app).game.candidates_from_marks(),
            CandidatesFromMarks::Merge
        );
        change(&mut app, Setting::SameDigit);
        assert!(!game_of(&mut app).view.highlights.same_digit);
        change(&mut app, Setting::Theme);
        assert_eq!(app.theme.name, "Tokyo Night");
        change(&mut app, Setting::KeyBar);
        assert!(game_of(&mut app).view.key_bar_hidden);
        change(&mut app, Setting::Animations);
        assert!(!game_of(&mut app).animations);
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Pause));
    }

    /// Opened from the main menu, the settings go back to the main menu.
    #[test]
    fn settings_from_the_main_menu_go_back_there() {
        let mut app = App::new(
            Xoshiro256PlusPlus::seed_from_u64(1),
            Start::Menu,
            None,
            Config::default(),
            setup(),
            now,
        );
        press_keys(&mut app, &[KeyCode::Char('j'), KeyCode::Enter]);
        assert_eq!(app.screen, Screen::Settings(Page::new(false)));
        press_keys(&mut app, &[KeyCode::Esc]);
        assert_eq!(menu_kind(&app), Some(Kind::Main));
    }

    /// Each change is written to config.toml as it is made.
    #[test]
    fn a_change_is_saved_to_the_config_file() {
        let dir = std::env::temp_dir().join(format!("sumi-app-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut app = app_with(fixtures::game());
        app.setup.dir = Some(dir.clone());
        open_settings(&mut app);
        press_keys(&mut app, &[KeyCode::Enter]);
        let saved = std::fs::read_to_string(dir.join("config.toml")).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(Config::parse(&saved).unwrap().0.timer);
        assert_eq!(app.save_error, None);
    }

    fn click(column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    /// An app playing the fixture in a window with room for the full
    /// layout, and that layout's areas.
    fn clickable() -> (App, ui::Areas) {
        let mut app = app_with(fixtures::game());
        app.area = Rect::new(0, 0, 80, 50);
        let areas = ui::Areas::of(app.area).unwrap();
        (app, areas)
    }

    fn middle_of(rect: Rect) -> (u16, u16) {
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn a_click_on_a_cell_moves_the_cursor_there() {
        let (mut app, areas) = clickable();
        let (x, y) = middle_of(areas.cell(40));
        app.handle(click(x, y), true);
        assert_eq!(game_of(&mut app).game.cursor(), 40);
    }

    /// A pad digit is pressed at the cursor: placed in digit mode, toggled
    /// as a note in the others.
    #[test]
    fn a_click_on_the_pad_presses_its_digit() {
        let (mut app, areas) = clickable();
        type_keys(game_of(&mut app), "ll"); // r1c3, empty
        let (x, y) = middle_of(areas.pad_button(4));
        app.handle(click(x, y), true);
        assert_eq!(game_of(&mut app).game.board().get(2), Some(4));
        type_keys(game_of(&mut app), "0nl"); // r1c4, candidate mode
        app.handle(click(x, y), true);
        assert!(game_of(&mut app).game.candidates(3).contains(4));
    }

    /// After f, a pad digit picks the filter; a click anywhere else cancels
    /// the prompt as another key would.
    #[test]
    fn after_f_a_pad_digit_picks_the_filter() {
        let (mut app, areas) = clickable();
        type_keys(game_of(&mut app), "f");
        let (x, y) = middle_of(areas.pad_button(6));
        app.handle(click(x, y), true);
        let s = game_of(&mut app);
        assert_eq!(s.view.filter, Some(Filter::Digit(6)));
        assert!(!s.view.awaiting_filter);
        assert_eq!(s.game.board().get(0), Some(5), "nothing placed");
        type_keys(s, "f");
        let (x, y) = middle_of(areas.cell(40));
        app.handle(click(x, y), true);
        assert!(!game_of(&mut app).view.awaiting_filter);
        assert_eq!(game_of(&mut app).view.filter, Some(Filter::Digit(6)));
    }

    /// In a note mode a click on a note's spot toggles that note; in digit
    /// mode it only moves the cursor.
    #[test]
    fn a_click_on_a_notes_spot_toggles_it_in_a_note_mode() {
        let (mut app, areas) = clickable();
        let cell = areas.cell(2);
        // The 7's spot: bottom row, left column of the 3×3 grid.
        let (x, y) = (cell.x + 1, cell.y + 2);
        app.handle(click(x, y), true);
        assert_eq!(game_of(&mut app).game.cursor(), 2);
        assert!(game_of(&mut app).game.candidates(2).is_empty());
        type_keys(game_of(&mut app), "n");
        app.handle(click(x, y), true);
        assert!(game_of(&mut app).game.candidates(2).contains(7));
    }

    /// A click that changes the board closes a hint worked out before it,
    /// as a key would.
    #[test]
    fn a_click_that_changes_the_board_closes_the_hint() {
        let (mut app, areas) = clickable();
        type_keys(game_of(&mut app), "ll??");
        let (x, y) = middle_of(areas.pad_button(4));
        app.handle(click(x, y), true);
        assert_eq!(rung(game_of(&mut app)), None);
    }

    /// A click on nothing, a line between cells here, ends the prompt after
    /// `f` as any key would.
    #[test]
    fn a_click_on_nothing_ends_the_filter_prompt() {
        let (mut app, areas) = clickable();
        type_keys(game_of(&mut app), "f");
        let cell = areas.cell(40);
        app.handle(click(cell.x - 1, cell.y + 1), true);
        assert!(!game_of(&mut app).view.awaiting_filter);
        type_keys(game_of(&mut app), "3");
        assert_eq!(game_of(&mut app).view.filter, None);
    }

    #[test]
    fn a_click_on_the_close_button_closes_the_hint() {
        let (mut app, areas) = clickable();
        type_keys(game_of(&mut app), "??");
        let (x, y) = middle_of(areas.close_hint());
        app.handle(click(x, y), true);
        assert_eq!(rung(game_of(&mut app)), None);
    }

    /// Scrolling and other buttons do nothing, so a swipe never moves the
    /// cursor; neither does a click while the window is too small.
    #[test]
    fn only_a_left_click_on_a_drawn_screen_acts() {
        let (mut app, areas) = clickable();
        let (x, y) = middle_of(areas.cell(40));
        for kind in [
            MouseEventKind::ScrollDown,
            MouseEventKind::ScrollUp,
            MouseEventKind::Down(MouseButton::Right),
            MouseEventKind::Up(MouseButton::Left),
            MouseEventKind::Drag(MouseButton::Left),
        ] {
            let event = Event::Mouse(MouseEvent {
                kind,
                column: x,
                row: y,
                modifiers: KeyModifiers::NONE,
            });
            app.handle(event, true);
        }
        app.handle(click(x, y), false);
        assert_eq!(game_of(&mut app).game.cursor(), 0);
    }

    /// The first spot in the window a click chooses the menu item at `index`.
    fn spot_of_item(app: &App, index: usize) -> (u16, u16) {
        let Screen::Menu(menu) = &app.screen else {
            panic!("no menu")
        };
        let items = menu.items(app.progress());
        (0..app.area.height)
            .flat_map(|y| (0..app.area.width).map(move |x| (x, y)))
            .find(|&(x, y)| ui::menu_hit(app.area, menu, &items, x, y) == Some(index))
            .unwrap()
    }

    #[test]
    fn a_click_on_a_menu_item_chooses_it() {
        let (mut app, _) = clickable();
        press_keys(&mut app, &[KeyCode::Esc]);
        let (x, y) = spot_of_item(&app, 2); // Settings
        app.handle(click(x, y), true);
        assert_eq!(app.screen, Screen::Settings(Page::new(true)));
        let (x, y) = (0..app.area.height)
            .flat_map(|y| (0..app.area.width).map(move |x| (x, y)))
            .find(|&(x, y)| ui::settings_hit(app.area, x, y) == Some(0))
            .unwrap();
        app.handle(click(x, y), true);
        assert!(app.config.timer, "a click on a setting changes it");
    }

    /// A change that can't be saved says why on the page, and the same
    /// failure is named once on stderr, however often it happens.
    #[test]
    fn a_failed_save_is_shown_and_named_once() {
        let file = std::env::temp_dir().join(format!("sumi-not-a-dir-{}", std::process::id()));
        std::fs::write(&file, "").unwrap();
        let mut app = app_with(fixtures::game());
        app.setup.dir = Some(file.clone());
        open_settings(&mut app);
        press_keys(&mut app, &[KeyCode::Enter, KeyCode::Enter, KeyCode::Enter]);
        std::fs::remove_file(&file).unwrap();
        assert!(app.save_error.is_some());
        assert_eq!(app.warnings.len(), 1, "{:?}", app.warnings);
    }

    #[test]
    fn q_quits_from_any_menu_and_keys_do_nothing_in_a_window_too_small() {
        // Each list of keys reaches a screen: the pause menu, the level
        // list, the settings, the main menu.
        let screens: [&[KeyCode]; 4] = [
            &[KeyCode::Esc],
            &[KeyCode::Esc, KeyCode::Char('j'), KeyCode::Enter],
            &[
                KeyCode::Esc,
                KeyCode::Char('k'),
                KeyCode::Char('k'),
                KeyCode::Enter,
            ],
            &[KeyCode::Esc, KeyCode::Char('k'), KeyCode::Enter],
        ];
        for keys in screens {
            let mut app = app_with(fixtures::game());
            press_keys(&mut app, keys);
            assert_ne!(app.screen, Screen::Game);
            let screen = app.screen.clone();
            assert_eq!(app.handle(key(KeyCode::Enter), false), Flow::Continue);
            assert_eq!(app.screen, screen, "nothing acts in a window too small");
            assert_eq!(press_keys(&mut app, &[KeyCode::Char('q')]), Flow::Quit);
        }
    }
}
