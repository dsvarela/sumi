//! The menus: what each one lists and how keys move through it. Drawing is
//! in `ui`.

use sumi_core::technique::Level;

use crate::game::Direction;
use crate::input::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    /// Back to the game in progress, from the main menu.
    Continue(Progress),
    NewGame,
    Level(Level),
    /// Back to the game, from the pause menu.
    Resume,
    MainMenu,
    Settings,
    Quit,
}

/// How far the game in progress is: its level, when it is rated, and the
/// cells left to fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub level: Option<Level>,
    pub left: usize,
}

impl Item {
    pub fn label(self) -> String {
        match self {
            Item::Continue(Progress {
                level: Some(level),
                left,
            }) => format!("Continue · {}, {left} left", Item::Level(level).name()),
            Item::Continue(Progress { level: None, left }) => format!("Continue · {left} left"),
            item => item.name().to_string(),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Item::Continue(_) => "Continue",
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
    pub fn items(&self, progress: Option<Progress>) -> Vec<Item> {
        match self.kind {
            Kind::Main => {
                let mut items = Vec::new();
                if let Some(progress) = progress {
                    items.push(Item::Continue(progress));
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
    pub fn handle(&mut self, action: Action, progress: Option<Progress>) -> Option<Choice> {
        let items = self.items(progress);
        let count = items.len();
        match action {
            Action::Step(Direction::Up) => self.selected = (self.selected + count - 1) % count,
            Action::Step(Direction::Down) => self.selected = (self.selected + 1) % count,
            Action::Confirm => return Some(Choice::Chosen(items[self.selected])),
            Action::Close if self.has_back() => return Some(Choice::Back),
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRESS: Option<Progress> = Some(Progress {
        level: Some(Level::Hard),
        left: 34,
    });

    #[test]
    fn continue_shows_only_with_a_game_in_progress() {
        let menu = Menu::new(Kind::Main);
        assert_eq!(
            menu.items(None),
            [Item::NewGame, Item::Settings, Item::Quit]
        );
        assert_eq!(
            menu.items(PROGRESS),
            [
                Item::Continue(PROGRESS.unwrap()),
                Item::NewGame,
                Item::Settings,
                Item::Quit
            ]
        );
    }

    #[test]
    fn continue_says_the_level_and_the_cells_left() {
        assert_eq!(
            Item::Continue(PROGRESS.unwrap()).label(),
            "Continue · Hard, 34 left"
        );
        let unrated = Progress {
            level: None,
            left: 7,
        };
        assert_eq!(Item::Continue(unrated).label(), "Continue · 7 left");
    }

    #[test]
    fn up_and_down_wrap_and_enter_chooses() {
        let mut menu = Menu::new(Kind::Pause);
        assert_eq!(menu.handle(Action::Step(Direction::Up), PROGRESS), None);
        assert_eq!(menu.items(PROGRESS)[menu.selected], Item::MainMenu);
        menu.handle(Action::Step(Direction::Down), PROGRESS);
        menu.handle(Action::Step(Direction::Down), PROGRESS);
        assert_eq!(
            menu.handle(Action::Confirm, PROGRESS),
            Some(Choice::Chosen(Item::NewGame))
        );
        assert_eq!(menu.handle(Action::Digit(3), PROGRESS), None);
    }

    #[test]
    fn esc_goes_back_from_every_menu_but_the_main_one() {
        assert_eq!(Menu::new(Kind::Main).handle(Action::Close, None), None);
        assert_eq!(
            Menu::new(Kind::Pause).handle(Action::Close, PROGRESS),
            Some(Choice::Back)
        );
        let mut levels = Menu::new(Kind::Levels { from_pause: false });
        assert_eq!(levels.handle(Action::Close, None), Some(Choice::Back));
    }

    #[test]
    fn the_level_list_starts_at_medium() {
        let mut levels = Menu::new(Kind::Levels { from_pause: true });
        assert_eq!(
            levels.handle(Action::Confirm, PROGRESS),
            Some(Choice::Chosen(Item::Level(Level::Medium)))
        );
    }
}
