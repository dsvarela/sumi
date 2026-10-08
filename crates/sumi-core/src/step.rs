//! A step a technique found: what it is about, and what it places or removes.

use std::fmt;

use crate::digits::Digits;
use crate::grid::{Cell, House};
use crate::position::Position;
use crate::technique::Technique;

/// These are the fields the hint ladder is to reveal one rung at a time; its
/// wording is made from them rather than stored here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub technique: &'static dyn Technique,
    /// The digits the pattern is about.
    pub digits: Digits,
    /// The rows, columns and boxes the pattern lies in.
    pub houses: Vec<House>,
    /// The cells that make the pattern, in cell order.
    pub cells: Vec<Cell>,
    /// Candidates the step removes, one entry per cell, in cell order.
    pub removals: Vec<(Cell, Digits)>,
    pub placement: Option<(Cell, u8)>,
    /// The cells the explanation names apart from the rest of the pattern,
    /// such as a wing's pivot. Each technique says which, and in what order,
    /// in its `explain`; empty when the pattern's cells say it all.
    pub key_cells: Vec<Cell>,
}

impl Step {
    pub(crate) fn placing(
        technique: &'static dyn Technique,
        cell: Cell,
        digit: u8,
        houses: Vec<House>,
    ) -> Step {
        Step {
            technique,
            digits: [digit].into_iter().collect(),
            houses,
            cells: vec![cell],
            removals: Vec::new(),
            placement: Some((cell, digit)),
            key_cells: Vec::new(),
        }
    }

    /// Keeps only the removals that are candidates in `p`, and gives `None`
    /// when that leaves nothing to remove: a pattern that changes nothing is
    /// not a step.
    pub(crate) fn removing(
        technique: &'static dyn Technique,
        p: &Position,
        digits: Digits,
        houses: Vec<House>,
        mut cells: Vec<Cell>,
        removals: impl IntoIterator<Item = (Cell, Digits)>,
    ) -> Option<Step> {
        let mut merged: Vec<(Cell, Digits)> = Vec::new();
        for (cell, remove) in removals {
            let remove = remove.intersection(p.candidates(cell));
            match merged.iter_mut().find(|(c, _)| *c == cell) {
                Some((_, set)) => *set = set.union(remove),
                None => merged.push((cell, remove)),
            }
        }
        merged.retain(|(_, set)| !set.is_empty());
        if merged.is_empty() {
            return None;
        }
        merged.sort_by_key(|&(cell, _)| cell);
        cells.sort_unstable();
        cells.dedup();
        Some(Step {
            technique,
            digits,
            houses,
            cells,
            removals: merged,
            placement: None,
            key_cells: Vec::new(),
        })
    }

    /// Names the cells the explanation singles out; see `key_cells`.
    pub(crate) fn keyed(mut self, key_cells: Vec<Cell>) -> Step {
        debug_assert!(key_cells.iter().all(|c| self.cells.contains(c)));
        self.key_cells = key_cells;
        self
    }
}

/// What a hint says about a step once the player has been told its
/// technique. Written without the step's cells, which the hint's fourth rung
/// shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// Where to look, such as "On digit 4, rows 2 and 7."
    pub area: String,
    /// The houses the area names, which the board shades.
    pub area_houses: Vec<House>,
    /// What the step does: "Remove 4 from r5c3 and r9c8."
    pub effect: String,
    /// Why that is safe.
    pub reason: String,
    /// Pattern cells drawn apart from the rest, such as colouring's second
    /// group.
    pub second_group: Vec<Cell>,
}

impl fmt::Debug for dyn Technique {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Two techniques are the same when they have the same name.
impl PartialEq for dyn Technique {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for dyn Technique {}
