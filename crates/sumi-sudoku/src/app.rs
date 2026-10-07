//! The interactive game: one loop that draws, waits for a key and acts on it.

use std::io;

use rand::rngs::Xoshiro256PlusPlus;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use sumi_core::generate::generate;

use crate::game::Game;
use crate::input::{Action, FilterKey, action, filter_key};
use crate::theme::Theme;
use crate::ui::{self, Filter, View};

pub fn run() -> io::Result<()> {
    ratatui::run(play)
}

fn play(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let theme = Theme::tokyo_night_amoled();
    let mut rng: Xoshiro256PlusPlus = rand::make_rng();
    let mut game = Game::new(generate(&mut rng));
    let mut view = View::default();

    loop {
        terminal.draw(|frame| ui::draw(frame, &game, &view, &theme))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if view.awaiting_filter {
            view.awaiting_filter = false;
            match filter_key(key) {
                Some(FilterKey::Digit(d)) => view.filter = Some(Filter::Digit(d)),
                Some(FilterKey::TwoCandidates) => view.filter = Some(Filter::TwoCandidates),
                Some(FilterKey::Off) => view.filter = None,
                None => {}
            }
            continue;
        }

        let Some(action) = action(key) else {
            continue;
        };
        match action {
            Action::Quit => return Ok(()),
            Action::Confirm if game.is_won() => {
                game = Game::new(generate(&mut rng));
                view = View::default();
            }
            // A solved board stays as it is until the next game.
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
            Action::ToggleNotes => view.notes_hidden = !view.notes_hidden,
            Action::Filter => view.awaiting_filter = true,
            Action::Undo => game.undo(),
            Action::Redo => game.redo(),
            Action::Confirm => {}
        }
    }
}
