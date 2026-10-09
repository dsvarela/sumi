//! A puzzle shared by the tests, with its solution.

use sumi_core::grid::Grid;
use sumi_core::puzzle::Puzzle;

use crate::game::Game;

pub const CLUES: &str =
    "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
pub const SOLUTION: &str =
    "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

pub fn game() -> Game {
    Game::new(Puzzle::new(CLUES.parse().unwrap()).unwrap())
}

pub fn solution() -> Grid {
    SOLUTION.parse().unwrap()
}
