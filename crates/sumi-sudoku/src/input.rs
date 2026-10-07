//! Keys to actions. Remapping from config.toml arrives with the settings page.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Step(Direction),
    Jump(Direction),
    /// `None` clears the cell.
    Write(Option<u8>),
    Undo,
    Redo,
    Confirm,
    Quit,
}

pub fn action(key: KeyEvent) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let action = match key.code {
        KeyCode::Char('c') if ctrl => Action::Quit,
        KeyCode::Char('r') if ctrl => Action::Redo,
        _ if ctrl => return None,
        KeyCode::Char('h') | KeyCode::Left => Action::Step(Direction::Left),
        KeyCode::Char('j') | KeyCode::Down => Action::Step(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => Action::Step(Direction::Up),
        KeyCode::Char('l') | KeyCode::Right => Action::Step(Direction::Right),
        KeyCode::Char('H') => Action::Jump(Direction::Left),
        KeyCode::Char('J') => Action::Jump(Direction::Down),
        KeyCode::Char('K') => Action::Jump(Direction::Up),
        KeyCode::Char('L') => Action::Jump(Direction::Right),
        KeyCode::Char(c @ '1'..='9') => Action::Write(Some(c as u8 - b'0')),
        KeyCode::Char('0') | KeyCode::Backspace | KeyCode::Delete => Action::Write(None),
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Enter => Action::Confirm,
        KeyCode::Char('q') => Action::Quit,
        _ => return None,
    };
    Some(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn digits_place_and_zero_clears() {
        assert_eq!(
            action(key(KeyCode::Char('7'))),
            Some(Action::Write(Some(7)))
        );
        assert_eq!(action(key(KeyCode::Char('0'))), Some(Action::Write(None)));
        assert_eq!(action(key(KeyCode::Backspace)), Some(Action::Write(None)));
    }

    #[test]
    fn shifted_letters_jump_a_box() {
        // Terminals report Shift+h as 'H', with or without the SHIFT flag.
        let shifted = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT);
        assert_eq!(action(shifted), Some(Action::Jump(Direction::Left)));
        assert_eq!(
            action(key(KeyCode::Char('h'))),
            Some(Action::Step(Direction::Left))
        );
    }

    #[test]
    fn control_keys() {
        let ctrl = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert_eq!(action(ctrl('r')), Some(Action::Redo));
        assert_eq!(action(ctrl('c')), Some(Action::Quit));
        assert_eq!(action(ctrl('u')), None);
    }
}
