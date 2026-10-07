//! Keys to actions. Remapping from config.toml arrives with the settings page.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Step(Direction),
    Jump(Direction),
    /// A digit key: places, or toggles a note, depending on the mode.
    Digit(u8),
    Clear,
    NextMode,
    FillCandidates,
    RefillCandidates,
    MarksToCandidates,
    /// Clears the notes of the current mode's layer from the whole board.
    ClearLayer,
    ClearAllNotes,
    ToggleNotes,
    /// The next key picks what the candidate filter shows.
    Filter,
    Undo,
    Redo,
    Confirm,
    Quit,
}

/// What the key after `f` asks the candidate filter for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKey {
    Digit(u8),
    TwoCandidates,
    Off,
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
        KeyCode::Char(c @ '1'..='9') => Action::Digit(c as u8 - b'0'),
        KeyCode::Char('0') | KeyCode::Backspace | KeyCode::Delete => Action::Clear,
        KeyCode::Char('n') => Action::NextMode,
        KeyCode::Char('a') => Action::FillCandidates,
        KeyCode::Char('A') => Action::RefillCandidates,
        KeyCode::Char('t') => Action::MarksToCandidates,
        KeyCode::Char('x') => Action::ClearLayer,
        KeyCode::Char('X') => Action::ClearAllNotes,
        KeyCode::Char('v') => Action::ToggleNotes,
        KeyCode::Char('f') => Action::Filter,
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Enter => Action::Confirm,
        KeyCode::Char('q') => Action::Quit,
        _ => return None,
    };
    Some(action)
}

/// Reads the key after `f`. Any key without a meaning here cancels.
pub fn filter_key(key: KeyEvent) -> Option<FilterKey> {
    match key.code {
        KeyCode::Char(c @ '1'..='9') => Some(FilterKey::Digit(c as u8 - b'0')),
        KeyCode::Char('b') => Some(FilterKey::TwoCandidates),
        KeyCode::Char('0') => Some(FilterKey::Off),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn digits_and_clearing() {
        assert_eq!(action(key(KeyCode::Char('7'))), Some(Action::Digit(7)));
        assert_eq!(action(key(KeyCode::Char('0'))), Some(Action::Clear));
        assert_eq!(action(key(KeyCode::Backspace)), Some(Action::Clear));
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
        assert_eq!(
            action(key(KeyCode::Char('A'))),
            Some(Action::RefillCandidates)
        );
    }

    #[test]
    fn control_keys() {
        let ctrl = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert_eq!(action(ctrl('r')), Some(Action::Redo));
        assert_eq!(action(ctrl('c')), Some(Action::Quit));
        assert_eq!(action(ctrl('u')), None);
    }

    #[test]
    fn the_key_after_f() {
        assert_eq!(
            filter_key(key(KeyCode::Char('4'))),
            Some(FilterKey::Digit(4))
        );
        assert_eq!(
            filter_key(key(KeyCode::Char('b'))),
            Some(FilterKey::TwoCandidates)
        );
        assert_eq!(filter_key(key(KeyCode::Char('0'))), Some(FilterKey::Off));
        assert_eq!(filter_key(key(KeyCode::Esc)), None);
    }
}
