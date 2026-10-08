//! The menus: what each one lists and how keys move through it. Drawing is
//! in `ui`.

use sumi_core::technique::Level;

use crate::game::Direction;
use crate::input::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    /// Back to the game in progress, from the main menu.
    Continue,
    NewGame,
    Level(Level),
    /// Back to the game, from the pause menu.
    Resume,
    MainMenu,
    Settings,
    Quit,
}

impl Item {
    pub fn label(self) -> &'static str {
        match self {
            Item::Continue => "Continue",
            Item::NewGame => "New game",
            Item::Level(Level::Easy) => "Easy",
            Item::Level(Level::Medium) => "Medium",
            Item::Level(Level::Hard) => "Hard",
            Item::Level(Level::Expert) => "Expert",
            Item::Level(Level::Beyond) => "Beyond",
            Item::Resume => "Resume",
            Item::MainMenu => "Main menu",
            Item::Settings => "Settings",
            Item::Quit => "Quit",
        }
    }
}

/// Which menu is open. The level list remembers which menu it came from,
/// so Esc goes back there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Main,
    Pause,
    Levels { from_pause: bool },
}

/// What a key in a menu asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Chosen(Item),
    Back,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub kind: Kind,
    /// The selected item's place in `items`.
    pub selected: usize,
}

const LEVELS: [Level; 4] = [Level::Easy, Level::Medium, Level::Hard, Level::Expert];

impl Menu {
    pub fn new(kind: Kind) -> Menu {
        let selected = match kind {
            // Medium: a middle ground for someone who hasn't said.
            Kind::Levels { .. } => 1,
            Kind::Main | Kind::Pause => 0,
        };
        Menu { kind, selected }
    }

    /// The items in order. Continue is there only while a game is in
    /// progress.
    pub fn items(&self, in_progress: bool) -> Vec<Item> {
        match self.kind {
            Kind::Main => {
                let mut items = Vec::new();
                if in_progress {
                    items.push(Item::Continue);
                }
                items.extend([Item::NewGame, Item::Settings, Item::Quit]);
                items
            }
            Kind::Pause => vec![Item::Resume, Item::NewGame, Item::Settings, Item::MainMenu],
            Kind::Levels { .. } => LEVELS.map(Item::Level).to_vec(),
        }
    }

    /// Whether Esc leaves this menu. The main menu is where Esc leads.
    pub fn has_back(&self) -> bool {
        self.kind != Kind::Main
    }

    /// Up and down move the selection, wrapping; Enter chooses; Esc goes
    /// back. Other keys do nothing.
    pub fn handle(&mut self, action: Action, in_progress: bool) -> Option<Choice> {
        let count = self.items(in_progress).len();
        match action {
            Action::Step(Direction::Up) => self.selected = (self.selected + count - 1) % count,
            Action::Step(Direction::Down) => self.selected = (self.selected + 1) % count,
            Action::Confirm => return Some(Choice::Chosen(self.items(in_progress)[self.selected])),
            Action::Close if self.has_back() => return Some(Choice::Back),
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_shows_only_with_a_game_in_progress() {
        let menu = Menu::new(Kind::Main);
        assert_eq!(
            menu.items(false),
            [Item::NewGame, Item::Settings, Item::Quit]
        );
        assert_eq!(
            menu.items(true),
            [Item::Continue, Item::NewGame, Item::Settings, Item::Quit]
        );
    }

    #[test]
    fn up_and_down_wrap_and_enter_chooses() {
        let mut menu = Menu::new(Kind::Pause);
        assert_eq!(menu.handle(Action::Step(Direction::Up), true), None);
        assert_eq!(menu.items(true)[menu.selected], Item::MainMenu);
        menu.handle(Action::Step(Direction::Down), true);
        menu.handle(Action::Step(Direction::Down), true);
        assert_eq!(
            menu.handle(Action::Confirm, true),
            Some(Choice::Chosen(Item::NewGame))
        );
        assert_eq!(menu.handle(Action::Digit(3), true), None);
    }

    #[test]
    fn esc_goes_back_from_every_menu_but_the_main_one() {
        assert_eq!(Menu::new(Kind::Main).handle(Action::Close, false), None);
        assert_eq!(
            Menu::new(Kind::Pause).handle(Action::Close, true),
            Some(Choice::Back)
        );
        let mut levels = Menu::new(Kind::Levels { from_pause: false });
        assert_eq!(levels.handle(Action::Close, false), Some(Choice::Back));
    }

    #[test]
    fn the_level_list_starts_at_medium() {
        let mut levels = Menu::new(Kind::Levels { from_pause: true });
        assert_eq!(
            levels.handle(Action::Confirm, true),
            Some(Choice::Chosen(Item::Level(Level::Medium)))
        );
    }
}
