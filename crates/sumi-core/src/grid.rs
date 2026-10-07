//! The board: 81 cells, each empty or holding a digit from 1 to 9.

use std::fmt;
use std::str::FromStr;

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
        let value = digit.unwrap_or(0);
        assert!(value <= 9, "digit {value} is not 1 to 9");
        self.cells[cell] = value;
    }

    pub fn filled(&self) -> usize {
        self.cells.iter().filter(|&&d| d != 0).count()
    }
}

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
}
