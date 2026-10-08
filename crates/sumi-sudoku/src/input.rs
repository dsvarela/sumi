//! Keys to actions, and the key text the key bar shows for each action.

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
    /// Opens the hint, or moves it up a rung.
    Hint,
    /// Closes the hint, or the marks a check left.
    Close,
    /// Marks the digits that differ from the solution.
    Check,
    /// Writes the cursor cell's digit from the solution.
    Reveal,
    Confirm,
    /// Opens or closes the list of every key.
    Help,
    /// Stops the clock and hides the board, or carries on.
    Pause,
    Quit,
}

impl Action {
    /// One of each action, in the order the help screen lists them. A test
    /// holds this to `index`, whose match the compiler makes complete.
    pub const ALL: [Action; 22] = [
        Action::Step(Direction::Up),
        Action::Jump(Direction::Up),
        Action::Digit(1),
        Action::Clear,
        Action::NextMode,
        Action::FillCandidates,
        Action::RefillCandidates,
        Action::MarksToCandidates,
        Action::ClearLayer,
        Action::ClearAllNotes,
        Action::ToggleNotes,
        Action::Filter,
        Action::Hint,
        Action::Confirm,
        Action::Check,
        Action::Reveal,
        Action::Close,
        Action::Undo,
        Action::Redo,
        Action::Pause,
        Action::Help,
        Action::Quit,
    ];

    /// The action's place in `ALL`.
    #[cfg(test)]
    fn index(self) -> usize {
        match self {
            Action::Step(_) => 0,
            Action::Jump(_) => 1,
            Action::Digit(_) => 2,
            Action::Clear => 3,
            Action::NextMode => 4,
            Action::FillCandidates => 5,
            Action::RefillCandidates => 6,
            Action::MarksToCandidates => 7,
            Action::ClearLayer => 8,
            Action::ClearAllNotes => 9,
            Action::ToggleNotes => 10,
            Action::Filter => 11,
            Action::Hint => 12,
            Action::Confirm => 13,
            Action::Check => 14,
            Action::Reveal => 15,
            Action::Close => 16,
            Action::Undo => 17,
            Action::Redo => 18,
            Action::Pause => 19,
            Action::Help => 20,
            Action::Quit => 21,
        }
    }
}

/// What the key after `f` asks the candidate filter for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKey {
    Digit(u8),
    TwoCandidates,
    Off,
}

/// Keys held with Alt mean nothing here, so Alt+h doesn't move like h.
pub fn action(key: KeyEvent) -> Option<Action> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    let action = match key.code {
        KeyCode::Char('c') if ctrl => Action::Quit,
        KeyCode::Char('r') if ctrl => Action::Redo,
        // Terminals that send ^H for Backspace arrive as Ctrl+h.
        KeyCode::Char('h') if ctrl => Action::Clear,
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
        KeyCode::Char('?') => Action::Hint,
        KeyCode::Char('c') => Action::Check,
        KeyCode::Char('r') => Action::Reveal,
        KeyCode::Esc => Action::Close,
        KeyCode::Enter => Action::Confirm,
        KeyCode::Char('i') => Action::Help,
        KeyCode::Char('p') | KeyCode::Char(' ') => Action::Pause,
        KeyCode::Char('q') => Action::Quit,
        _ => return None,
    };
    Some(action)
}

/// Reads the key after `f`. Any key without a meaning here cancels.
pub fn filter_key(key: KeyEvent) -> Option<FilterKey> {
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return None;
    }
    match key.code {
        KeyCode::Char(c @ '1'..='9') => Some(FilterKey::Digit(c as u8 - b'0')),
        KeyCode::Char('b') => Some(FilterKey::TwoCandidates),
        KeyCode::Char('0') => Some(FilterKey::Off),
        _ => None,
    }
}

/// The key text the key bar shows for `action`. The tests check every label
/// against [`action`], so the two cannot drift apart.
pub fn key_label(action: Action) -> &'static str {
    match action {
        Action::Step(_) => "hjkl",
        Action::Jump(_) => "HJKL",
        Action::Digit(_) => "1-9",
        Action::Clear => "0",
        Action::NextMode => "n",
        Action::FillCandidates => "a",
        Action::RefillCandidates => "A",
        Action::MarksToCandidates => "t",
        Action::ClearLayer => "x",
        Action::ClearAllNotes => "X",
        Action::ToggleNotes => "v",
        Action::Filter => "f",
        Action::Undo => "u",
        Action::Redo => "^r",
        Action::Hint => "?",
        Action::Close => "Esc",
        Action::Check => "c",
        Action::Reveal => "r",
        Action::Confirm => "Enter",
        Action::Help => "i",
        Action::Pause => "p",
        Action::Quit => "q",
    }
}

/// The key text for what the key after `f` asks for. The tests check each
/// label against [`filter_key`].
pub fn filter_label(key: FilterKey) -> &'static str {
    match key {
        FilterKey::Digit(_) => "1-9",
        FilterKey::TwoCandidates => "b",
        FilterKey::Off => "0",
    }
}

/// The keys that move up and down a list, for a menu's key bar.
pub fn up_down_label() -> &'static str {
    "jk"
}

/// The keys that change a setting back and forward.
pub fn left_right_label() -> &'static str {
    "hl"
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

    /// The key events a label stands for: "^r" is Ctrl+r, "1-9" each digit,
    /// and any other label each of its characters.
    fn keys_of(label: &str) -> Vec<KeyEvent> {
        match label {
            "Enter" => vec![key(KeyCode::Enter)],
            "Esc" => vec![key(KeyCode::Esc)],
            "1-9" => ('1'..='9').map(|c| key(KeyCode::Char(c))).collect(),
            l if l.starts_with('^') => l[1..]
                .chars()
                .map(|c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
                .collect(),
            l => l.chars().map(|c| key(KeyCode::Char(c))).collect(),
        }
    }

    /// `ALL` has every action once, in `index` order. A new action fails to
    /// compile until `index` has it, and then fails here until `ALL` does.
    #[test]
    fn all_lists_every_action_once() {
        for (i, action) in Action::ALL.iter().enumerate() {
            assert_eq!(action.index(), i, "{action:?}");
        }
        let last = Action::ALL.iter().map(|a| a.index()).max().unwrap();
        assert_eq!(Action::ALL.len(), last + 1);
    }

    #[test]
    fn every_key_label_names_keys_that_do_that_action() {
        for wanted in Action::ALL {
            for k in keys_of(key_label(wanted)) {
                let got = action(k).unwrap_or_else(|| panic!("{k:?} does nothing"));
                assert_eq!(
                    std::mem::discriminant(&got),
                    std::mem::discriminant(&wanted),
                    "{k:?}"
                );
            }
        }
    }

    /// Ctrl+c quits and Ctrl+r redoes, so c and r need no modifier.
    #[test]
    fn check_and_reveal_are_plain_keys() {
        assert_eq!(action(key(KeyCode::Char('c'))), Some(Action::Check));
        assert_eq!(action(key(KeyCode::Char('r'))), Some(Action::Reveal));
        let ctrl = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert_eq!(action(ctrl('c')), Some(Action::Quit));
        assert_eq!(action(ctrl('r')), Some(Action::Redo));
    }

    #[test]
    fn alt_keys_do_nothing_and_ctrl_h_clears() {
        let alt_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT);
        assert_eq!(action(alt_h), None);
        let ctrl_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);
        assert_eq!(action(ctrl_h), Some(Action::Clear));
    }

    #[test]
    fn the_up_and_down_label_names_keys_that_move_up_and_down() {
        let label: Vec<char> = up_down_label().chars().collect();
        assert_eq!(
            action(key(KeyCode::Char(label[0]))),
            Some(Action::Step(Direction::Down))
        );
        assert_eq!(
            action(key(KeyCode::Char(label[1]))),
            Some(Action::Step(Direction::Up))
        );
    }

    #[test]
    fn the_left_and_right_label_names_keys_that_move_left_and_right() {
        let label: Vec<char> = left_right_label().chars().collect();
        assert_eq!(
            action(key(KeyCode::Char(label[0]))),
            Some(Action::Step(Direction::Left))
        );
        assert_eq!(
            action(key(KeyCode::Char(label[1]))),
            Some(Action::Step(Direction::Right))
        );
    }

    #[test]
    fn every_filter_label_names_keys_that_ask_for_it() {
        for wanted in [
            FilterKey::Digit(1),
            FilterKey::TwoCandidates,
            FilterKey::Off,
        ] {
            for k in keys_of(filter_label(wanted)) {
                let got = filter_key(k).unwrap_or_else(|| panic!("{k:?} does nothing"));
                assert_eq!(
                    std::mem::discriminant(&got),
                    std::mem::discriminant(&wanted),
                    "{k:?}"
                );
            }
        }
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
        let ctrl_4 = KeyEvent::new(KeyCode::Char('4'), KeyModifiers::CONTROL);
        assert_eq!(filter_key(ctrl_4), None);
    }
}
