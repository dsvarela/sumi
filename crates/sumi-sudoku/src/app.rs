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
use sumi_core::generate::{LevelGenerator, Puzzle, generate};
use sumi_core::technique::Level;

use crate::game::Game;
use crate::input::{Action, FilterKey, action, filter_key};
use crate::theme::Theme;
use crate::ui::{self, Filter, View};

/// Plays puzzles of `level`, or any generated puzzle without one. Fails
/// without changing the terminal when there is none to draw on.
pub fn run(level: Option<Level>) -> io::Result<()> {
    let mut terminal = ratatui::try_init()
        .map_err(|err| io::Error::new(err.kind(), format!("can't open the terminal: {err}")))?;
    // With bracketed paste on, pasted text arrives as one event that the game
    // ignores, instead of as keys that could place digits or quit.
    let result =
        execute!(io::stdout(), EnableBracketedPaste).and_then(|()| play(&mut terminal, level));
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    result
}

fn play(terminal: &mut DefaultTerminal, level: Option<Level>) -> io::Result<()> {
    let theme = Theme::tokyo_night_amoled();
    let mut session = Session::new(rand::make_rng(), level);
    loop {
        let mut board_shown = true;
        terminal.draw(|frame| {
            board_shown = ui::fits(frame.area());
            ui::draw(frame, &session.game, &session.view, &theme);
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
    /// board; keys then do nothing but quit, so nothing changes unseen.
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
        let game = &mut self.game;
        match action {
            Action::Quit => return Flow::Quit,
            _ if !board_shown => {}
            Action::Confirm if game.is_won() => {
                self.game = Game::new(new_puzzle(&mut self.rng, self.level));
                self.view.filter = None;
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
            Action::Confirm => {}
        }
        Flow::Continue
    }
}

/// Easy and Medium puzzles are generated on the spot; Hard and Expert ones
/// are too rare for that and come from the bank.
fn new_puzzle(rng: &mut Xoshiro256PlusPlus, level: Option<Level>) -> Puzzle {
    match level {
        None => generate(rng),
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
            assert_eq!(grade(s.game.board()).level, level);
            s.game = fixtures::game();
            win(&mut s);
            s.handle(key(KeyCode::Enter), true);
            assert_eq!(grade(s.game.board()).level, level);
        }
    }
}
