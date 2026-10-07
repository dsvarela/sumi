//! The board: 81 cells, each empty or holding a digit from 1 to 9.

use std::fmt;
use std::str::FromStr;

use crate::digits::Digits;

/// Cells are numbered 0 to 80, row by row from the top left.
pub type Cell = usize;

pub const CELLS: usize = 81;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Grid {
    /// 0 is an empty cell.
    cells: [u8; CELLS],
}

impl Grid {
    pub const EMPTY: Grid = Grid { cells: [0; CELLS] };

    pub fn get(&self, cell: Cell) -> Option<u8> {
        match self.cells[cell] {
            0 => None,
            digit => Some(digit),
        }
    }

    /// Panics on a digit outside 1 to 9, which is a bug in the caller.
    pub fn set(&mut self, cell: Cell, digit: Option<u8>) {
        if let Some(d) = digit {
            assert!((1..=9).contains(&d), "digit {d} is not 1 to 9");
        }
        self.cells[cell] = digit.unwrap_or(0);
    }

    pub fn filled(&self) -> usize {
        self.cells.iter().filter(|&&d| d != 0).count()
    }

    /// Marks every cell whose digit is repeated in its row, column or box.
    pub fn conflicts(&self) -> [bool; CELLS] {
        let mut clash = [false; CELLS];
        for a in 0..CELLS {
            let Some(digit) = self.get(a) else { continue };
            for b in a + 1..CELLS {
                if self.get(b) == Some(digit) && sees(a, b) {
                    clash[a] = true;
                    clash[b] = true;
                }
            }
        }
        clash
    }

    /// Full and free of conflicts. In a puzzle with one solution that is the
    /// solution, so no comparison with it is needed.
    pub fn is_solved(&self) -> bool {
        self.filled() == CELLS && !self.conflicts().contains(&true)
    }

    /// The digits no placed digit in the cell's row, column or box rules out.
    /// The cell's own digit, if any, is ignored.
    pub fn possible(&self, cell: Cell) -> Digits {
        Digits::ALL.without(peers(cell).filter_map(|p| self.get(p)).collect())
    }
}

/// Two different cells that share a row, column or box.
pub fn sees(a: Cell, b: Cell) -> bool {
    a != b && (row(a) == row(b) || col(a) == col(b) || box_of(a) == box_of(b))
}

/// The 20 cells that share a row, column or box with `cell`.
pub fn peers(cell: Cell) -> impl Iterator<Item = Cell> {
    (0..CELLS).filter(move |&other| sees(cell, other))
}

/// Rows, columns and boxes are numbered from 0 here, boxes left to right
/// and top to bottom. Text for players counts from 1: r1c1 is cell 0, and
/// "box 1" is `box_of` 0.
pub fn row(cell: Cell) -> usize {
    cell / 9
}

pub fn col(cell: Cell) -> usize {
    cell % 9
}

pub fn box_of(cell: Cell) -> usize {
    row(cell) / 3 * 3 + col(cell) / 3
}

/// Reads 81 cells written as `1` to `9`, with `0` or `.` for an empty cell.
/// Whitespace is ignored, so a grid pasted as nine lines reads the same as one
/// line.
impl FromStr for Grid {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Grid, ParseError> {
        let mut grid = Grid::EMPTY;
        let mut count = 0;
        for c in s.chars().filter(|c| !c.is_whitespace()) {
            let digit = match c {
                '.' | '0' => 0,
                '1'..='9' => c as u8 - b'0',
                _ => {
                    return Err(ParseError::BadChar {
                        position: count,
                        found: c,
                    });
                }
            };
            if count < CELLS {
                grid.cells[count] = digit;
            }
            count += 1;
        }
        if count != CELLS {
            return Err(ParseError::WrongLength(count));
        }
        Ok(grid)
    }
}

/// One line of 81 characters, `.` for an empty cell.
impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &d in &self.cells {
            let c = if d == 0 { '.' } else { (b'0' + d) as char };
            fmt::Write::write_char(f, c)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Grid({self})")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    WrongLength(usize),
    /// `position` counts cells, not characters, so whitespace does not shift it.
    BadChar {
        position: usize,
        found: char,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::WrongLength(n) => write!(f, "expected 81 cells, found {n}"),
            ParseError::BadChar { position, found } => {
                write!(f, "cell {}: '{found}' is not 1 to 9, 0 or .", position + 1)
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    #[test]
    fn parse_and_display_round_trip() {
        let grid: Grid = LINE.parse().unwrap();
        assert_eq!(grid.to_string(), LINE);
        assert_eq!(grid.get(0), Some(5));
        assert_eq!(grid.get(2), None);
        assert_eq!(grid.filled(), 30);
    }

    #[test]
    fn parse_accepts_zeros_and_whitespace() {
        let spaced: String = LINE
            .replace('.', "0")
            .as_bytes()
            .chunks(9)
            .map(|row| std::str::from_utf8(row).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            spaced.parse::<Grid>().unwrap(),
            LINE.parse::<Grid>().unwrap()
        );
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert_eq!("123".parse::<Grid>(), Err(ParseError::WrongLength(3)));
        assert_eq!(
            format!("{LINE}.").parse::<Grid>(),
            Err(ParseError::WrongLength(82))
        );
        let bad = format!("x{}", &LINE[1..]);
        assert_eq!(
            bad.parse::<Grid>(),
            Err(ParseError::BadChar {
                position: 0,
                found: 'x'
            })
        );
    }

    #[test]
    fn houses_of_a_cell() {
        assert_eq!((row(0), col(0), box_of(0)), (0, 0, 0));
        assert_eq!((row(40), col(40), box_of(40)), (4, 4, 4));
        assert_eq!((row(80), col(80), box_of(80)), (8, 8, 8));
        assert_eq!((row(33), col(33), box_of(33)), (3, 6, 5));
    }

    #[test]
    fn sees_shares_a_row_column_or_box() {
        assert!(sees(0, 8)); // row
        assert!(sees(0, 72)); // column
        assert!(sees(0, 20)); // box, r3c3
        assert!(!sees(0, 21)); // r3c4: none of them
        assert!(!sees(5, 5));
    }

    #[test]
    fn conflicts_mark_both_cells_of_each_clash() {
        let mut grid: Grid = LINE.parse().unwrap();
        assert!(!grid.conflicts().contains(&true));

        grid.set(20, Some(5)); // r3c3, in box 1 with the 5 at r1c1
        let clash = grid.conflicts();
        let marked: Vec<usize> = (0..CELLS).filter(|&c| clash[c]).collect();
        assert_eq!(marked, vec![0, 20]);
    }

    #[test]
    fn solved_means_full_and_without_conflicts() {
        let solution: Grid =
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
                .parse()
                .unwrap();
        assert!(solution.is_solved());
        assert!(!LINE.parse::<Grid>().unwrap().is_solved());

        let mut changed = solution;
        changed.set(0, Some(3));
        assert!(!changed.is_solved());
    }

    #[test]
    fn peers_are_the_twenty_cells_a_cell_sees() {
        assert_eq!(peers(40).count(), 20);
        assert!(peers(40).all(|p| sees(40, p)));
    }

    #[test]
    fn possible_digits_leave_out_what_peers_hold() {
        let grid: Grid = LINE.parse().unwrap();
        // r1c3 sees 5 3 7 in row 1, 8 in column 3, 6 9 8 in box 1.
        let expected: Digits = [1, 2, 4].into_iter().collect();
        assert_eq!(grid.possible(2), expected);
        // A clue's own digit is not counted against it.
        assert!(grid.possible(0).contains(5));
    }

    #[test]
    #[should_panic(expected = "digit 0 is not 1 to 9")]
    fn set_rejects_zero_as_a_digit() {
        let mut grid = Grid::EMPTY;
        grid.set(0, Some(0));
    }
}
