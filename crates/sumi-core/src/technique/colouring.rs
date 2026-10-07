//! Simple colouring: follow one digit through houses where it has only two
//! spots, giving the cells of each link opposite colours, A and B. Either
//! every A cell holds the digit or every B cell does.
//!
//! - Wrap: two cells of one colour see each other, so that colour is false
//!   and none of its cells holds the digit.
//! - Trap: a cell outside the chain sees an A cell and a B cell, so it
//!   can't hold the digit.

use crate::digits::Digits;
use crate::grid::{CELLS, Cell, House, sees};
use crate::position::Position;
use crate::step::Step;
use crate::technique::{Level, Technique, strong_links};

pub struct SimpleColouring;

impl Technique for SimpleColouring {
    fn name(&self) -> &'static str {
        "Simple colouring"
    }

    fn level(&self) -> Level {
        Level::Expert
    }

    fn score(&self) -> u32 {
        150
    }

    fn search(&self, p: &Position) -> Vec<Step> {
        let mut steps = Vec::new();
        for digit in 1..=9 {
            let only: Digits = [digit].into_iter().collect();
            let links = strong_links(p, digit, &House::ALL);
            let mut colour: [Option<bool>; CELLS] = [None; CELLS];
            for start in 0..CELLS {
                if colour[start].is_some()
                    || !links.iter().any(|&(a, b, _)| a == start || b == start)
                {
                    continue;
                }
                let chain = colour_chain(start, &links, &mut colour);
                let mut houses: Vec<House> = links
                    .iter()
                    .filter(|(a, _, _)| chain.contains(a))
                    .map(|&(_, _, h)| h)
                    .collect();
                houses.sort_unstable();
                houses.dedup();
                let side = |c: bool| -> Vec<Cell> {
                    chain
                        .iter()
                        .copied()
                        .filter(|&x| colour[x] == Some(c))
                        .collect()
                };
                for c in [true, false] {
                    let cells = side(c);
                    let wraps = cells.iter().any(|&a| cells.iter().any(|&b| sees(a, b)));
                    if wraps {
                        steps.extend(Step::removing(
                            &SimpleColouring,
                            p,
                            only,
                            houses.clone(),
                            chain.clone(),
                            cells.iter().map(|&x| (x, only)),
                        ));
                    }
                }
                let (a, b) = (side(true), side(false));
                let trapped = (0..CELLS).filter(|&x| {
                    !chain.contains(&x)
                        && a.iter().any(|&y| sees(x, y))
                        && b.iter().any(|&y| sees(x, y))
                });
                steps.extend(Step::removing(
                    &SimpleColouring,
                    p,
                    only,
                    houses,
                    chain.clone(),
                    trapped.map(|x| (x, only)).collect::<Vec<_>>(),
                ));
            }
        }
        steps
    }
}

/// Colours every cell linked to `start`, giving the two cells of each link
/// opposite colours, and returns them in cell order.
fn colour_chain(
    start: Cell,
    links: &[(Cell, Cell, House)],
    colour: &mut [Option<bool>; CELLS],
) -> Vec<Cell> {
    colour[start] = Some(true);
    let mut chain = vec![start];
    let mut next = 0;
    while next < chain.len() {
        let cell = chain[next];
        next += 1;
        for &(a, b, _) in links {
            let other = match cell {
                _ if cell == a => b,
                _ if cell == b => a,
                _ => continue,
            };
            if colour[other].is_none() {
                colour[other] = colour[cell].map(|c| !c);
                chain.push(other);
            }
        }
    }
    chain.sort_unstable();
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::technique::examples::{SIMPLE_COLOURING_1, SIMPLE_COLOURING_2, check};

    #[test]
    fn trap_removes_the_digit_from_cells_seeing_both_colours() {
        check(&SimpleColouring, &SIMPLE_COLOURING_1, "r13c9,r8c1<>3");
    }

    #[test]
    fn wrap_removes_the_digit_from_a_colour_that_sees_itself() {
        check(
            &SimpleColouring,
            &SIMPLE_COLOURING_2,
            "r1c9,r2c14,r39c2,r4c4,r5c3,r6c7,r7c6,r9c5<>8",
        );
    }
}
