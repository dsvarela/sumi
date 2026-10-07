//! X-Wing (`N` = 2) and Swordfish (`N` = 3): `N` rows in which a digit's
//! spots all fall in the same `N` columns. The digit goes once in each of
//! those rows, so it fills those columns, and the rest of each column can't
//! have it. The same holds with rows and columns swapped.

use crate::digits::Digits;
use crate::grid::{Cell, House, col, row};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, choose};

pub struct Fish<const N: usize>;

impl<const N: usize> Technique for Fish<N> {
    fn name(&self) -> &'static str {
        match N {
            2 => "X-Wing",
            3 => "Swordfish",
            _ => unreachable!("sumi's fish are X-Wings and Swordfish"),
        }
    }

    fn level(&self) -> Level {
        Level::Hard
    }

    fn all(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for digit in 1..=9 {
            steps.extend(fish::<N>(p, digit, House::Row, House::Col, col));
            steps.extend(fish::<N>(p, digit, House::Col, House::Row, row));
        }
        steps
    }
}

/// Fish whose base lines are made by `base` and whose cover lines are made by
/// `cover`, with `cover_index` giving the cover line a cell is in.
fn fish<const N: usize>(
    p: &Position,
    digit: u8,
    base: fn(usize) -> House,
    cover: fn(usize) -> House,
    cover_index: fn(Cell) -> usize,
) -> Vec<Step> {
    // A line with one spot is a hidden single; one with more than N can't
    // fit in N cover lines.
    let lines: Vec<usize> = (0..9)
        .filter(|&i| (2..=N).contains(&p.spots(base(i), digit).len()))
        .collect();
    let only: Digits = [digit].into_iter().collect();
    let mut steps = Vec::new();
    for bases in choose(&lines, N) {
        let cells: Vec<Cell> = bases
            .iter()
            .flat_map(|&i| p.spots(base(i), digit))
            .collect();
        let mut covers: Vec<usize> = cells.iter().map(|&c| cover_index(c)).collect();
        covers.sort_unstable();
        covers.dedup();
        if covers.len() != N {
            continue;
        }
        let targets = covers
            .iter()
            .flat_map(|&i| cover(i).cells())
            .filter(|&c| !bases.iter().any(|&b| base(b).contains(c)));
        let houses = bases.iter().map(|&i| base(i));
        steps.extend(Step::removing(
            &Fish::<N>,
            p,
            only,
            houses.chain(covers.iter().map(|&i| cover(i))).collect(),
            cells,
            targets.map(|c| (c, only)).collect::<Vec<_>>(),
        ));
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{SWORDFISH_1, SWORDFISH_2, X_WING_1, X_WING_2, check};

    #[test]
    fn x_wing() {
        check(&Fish::<2>, &X_WING_1, "r4c5<>5");
        check(&Fish::<2>, &X_WING_2, "r2c4789,r5c34789<>1");
    }

    #[test]
    fn swordfish() {
        check(&Fish::<3>, &SWORDFISH_1, "r6c8,r7c1<>2");
        check(&Fish::<3>, &SWORDFISH_2, "r3c235,r6c235,r8c235,r9c23<>4");
    }
}
