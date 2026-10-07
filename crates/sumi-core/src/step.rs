//! A step a technique found: what it is about, and what it places or removes.

use std::fmt;

use crate::digits::Digits;
use crate::grid::{Cell, House};
use crate::position::Position;
use crate::technique::Technique;

/// The hint ladder reveals these fields one rung at a time, and its wording is
/// made from them rather than stored here.
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
        })
    }
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
