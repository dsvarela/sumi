//! The menus: what each one lists and how keys move through it. Drawing is
//! in `ui`.

use sumi_core::technique::{Level, ORDER};

use crate::game::Direction;
use crate::input::Action;
use crate::learn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    /// Back to the game in progress, from the main menu.
    Continue(Progress),
    NewGame,
    /// Type in a puzzle, from the main menu.
    CustomPuzzle,
    Learn,
    Stats,
    /// A technique in the Learn list, by its place in sumi's order.
    Technique(usize),
    Lesson,
    Drill,
    Level(Level),
    /// Back to the game, from the pause menu.
    Resume,
    MainMenu,
    Settings,
    Quit,
    /// Keep the saved game rather than start a new one.
    Keep,
    /// Start the new game, replacing the saved one.
    Replace,
    /// The list of bookmarks, from the pause menu.
    Bookmarks,
    /// Back to the bookmark at `index`, oldest first, which has `left`
    /// cells left to fill.
    Bookmark {
        index: usize,
        left: usize,
    },
}

/// What the menus list depends on outside them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Context {
    /// The game there is to continue.
    pub progress: Option<Progress>,
    /// The cells left at each of its bookmarks, oldest first.
    pub bookmarks: Vec<usize>,
    /// The stats are turned on.
    pub stats: bool,
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
            Item::Technique(index) => {
                let technique = ORDER[index];
                format!(
                    "{} · {}",
                    technique.name(),
                    Item::Level(technique.level()).name()
                )
            }
            Item::Continue(Progress {
                level: Some(level),
                left,
            }) => format!("Continue · {}, {left} left", Item::Level(level).name()),
            Item::Continue(Progress { level: None, left }) => format!("Continue · {left} left"),
            Item::Bookmark { index, left } => format!("Bookmark {} · {left} left", index + 1),
            item => item.name().to_string(),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Item::Continue(_) => "Continue",
            Item::NewGame => "New game",
            Item::CustomPuzzle => "Custom puzzle",
            Item::Learn => "Learn",
            Item::Stats => "Stats",
            Item::Technique(_) => "Technique",
            Item::Lesson => "Lesson",
            Item::Drill => "Drill",
            Item::Level(Level::Easy) => "Easy",
            Item::Level(Level::Medium) => "Medium",
            Item::Level(Level::Hard) => "Hard",
            Item::Level(Level::Expert) => "Expert",
            Item::Level(Level::Beyond) => "Beyond",
            Item::Resume => "Resume",
            Item::MainMenu => "Main menu",
            Item::Settings => "Settings",
            Item::Quit => "Quit",
            Item::Keep => "Keep it",
            Item::Replace => "Replace it",
            Item::Bookmarks => "Bookmarks",
            Item::Bookmark { .. } => "Bookmark",
        }
    }
}

/// Which menu is open. The level list remembers which menu it came from,
/// so Esc goes back there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Main,
    Pause,
    Levels {
        from_pause: bool,
    },
    /// A new game was asked for while a game is saved.
    Replace,
    /// The game's bookmarks, newest first.
    Bookmarks,
    /// Every technique, to learn.
    Learn,
    /// What there is to learn of the technique at this place in sumi's
    /// order: its lesson, if it has one, and its drill.
    Technique(usize),
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
            // Replace's first item keeps the game: Enter pressed in a hurry
            // loses nothing.
            Kind::Main
            | Kind::Pause
            | Kind::Replace
            | Kind::Bookmarks
            | Kind::Learn
            | Kind::Technique(_) => 0,
        };
        Menu { kind, selected }
    }

    /// The items in order. Continue is there only while a game is in
    /// progress, Bookmarks only while it has some, Stats only when turned
    /// on, and Lesson only for a technique with one.
    pub fn items(&self, context: &Context) -> Vec<Item> {
        match self.kind {
            Kind::Main => {
                let mut items = Vec::new();
                if let Some(progress) = context.progress {
                    items.push(Item::Continue(progress));
                }
                items.extend([Item::NewGame, Item::CustomPuzzle, Item::Learn]);
                if context.stats {
                    items.push(Item::Stats);
                }
                items.extend([Item::Settings, Item::Quit]);
                items
            }
            Kind::Pause => {
                let mut items = vec![Item::Resume];
                if !context.bookmarks.is_empty() {
                    items.push(Item::Bookmarks);
                }
                items.extend([Item::NewGame, Item::Settings, Item::MainMenu]);
                items
            }
            Kind::Bookmarks => (context.bookmarks.iter().enumerate())
                .rev()
                .map(|(index, &left)| Item::Bookmark { index, left })
                .collect(),
            Kind::Levels { .. } => LEVELS.map(Item::Level).to_vec(),
            Kind::Replace => vec![Item::Keep, Item::Replace],
            Kind::Learn => (0..ORDER.len()).map(Item::Technique).collect(),
            Kind::Technique(index) => {
                let mut items = Vec::new();
                if learn::lesson(ORDER[index]).is_some() {
                    items.push(Item::Lesson);
                }
                items.push(Item::Drill);
                items
            }
        }
    }

    /// The words over the items, in place of the logo, for a menu that
    /// asks a question.
    pub fn question(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Replace => Some("Replace the saved game?"),
            Kind::Technique(index) => Some(ORDER[index].name()),
            _ => None,
        }
    }

    /// Whether Esc leaves this menu. The main menu is where Esc leads.
    pub fn has_back(&self) -> bool {
        self.kind != Kind::Main
    }

    /// Up and down move the selection, wrapping; Enter chooses; Esc goes
    /// back. Other keys do nothing.
    pub fn handle(&mut self, action: Action, context: &Context) -> Option<Choice> {
        let items = self.items(context);
        let count = items.len();
        if count == 0 {
            return (action == Action::Close && self.has_back()).then_some(Choice::Back);
        }
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

    fn playing() -> Context {
        Context {
            progress: PROGRESS,
            ..Context::default()
        }
    }

    #[test]
    fn continue_shows_only_with_a_game_in_progress() {
        let menu = Menu::new(Kind::Main);
        assert_eq!(
            menu.items(&Context::default()),
            [
                Item::NewGame,
                Item::CustomPuzzle,
                Item::Learn,
                Item::Settings,
                Item::Quit
            ]
        );
        assert_eq!(
            menu.items(&playing()),
            [
                Item::Continue(PROGRESS.unwrap()),
                Item::NewGame,
                Item::CustomPuzzle,
                Item::Learn,
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

    /// A technique's menu names it over its Lesson and Drill.
    #[test]
    fn a_techniques_menu_names_it() {
        assert_eq!(
            Menu::new(Kind::Technique(3)).question(),
            Some("Pointing pair")
        );
        assert_eq!(
            Menu::new(Kind::Technique(3)).items(&Context::default()),
            [Item::Lesson, Item::Drill]
        );
    }

    /// Bookmarks is in the pause menu only when there are some, and its
    /// list has the newest first.
    #[test]
    fn bookmarks_are_listed_newest_first() {
        let marked = Context {
            bookmarks: vec![51, 40],
            ..playing()
        };
        assert_eq!(Menu::new(Kind::Pause).items(&playing())[1], Item::NewGame);
        assert_eq!(Menu::new(Kind::Pause).items(&marked)[1], Item::Bookmarks);
        let list = Menu::new(Kind::Bookmarks).items(&marked);
        assert_eq!(
            list,
            [
                Item::Bookmark { index: 1, left: 40 },
                Item::Bookmark { index: 0, left: 51 }
            ]
        );
        assert_eq!(list[0].label(), "Bookmark 2 · 40 left");
    }

    /// A list with nothing in it can only be left.
    #[test]
    fn an_empty_list_can_only_be_left() {
        let mut empty = Menu::new(Kind::Bookmarks);
        for action in [Action::Step(Direction::Down), Action::Confirm] {
            assert_eq!(empty.handle(action, &Context::default()), None);
        }
        assert_eq!(
            empty.handle(Action::Close, &Context::default()),
            Some(Choice::Back)
        );
    }

    #[test]
    fn up_and_down_wrap_and_enter_chooses() {
        let mut menu = Menu::new(Kind::Pause);
        assert_eq!(menu.handle(Action::Step(Direction::Up), &playing()), None);
        assert_eq!(menu.items(&playing())[menu.selected], Item::MainMenu);
        menu.handle(Action::Step(Direction::Down), &playing());
        menu.handle(Action::Step(Direction::Down), &playing());
        assert_eq!(
            menu.handle(Action::Confirm, &playing()),
            Some(Choice::Chosen(Item::NewGame))
        );
        assert_eq!(menu.handle(Action::Digit(3), &playing()), None);
    }

    #[test]
    fn esc_goes_back_from_every_menu_but_the_main_one() {
        assert_eq!(
            Menu::new(Kind::Main).handle(Action::Close, &Context::default()),
            None
        );
        assert_eq!(
            Menu::new(Kind::Pause).handle(Action::Close, &playing()),
            Some(Choice::Back)
        );
        let mut levels = Menu::new(Kind::Levels { from_pause: false });
        assert_eq!(
            levels.handle(Action::Close, &Context::default()),
            Some(Choice::Back)
        );
    }

    #[test]
    fn the_level_list_starts_at_medium() {
        let mut levels = Menu::new(Kind::Levels { from_pause: true });
        assert_eq!(
            levels.handle(Action::Confirm, &playing()),
            Some(Choice::Chosen(Item::Level(Level::Medium)))
        );
    }
}
