//! How hint texts write cells, houses, digits and lists. The plan's Wording
//! section decides the forms: `r2c3`, "rows 2 and 7", "1, 4 and 6".

use crate::digits::Digits;
use crate::grid::{Cell, House, box_of, col, row};

/// `r2c3`, counting from 1.
pub fn cell(cell: Cell) -> String {
    format!("r{}c{}", row(cell) + 1, col(cell) + 1)
}

/// The text with its first letter in capitals.
pub fn capital(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// "a", "a and b", "a, b and c".
pub fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// Up to three cells by name. More are a count and the row, column or box
/// they share, since a long list does not fit the panel.
pub fn cells(cells: &[Cell]) -> String {
    cells_up_to(cells, 3)
}

/// The cells by name when there are at most `most`, otherwise a count and
/// the row, column or box they share.
pub fn cells_up_to(cells: &[Cell], most: usize) -> String {
    if cells.len() <= most {
        return list(&cells.iter().map(|&c| cell(c)).collect::<Vec<_>>());
    }
    let shared = [
        House::Row(row(cells[0])),
        House::Col(col(cells[0])),
        House::Box(box_of(cells[0])),
    ]
    .into_iter()
    .find(|h| cells.iter().all(|&c| h.contains(c)));
    match shared {
        Some(house) => format!("{} cells in {}", cells.len(), self::house(house)),
        None => format!("{} cells", cells.len()),
    }
}

/// `row 4`, `column 2` or `box 5`.
pub fn house(house: House) -> String {
    match house {
        House::Row(i) => format!("row {}", i + 1),
        House::Col(i) => format!("column {}", i + 1),
        House::Box(i) => format!("box {}", i + 1),
    }
}

/// The houses in the order given: "box 1, row 3 and column 5".
pub fn house_list(houses: &[House]) -> String {
    list(&houses.iter().map(|&h| house(h)).collect::<Vec<_>>())
}

/// "rows 2 and 7", or "rows 2 and 4 and column 3" for mixed kinds: one
/// phrase per kind, rows first.
pub fn houses(houses: &[House]) -> String {
    let rows = houses.iter().filter_map(|h| match h {
        House::Row(i) => Some(*i),
        _ => None,
    });
    let cols = houses.iter().filter_map(|h| match h {
        House::Col(i) => Some(*i),
        _ => None,
    });
    let boxes = houses.iter().filter_map(|h| match h {
        House::Box(i) => Some(*i),
        _ => None,
    });
    let parts: Vec<String> = [
        kind("row", rows.collect()),
        kind("column", cols.collect()),
        kind("box", boxes.collect()),
    ]
    .into_iter()
    .flatten()
    .collect();
    list(&parts)
}

/// "rows 2 and 7" for the zero-based numbers 1 and 6; nothing for none.
fn kind(name: &str, numbers: Vec<usize>) -> Option<String> {
    let plural = if numbers.len() == 1 { "" } else { "s" };
    let shown: Vec<String> = numbers.iter().map(|n| (n + 1).to_string()).collect();
    (!numbers.is_empty()).then(|| format!("{name}{plural} {}", list(&shown)))
}

/// "1, 4 and 6".
pub fn digits(digits: Digits) -> String {
    list(&digits.iter().map(|d| d.to_string()).collect::<Vec<_>>())
}

/// "Remove 4 from r5c3 and r9c8." Cells that lose different digits are only
/// counted. Past two cells they are counted too, which keeps the effect and its reason
/// within four lines of the panel; the board shows the cells.
pub fn removing(removals: &[(Cell, Digits)]) -> String {
    let mut groups: Vec<(Digits, Vec<Cell>)> = Vec::new();
    for &(c, d) in removals {
        match groups.iter_mut().find(|(set, _)| *set == d) {
            Some((_, cells)) => cells.push(c),
            None => groups.push((d, vec![c])),
        }
    }
    if groups.len() > 1 {
        let cells: Vec<Cell> = removals.iter().map(|&(c, _)| c).collect();
        let count: usize = removals.iter().map(|(_, d)| d.len()).sum();
        return format!("Remove {count} candidates from {}.", cells_up_to(&cells, 2));
    }
    let parts: Vec<String> = groups
        .iter()
        .map(|(d, c)| format!("{} from {}", digits(*d), cells_up_to(c, 2)))
        .collect();
    format!("Remove {}.", parts.join(" and "))
}

/// "r2c2 sees both", "these cells see both".
pub fn see_all(cells: &[Cell], all: &str) -> String {
    match cells {
        [only] => format!("{} sees {all}", cell(*only)),
        _ => format!("these cells see {all}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(list: &[u8]) -> Digits {
        list.iter().copied().collect()
    }

    #[test]
    fn cells_are_written_row_then_column_from_1() {
        assert_eq!(cell(0), "r1c1");
        assert_eq!(cell(2 * 9 + 2), "r3c3");
        assert_eq!(cell(80), "r9c9");
    }

    #[test]
    fn lists_read_as_english() {
        let s = |x: &[&str]| x.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(list(&s(&["a"])), "a");
        assert_eq!(list(&s(&["a", "b"])), "a and b");
        assert_eq!(list(&s(&["a", "b", "c"])), "a, b and c");
    }

    #[test]
    fn long_cell_lists_become_a_count_and_the_house() {
        assert_eq!(cells(&[1, 2, 3]), "r1c2, r1c3 and r1c4");
        assert_eq!(cells(&[1, 2, 3, 4]), "4 cells in row 1");
        assert_eq!(cells(&[7, 16, 25, 34]), "4 cells in column 8");
        assert_eq!(cells(&[0, 1, 9, 10]), "4 cells in box 1");
        assert_eq!(cells(&[0, 40, 41, 80]), "4 cells");
    }

    #[test]
    fn houses_are_plural_by_kind() {
        assert_eq!(houses(&[House::Row(1), House::Row(6)]), "rows 2 and 7");
        assert_eq!(houses(&[House::Box(4)]), "box 5");
        assert_eq!(
            houses(&[House::Col(2), House::Row(1), House::Row(3)]),
            "rows 2 and 4 and column 3"
        );
    }

    #[test]
    fn removals_of_different_digits_are_counted() {
        assert_eq!(
            removing(&[(29, set(&[4])), (35, set(&[4]))]),
            "Remove 4 from r4c3 and r4c9."
        );
        assert_eq!(
            removing(&[(0, set(&[1])), (1, set(&[2]))]),
            "Remove 2 candidates from r1c1 and r1c2."
        );
        assert_eq!(
            removing(&[(0, set(&[1])), (1, set(&[2])), (2, set(&[3, 4]))]),
            "Remove 4 candidates from 3 cells in row 1."
        );
    }
}
