//! Positions from HoDoKu's technique pages, each with the candidates shown
//! there: <https://hodoku.sourceforge.net/en/tech_intro.php>. The step each
//! page shows is written in the test that uses the position.

use crate::digits::Digits;
use crate::grid::{CELLS, Cell, Grid};
use crate::position::Position;
use crate::solver::{Solutions, solve};
use crate::technique::Technique;

/// Checks that `technique` finds the step HoDoKu shows, written as HoDoKu
/// writes it (`r9c6=6`, or `r1c78,r3c45<>1, r2c2<>9`), and that no step it
/// finds removes or places anything against the solution.
pub fn check(technique: &dyn Technique, example: &Example, expected: &str) {
    check_position(technique, &example.position(), expected);
}

/// The same check for a position a test has changed.
pub fn check_position(technique: &dyn Technique, p: &Position, expected: &str) {
    let Solutions::Unique(solution) = solve(p.grid()) else {
        panic!("the example has no unique solution");
    };
    let steps = technique.all(p);
    for step in &steps {
        for &(cell, digits) in &step.removals {
            let answer = solution.get(cell).unwrap();
            assert!(
                !digits.contains(answer),
                "{step:?} removes the answer from {cell}"
            );
        }
        if let Some((cell, digit)) = step.placement {
            assert_eq!(solution.get(cell), Some(digit), "{step:?}");
        }
    }
    let (placement, removals) = parse(expected);
    assert!(
        steps
            .iter()
            .any(|s| s.placement == placement && s.removals == removals),
        "{expected} not among {steps:#?}"
    );
}

/// What a step places, and what it removes.
type Effect = (Option<(Cell, u8)>, Vec<(Cell, Digits)>);

fn parse(text: &str) -> Effect {
    if let Some((cells, digit)) = text.split_once('=') {
        let digit = digit.parse().unwrap();
        return (Some((cells_of(cells)[0], digit)), Vec::new());
    }
    let mut removals: Vec<(Cell, Digits)> = Vec::new();
    for part in text.split(", ") {
        let (cells, digit) = part.split_once("<>").unwrap();
        let digit: u8 = digit.parse().unwrap();
        for group in cells.split(',') {
            for cell in cells_of(group) {
                match removals.iter_mut().find(|(c, _)| *c == cell) {
                    Some((_, set)) => set.insert(digit),
                    None => removals.push((cell, [digit].into_iter().collect())),
                }
            }
        }
    }
    removals.sort_by_key(|&(cell, _)| cell);
    (None, removals)
}

/// `r12c3` is r1c3 and r2c3.
fn cells_of(group: &str) -> Vec<Cell> {
    let (rows, cols) = group.trim_start_matches('r').split_once('c').unwrap();
    let mut cells = Vec::new();
    for r in rows.bytes() {
        for c in cols.bytes() {
            cells.push((r - b'1') as usize * 9 + (c - b'1') as usize);
        }
    }
    cells
}

pub struct Example {
    /// Placed digits, `.` for an empty cell.
    pub grid: &'static str,
    /// Each cell's candidates, or its digit, row by row.
    pub candidates: &'static str,
}

impl Example {
    pub fn position(&self) -> Position {
        let grid: Grid = self.grid.parse().unwrap();
        let cells: Vec<Digits> = self
            .candidates
            .split_whitespace()
            .map(|cell| cell.bytes().map(|b| b - b'0').collect())
            .collect();
        assert_eq!(cells.len(), CELLS);
        Position::with_candidates(&grid, std::array::from_fn(|c| cells[c]))
    }
}

/// HoDoKu `fh01`. Full House: r9c6=6
pub const FULL_HOUSE_1: Example = Example {
    grid: "8..739..637.465....4.182..9...6...4..543..61..6.5.....4..853.7....271.641..94...2",
    candidates: "
        8      12     125    7      3      9      1245   25     6
        3      7      129    4      6      5      128    28     18
        56     4      56     1      8      2      357    35     9
        279    12389  123789 6      129    78     235789 4      3578
        279    5      4      3      29     78     6      1      78
        279    6      123789 5      129    478    23789  2389   378
        4      29     269    8      5      3      19     7      1
        59     389    3589   2      7      1      3589   6      4
        1      38     35678  9      4      6      358    358    2",
};

/// HoDoKu `h101`. Hidden Single: r3c4=6
pub const HIDDEN_SINGLE_1: Example = Example {
    grid: ".28..7....16.83.7.....2.85113729.......73........463.729..7.......86.14....3..7..",
    candidates: "
        3459   2      8      14569  15     7      469    369    3469
        459    1      6      459    8      3      249    7      249
        3479   47     349    469    2      49     8      5      1
        1      3      7      2      9      58     456    68     4568
        45689  4568   2459   7      3      158    24569  12689  245689
        589    58     259    15     4      6      3      1289   7
        2      9      1345   145    7      145    56     368    3568
        357    57     35     8      6      259    1      4      2359
        4568   4568   145    3      15     12459  7      2689   25689",
};

/// HoDoKu `h102`. Hidden Single: r6c4=3
pub const HIDDEN_SINGLE_2: Example = Example {
    grid: "..2193........7...7...4..198.3...6...45...23...7...5.437..8...6...6........5341..",
    candidates: "
        456    568    2      1      9      3      478    45678  578
        14569  135689 14689  28     256    7      348    24568  2358
        7      3568   68     28     4      2568   38     1      9
        8      129    3      2479   1257   1259   6      79     17
        169    4      5      789    167    1689   2      3      178
        1269   1269   7      2389   126    12689  5      89     4
        3      7      149    29     8      129    49     2459   6
        12459  12589  1489   6      127    129    34789  245789 23578
        269    2689   689    5      3      4      1      2789   278",
};

/// HoDoKu `h201`. Hidden Pair: 1,9 in r57c9 => r5c9<>6
pub const HIDDEN_PAIR_1: Example = Example {
    grid: ".49132....81479...327685914.96.518...75.28....38.46..5853267...712894563964513...",
    candidates: "
        56   4    9    1    3    2    67   578  678
        56   8    1    4    7    9    236  235  26
        3    2    7    6    8    5    9    1    4
        24   9    6    37   5    1    8    2347 27
        14   7    5    39   2    8    1346 349  169
        12   3    8    79   4    6    127  279  5
        8    5    3    2    6    7    14   49   19
        7    1    2    8    9    4    5    6    3
        9    6    4    5    1    3    27   278  278",
};

/// HoDoKu `h202`. Hidden Pair: 4,7 in r1c12 => r1c2<>1, r1c12<>2, r1c12<>3, r1c12<>5, r1c12<>8, r1c1<>9
pub const HIDDEN_PAIR_2: Example = Example {
    grid: "....6........42736..673..4..94....68....964.76.7.5.9231......85.6..8.271..5.1..94",
    candidates: "
        2345789 1234578 12389   1589    6       1589    158     15      29
        589     158     189     1589    4       2       7       3       6
        2589    1258    6       7       3       1589    158     4       29
        235     9       4       123     27      137     15      6       8
        2358    12358   1238    1238    9       6       4       15      7
        6       18      7       148     5       148     9       2       3
        1       2347    239     23469   27      3479    36      8       5
        349     6       39      3459    8       3459    2       7       1
        2378    2378    5       236     1       37      36      9       4",
};

/// HoDoKu `lc101`. Locked Candidates Type 1 (Pointing): 5 in b1 => r3c7<>5
pub const POINTING_1: Example = Example {
    grid: "984........25...4...19.4..2..6.9723...36.2...2.9.3561.195768423427351896638..9751",
    candidates: "
        9     8     4     12    127   36    135   67    57
        37    67    2     5     178   36    139   4     789
        357   567   1     9     78    4     35    678   2
        58    145   6     148   9     7     2     3     458
        578   1457  3     6     148   2     59    78    45789
        2     47    9     48    3     5     6     1     478
        1     9     5     7     6     8     4     2     3
        4     2     7     3     5     1     8     9     6
        6     3     8     24    24    9     7     5     1",
};

/// HoDoKu `lc102`. Locked Candidates Type 1 (Pointing): 1 in b8 => r7c123789<>1
pub const POINTING_2: Example = Example {
    grid: "34...6.7..8....93...2.3..6.....1.....9736485......2...............6.8.9....923785",
    candidates: "
        3        4        159      1258     589      6        125      7        128
        1567     8        156      12457    457      157      9        3        124
        1579     157      2        14578    3        1579     145      6        148
        24568    2356     34568    578      1        579      2346     24       234679
        12       9        7        3        6        4        8        5        12
        14568    1356     134568   578      5789     2        1346     14       134679
        12456789 123567   1345689  1457     457      157      12346    124      12346
        12457    12357    1345     6        457      8        1234     9        1234
        146      16       146      9        2        3        7        8        5",
};

/// HoDoKu `lc201`. Locked Candidates Type 2 (Claiming): 7 in r2 => r3c2<>7
pub const CLAIMING_1: Example = Example {
    grid: "318..54.6...6.381...6.8.5.3864952137123476958795318264.3.5..78......73.5....39641",
    candidates: "
        3    1    8    27   29   5    4    279  6
        2459 457  279  6    249  3    8    1    29
        249  47   6    127  8    14   5    279  3
        8    6    4    9    5    2    1    3    7
        1    2    3    4    7    6    9    5    8
        7    9    5    3    1    8    2    6    4
        2469 3    129  5    246  14   7    8    29
        2469 48   129  128  246  7    3    29   5
        25   578  27   28   3    9    6    4    1",
};

/// HoDoKu `lc202`. Locked Candidates Type 2 (Claiming): 4 in c6 => r1c45,r2c45,r3c45<>4
pub const CLAIMING_2: Example = Example {
    grid: "762..8..198......615.....87478..3169526..98733198..425835..1692297685314641932758",
    candidates: "
        7      6      2      345    459    8      59     34     1
        9      8      34     123457 12457  47     25     34     6
        1      5      34     234    2469   46     29     8      7
        4      7      8      25     25     3      1      6      9
        5      2      6      14     14     9      8      7      3
        3      1      9      8      67     67     4      2      5
        8      3      5      47     47     1      6      9      2
        2      9      7      6      8      5      3      1      4
        6      4      1      9      3      2      7      5      8",
};

/// HoDoKu `n101`. Naked Single: r6c7=6
pub const NAKED_SINGLE_1: Example = Example {
    grid: "412736589......1.6568.1.37....85.21.1.......8.87.9.....3..7.8658...........9.84.1",
    candidates: "
        4      1      2      7      3      6      5      8      9
        379    79     39     245    248    2459   1      24     6
        5      6      8      24     1      249    3      7      24
        369    49     3469   8      5      347    2      1      347
        1      2459   3459   2346   246    2347   679    349    8
        23     8      7      12346  9      1234   6      345    34
        29     3      149    124    7      124    8      6      5
        8      24     146    123456 246    12345  79     239    237
        267    257    56     9      26     8      4      23     1",
};

/// HoDoKu `n201`. Naked Pair: 3,9 in r8c34 => r8c2<>3
pub const NAKED_PAIR_1: Example = Example {
    grid: "7..849.3.928135..64..267.896427839513974516288156923..2.4516.931....8.6.5....4.1.",
    candidates: "
        7    56   16   8    4    9    125  3    25
        9    2    8    1    3    5    47   47   6
        4    35   13   2    6    7    15   8    9
        6    4    2    7    8    3    9    5    1
        3    9    7    4    5    1    6    2    8
        8    1    5    6    9    2    3    47   47
        2    78   4    5    1    6    78   9    3
        1    37   39   39   27   8    2457 6    2457
        5    3678 369  39   27   4    278  1    27",
};

/// HoDoKu `n202`. Naked Pair: 8,9 in r4c5,r5c6 => r456c4,r56c5,r6c6<>8, r456c4,r56c5,r6c6<>9
pub const NAKED_PAIR_2: Example = Example {
    grid: "687..4523953..261414235697831...724676....3.5.2....7.1.96..1.3223.....57.7.....69",
    candidates: "
        6     8     7     19    19    4     5     2     3
        9     5     3     78    78    2     6     1     4
        1     4     2     3     5     6     9     7     8
        3     1     589   589   89    7     2     4     6
        7     6     489   12489 12489 89    3     89    5
        458   2     4589  45689 34689 3589  7     89    1
        458   9     6     4578  478   1     48    3     2
        2     3     148   4689  4689  89    148   5     7
        458   7     1458  2458  2348  358   148   6     9",
};
