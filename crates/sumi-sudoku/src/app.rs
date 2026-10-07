//! The interactive game: one loop that draws, waits for a key and acts on it.

use std::io;

use rand::rngs::Xoshiro256PlusPlus;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use sumi_core::generate::generate;

use crate::game::Game;
use crate::input::{Action, action};
use crate::theme::Theme;
use crate::ui;

pub fn run() -> io::Result<()> {
    ratatui::run(play)
}

fn play(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let theme = Theme::tokyo_night_amoled();
    let mut rng: Xoshiro256PlusPlus = rand::make_rng();
    let mut game = Game::new(generate(&mut rng));

    loop {
        terminal.draw(|frame| ui::draw(frame, &game, &theme))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let Some(action) = action(key) else {
            continue;
        };
        match action {
            Action::Quit => return Ok(()),
            Action::Confirm if game.is_won() => game = Game::new(generate(&mut rng)),
            // A solved board stays as it is until the next game.
            _ if game.is_won() => {}
            Action::Step(direction) => game.step(direction),
            Action::Jump(direction) => game.jump(direction),
            Action::Write(digit) => game.write(digit),
            Action::Undo => game.undo(),
            Action::Redo => game.redo(),
            Action::Confirm => {}
        }
    }
}
