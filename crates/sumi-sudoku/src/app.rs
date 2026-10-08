//! The interactive game: one loop that draws, waits for an event and hands it
//! to the session.

use std::io;

use rand::rngs::Xoshiro256PlusPlus;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use ratatui::crossterm::execute;
use sumi_core::bank;
use sumi_core::generate::{LevelGenerator, Puzzle, generate_solvable};
use sumi_core::hint::Hint;
use sumi_core::technique::Level;

use crate::config::Config;
use crate::game::Game;
use crate::input::{Action, FilterKey, action, filter_key};
use crate::theme::Theme;
use crate::ui::{self, Filter, OpenHint, View};

/// Plays puzzles of `level`, or any generated puzzle without one. Fails
/// without changing the terminal when there is none to draw on.
pub fn run(level: Option<Level>) -> io::Result<()> {
    let (theme, warnings) = start_theme();
    let mut terminal = ratatui::try_init()
        .map_err(|err| io::Error::new(err.kind(), format!("can't open the terminal: {err}")))?;
    // With bracketed paste on, pasted text arrives as one event that the game
    // ignores, instead of as keys that could place digits or quit.
    let result = execute!(io::stdout(), EnableBracketedPaste)
        .and_then(|()| play(&mut terminal, level, &theme));
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    // The screen hid them while the game ran.
    for line in warnings {
        eprintln!("sumi: {line}");
    }
    result
}

/// The theme the player asked for, if the environment allows it, and a
/// line for each file that couldn't be read.
fn start_theme() -> (Theme, Vec<String>) {
    let dir = Config::dir();
    let (config, warning) = Config::load(dir.as_ref());
    let (themes, warnings) = Theme::load_all(dir.map(|d| d.join("themes")).as_deref());
    let warnings: Vec<String> = warning.into_iter().chain(warnings).collect();
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let truecolor = std::env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit");
    let theme = Theme::choose(&themes, config.theme.as_deref(), no_color, truecolor);
    (theme, warnings)
}

fn play(terminal: &mut DefaultTerminal, level: Option<Level>, theme: &Theme) -> io::Result<()> {
    let mut session = Session::new(rand::make_rng(), level);
    loop {
        let mut board_shown = true;
        terminal.draw(|frame| {
            board_shown = ui::fits(frame.area());
            ui::draw(frame, &session.game, &session.view, theme);
        })?;
        if session.handle(event::read()?, board_shown) == Flow::Quit {
            return Ok(());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Quit,
}

/// The game being played, how it is shown, and where new puzzles come from.
struct Session {
    game: Game,
    view: View,
    rng: Xoshiro256PlusPlus,
    level: Option<Level>,
}

impl Session {
    fn new(mut rng: Xoshiro256PlusPlus, level: Option<Level>) -> Session {
        Session {
            game: Game::new(new_puzzle(&mut rng, level)),
            view: View::default(),
            rng,
            level,
        }
    }

    /// `board_shown` is false while the window is too small to draw the
    /// board; keys then do nothing but quit, so nothing changes unseen, the
    /// view included.
    fn handle(&mut self, event: Event, board_shown: bool) -> Flow {
        // Only presses act. Terminals that report releases and repeats as
        // separate events would otherwise apply every key twice.
        let Event::Key(key) = event else {
            return Flow::Continue;
        };
        if key.kind != KeyEventKind::Press {
            return Flow::Continue;
        }
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
        // The key list covers the board, so nothing under it acts.
        if self.view.help {
            match action {
                Action::Quit => return Flow::Quit,
                Action::Help | Action::Close => self.view.help = false,
                _ => {}
            }
            return Flow::Continue;
        }
        let game = &mut self.game;
        match action {
            Action::Quit => return Flow::Quit,
            Action::Help => self.view.help = true,
            Action::Confirm if game.is_won() => {
                self.game = Game::new(new_puzzle(&mut self.rng, self.level));
                // Hidden candidates carry over; nothing else of the last
                // game does.
                self.view = View {
                    notes_hidden: self.view.notes_hidden,
                    ..View::default()
                };
            }
            // Once solved, the board and the view stay as they are until the
            // next game: Enter and quitting are the only keys that act.
            _ if game.is_won() => {}
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
            Action::Close => {
                if self.view.hint.take().is_none() {
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

/// Easy and Medium puzzles are generated on the spot; Hard and Expert ones
/// come from the bank, which says why.
fn new_puzzle(rng: &mut Xoshiro256PlusPlus, level: Option<Level>) -> Puzzle {
    match level {
        None => generate_solvable(rng).0,
        Some(level @ (Level::Hard | Level::Expert)) => {
            bank::pick(rng, level).expect("the bank holds Hard and Expert puzzles")
        }
        Some(level) => LevelGenerator::new(level).next(rng).0,
    }
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
        let mut s = Session::new(Xoshiro256PlusPlus::seed_from_u64(1), None);
        s.game = fixtures::game();
        s
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
    fn a_new_game_starts_with_a_clean_view_but_keeps_hidden_candidates() {
        let mut s = session();
        type_keys(&mut s, "v");
        type_keys(&mut s, "ll1c");
        type_keys(&mut s, "f3");
        type_keys(&mut s, "?");
        win(&mut s);
        s.handle(key(KeyCode::Enter), true);
        assert_eq!(
            s.view,
            View {
                notes_hidden: true,
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

        s.handle(key(KeyCode::Enter), true);
        assert!(!s.game.is_won());
        assert!(s.view.notes_hidden);
    }

    #[test]
    fn every_game_is_of_the_level_asked_for() {
        for level in [Level::Medium, Level::Hard] {
            let mut s = Session::new(Xoshiro256PlusPlus::seed_from_u64(2), Some(level));
            assert_eq!(grade(s.game.board()).unwrap().level, level);
            s.game = fixtures::game();
            win(&mut s);
            s.handle(key(KeyCode::Enter), true);
            assert_eq!(grade(s.game.board()).unwrap().level, level);
        }
    }
}
