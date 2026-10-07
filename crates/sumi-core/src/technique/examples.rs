//! Positions from HoDoKu's technique pages, each with the candidates shown
//! there: <https://hodoku.sourceforge.net/en/tech_intro.php>. Each one's doc
//! quotes the step the page shows, and the test that uses it checks that
//! step.

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

/// HoDoKu `2sk01`. 2-String Kite: 5 in r2c7,r8c4 (connected by r8c9,r9c7) => r2c4<>5
pub const KITE_1: Example = Example {
    grid: ".81.2.6...42.6..89.568..24.69314275842835791617568932451..3689223...846.86.2.....",
    candidates: "
        379  8    1    4579 2    345  6    37   357
        37   4    2    57   6    135  15   8    9
        379  5    6    8    179  13   2    4    137
        6    9    3    1    4    2    7    5    8
        4    2    8    3    5    7    9    1    6
        1    7    5    6    8    9    3    2    4
        5    1    47   47   3    6    8    9    2
        2    3    79   579  179  8    4    6    157
        8    6    479  2    179  145  15   37   1357",
};

/// HoDoKu `2sk02`. 2-String Kite: 9 in r6c6,r7c2 (connected by r4c2,r6c1) => r7c6<>9
pub const KITE_2: Example = Example {
    grid: "3617..295842395671.5.2614831.8526.34625....18.341..5264..61.85258...2167216857349",
    candidates: "
        3   6   1   7   48  48  2   9   5
        8   4   2   3   9   5   6   7   1
        79  5   79  2   6   1   4   8   3
        1   79  8   5   2   6   79  3   4
        6   2   5   49  347 349 79  1   8
        79  3   4   1   78  89  5   2   6
        4   79  379 6   1   39  8   5   2
        5   8   39  49  34  2   1   6   7
        2   1   6   8   5   7   3   4   9",
};

/// HoDoKu `bf201`. X-Wing: 5 r25 c58 => r4c5<>5
pub const X_WING_1: Example = Example {
    grid: ".41729.3.769..34.2.3264.7194.39..17.6.7..49.319537..24214567398376.9.541958431267",
    candidates: "
        58   4    1    7    2    9    68   3    56
        7    6    9    18   158  3    4    58   2
        58   3    2    6    4    58   7    1    9
        4    28   3    9    58   2568 1    7    56
        6    28   7    128  158  4    9    58   3
        1    9    5    3    7    68   68   2    4
        2    1    4    5    6    7    3    9    8
        3    7    6    28   9    28   5    4    1
        9    5    8    4    3    1    2    6    7",
};

/// HoDoKu `bf202`. X-Wing: 1 c15 r25 => r2c4789,r5c34789<>1
pub const X_WING_2: Example = Example {
    grid: "98..62753.65..3...327.5...679..3.5...5...9...832.45..9673591428249.87..5518.2...7",
    candidates: "
        9      8      14     14     6      2      7      5      3
        14     6      5      14789  17     3      1289   1489   124
        3      2      7      1489   5      48     189    1489   6
        7      9      146    1268   3      68     5      1468   124
        14     5      146    12678  17     9      12368  134678 124
        8      3      2      167    4      5      16     167    9
        6      7      3      5      9      1      4      2      8
        2      4      9      36     8      7      136    136    5
        5      1      8      346    2      46     369    369    7",
};

/// HoDoKu `bf301`. Swordfish: 2 r239 c158 => r6c8,r7c1<>2
pub const SWORDFISH_1: Example = Example {
    grid: "16.543.7..786.14354358.76.172.458.696..912.57...376..4.16.3..4.3...8..16..71645.3",
    candidates: "
        1    6    29   5    4    3    289  7    28
        29   7    8    6    29   1    4    3    5
        4    3    5    8    29   7    6    29   1
        7    2    13   4    5    8    13   6    9
        6    48   34   9    1    2    38   5    7
        589  589  19   3    7    6    128  28   4
        2589 1    6    27   3    59   2789 4    28
        3    459  249  27   8    59   279  1    6
        289  89   7    1    6    4    5    289  3",
};

/// HoDoKu `bf302`. Swordfish: 4 r247 c235 => r3c235,r6c235,r8c235,r9c23<>4
pub const SWORDFISH_2: Example = Example {
    grid: "1.85..2345..3.2178...8..5698..6.5793..59..4813....865298.2.631.......8.....78.9..",
    candidates: "
        1       679     8       5       679     79      2       3       4
        5       469     469     3       469     2       1       7       8
        247     2347    2347    8       147     147     5       6       9
        8       124     124     6       124     5       7       9       3
        267     267     5       9       237     37      4       8       1
        3       1479    1479    14      147     8       6       5       2
        9       8       47      2       45      6       3       1       57
        2467    1234567 123467  14      13459   1349    8       24      567
        246     123456  12346   7       8       134     9       24      56",
};

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

/// HoDoKu `h301`. Hidden Triple: 2,4,5 in r89c2,r9c3 => r9c2<>1, r9c3<>6
pub const HIDDEN_TRIPLE_1: Example = Example {
    grid: "28....473534827196.71.34.8.3..5...4....34..6.46.79.31..9.2.3654..3..9821....8.937",
    candidates: "
        2     8     69    169   156   156   4     7     3
        5     3     4     8     2     7     1     9     6
        69    7     1     69    3     4     25    8     25
        3     12    2789  5     16    1268  27    4     289
        1789  125   25789 3     4     128   257   6     2589
        4     6     258   7     9     28    3     1     258
        178   9     78    2     17    3     6     5     4
        67    45    3     46    567   9     8     2     1
        16    1245  256   146   8     156   9     3     7",
};

/// HoDoKu `h302`. Hidden Triple: 2,5,6 in r468c6 => r468c6<>1, r8c6<>3, r468c6<>4, r4c6<>7, r468c6<>8
pub const HIDDEN_TRIPLE_2: Example = Example {
    grid: "5..62..37..489........5....93........2....6.57.......3.....9.........7..68.57...2",
    candidates: "
        5       19      189     6       2       14      1489    3       7
        123     167     4       8       9       137     125     1256    16
        1238    1679    1236789 1347    5       1347    12489   124689  14689
        9       3       1568    1247    1468    1245678 1248    12478   148
        148     2       18      13479   1348    13478   6       14789   5
        7       1456    1568    1249    1468    124568  12489   12489   3
        1234    1457    12357   1234    13468   9       13458   14568   1468
        1234    1459    12359   1234    13468   123468  7       145689  14689
        6       8       139     5       7       134     1349    149     2",
};

/// HoDoKu `h401`. Hidden Quadruple: 2,4,5,8 in r7c56,r8c56 => r7c5<>3, r78c5<>6
pub const HIDDEN_QUAD_1: Example = Example {
    grid: "816573294392......4572.9..6941...5687854961236238...4.279.....1138....7.564....82",
    candidates: "
        8     1     6     5     7     3     2     9     4
        3     9     2     16    1468  148   78    15    57
        4     5     7     2     18    9     38    13    6
        9     4     1     37    23    27    5     6     8
        7     8     5     4     9     6     1     2     3
        6     2     3     8     15    15    79    4     79
        2     7     9     36    34568 458   46    35    1
        1     3     8     69    2456  245   46    7     59
        5     6     4     1379  13    17    39    8     2",
};

/// HoDoKu `h402`. Hidden Quadruple: 2,4,5,9 in r1238c9 => r238c9<>3, r128c9<>6, r123c9<>7, r18c9<>8
pub const HIDDEN_QUAD_2: Example = Example {
    grid: ".3.....1...8.9....4..6.8......57694....98352....124...276..519....7.9....95...47.",
    candidates: "
        5679    3       279     24      45      27      2678    1       2456789
        567     1256    8       234     9       127     2367    356     234567
        4       125     1279    6       135     8       237     35      23579
        138     128     123     5       7       6       9       4       138
        67      146     147     9       8       3       5       2       167
        5679    568     379     1       2       4       3678    368     3678
        2       7       6       348     34      5       1       9       38
        138     148     134     7       136     9       2368    3568    23568
        138     9       5       238     136     12      4       7       368",
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

/// HoDoKu `n301`. Naked Triple: 3,9,6 in r245c2 => r1c2<>6
pub const NAKED_TRIPLE_1: Example = Example {
    grid: "...29438....17864.48.3561....48375.1...4157..5..629834953782416126543978.4.961253",
    candidates: "
        67   167  157  2    9    4    3    8    57
        23   39   259  1    7    8    6    4    259
        4    8    279  3    5    6    1    29   279
        26   69   4    8    3    7    5    269  1
        2368 369  289  4    1    5    7    269  29
        5    17   17   6    2    9    8    3    4
        9    5    3    7    8    2    4    1    6
        1    2    6    5    4    3    9    7    8
        78   4    78   9    6    1    2    5    3",
};

/// HoDoKu `n302`. Naked Triple: 1,2,6 in r13c5,r3c4 => r1c46,r2c456,r3c6<>1, r1c46,r2c456,r3c6<>2, r1c46,r3c6<>6
pub const NAKED_TRIPLE_2: Example = Example {
    grid: "39....7........65.5.7...349.4938.5.66.1.54983853...4..9..8..134..294.8654.....297",
    candidates: "
        3      9      468    12456  126    12568  7      12     128
        12     128    48     1247   12379  123789 6      5      128
        5      1268   7      126    126    1268   3      4      9
        27     4      9      3      8      127    5      127    6
        6      27     1      27     5      4      9      8      3
        8      5      3      1267   12679  12679  4      127    12
        9      67     56     8      267    2567   1      3      4
        17     137    2      9      4      137    8      6      5
        4      1368   568    156    136    1356   2      9      7",
};

/// HoDoKu `n401`. Naked Quadruple: 3,4,8,9 in r8c1346 => r8c7<>3, r8c7<>8, r8c8<>9
pub const NAKED_QUAD_1: Example = Example {
    grid: ".1.72.563.56.3.247732546189693287415247615938581394........2...........1..587....",
    candidates: "
        489   1     489   7     2     89    5     6     3
        89    5     6     19    3     189   2     4     7
        7     3     2     5     4     6     1     8     9
        6     9     3     2     8     7     4     1     5
        2     4     7     6     1     5     9     3     8
        5     8     1     3     9     4     67    27    26
        13489 67    489   149   56    2     3678  579   46
        3489  267   489   49    56    39    3678  2579  1
        1349  26    5     8     7     139   36    29    246",
};

/// HoDoKu `n402`. Naked Quadruple: 4,6,7,9 in r789c3,r8c2 => r7c12,r89c1,r9c2<>4, r7c12,r9c12<>6, r7c2<>9
pub const NAKED_QUAD_2: Example = Example {
    grid: "532786...978241.6...1953287.254..67...3617.527..5........1........8.51.6...3...98",
    candidates: "
        5     3     2     7     8     6     49    14    149
        9     7     8     2     4     1     35    6     35
        46    46    1     9     5     3     2     8     7
        18    2     5     4     39    89    6     7     139
        48    489   3     6     1     7     489   5     2
        7     14689 469   5     239   289   3489  134   1349
        23468 45689 4679  1     2679  249   3457  234   345
        234   49    479   8     279   5     1     234   6
        1246  1456  467   3     267   24    457   9     8",
};

/// HoDoKu `sc01`. Simple Colors: 3 (r1c4,r3c3,r5c6,r6c1,r7c5,r9c7) / (r6c4,r7c3,r8c9,r9c6) => r13c9,r8c1<>3
pub const SIMPLE_COLOURING_1: Example = Example {
    grid: "214..6.....79.2..4...4.7.....187..32..269.....48.21..642.7.9861..9168...18624...9",
    candidates: "
        2     1     4     35    358   6     3579  5789  3578
        3568  356   7     9     1358  2     1356  158   4
        35689 3569  35    4     1358  7     12356 1258  358
        569   569   1     8     7     45    459   3     2
        357   357   2     6     9     345   1457  14578 578
        3579  4     8     35    2     1     579   579   6
        4     2     35    7     35    9     8     6     1
        357   357   9     1     6     8     24    24    35
        1     8     6     2     4     35    357   57    9",
};

/// HoDoKu `sc02`. Simple Colors: 8 (r1c6,r3c7,r4c1,r5c9,r6c5,r7c3) / (r1c9,r2c1,r2c4,r3c2,r4c4,r5c3,r6c7,r7c6,r9c2,r9c5) => r1c9,r2c14,r39c2,r4c4,r5c3,r6c7,r7c6,r9c5<>8
pub const SIMPLE_COLOURING_2: Example = Example {
    grid: "659...13...1.3.6252.3165.49.2..9631.36.7..59.91.3.4.6279.6..2535.6...9811.2...476",
    candidates: "
        6   5   9   24  247 278 1   3   78
        48  478 1   89  3   79  6   2   5
        2   78  3   1   6   5   78  4   9
        48  2   57  58  9   6   3   1   47
        3   6   48  7   12  12  5   9   48
        9   1   57  3   58  4   78  6   2
        7   9   48  6   14  18  2   5   3
        5   34  6   24  47  237 9   8   1
        1   38  2   589 58  39  4   7   6",
};

/// HoDoKu `sk01`. Skyscraper: 1 in r1c6,r3c9 (connected by r5c69) => r1c78,r3c45<>1
pub const SKYSCRAPER_1: Example = Example {
    grid: "697.....2..1972.63..3..679.912...6.737426.95.8657.9.241486932757.9.24..6..68.7..9",
    candidates: "
        6     9     7     1345  13458 158   1458  148   2
        45    58    1     9     7     2     458   6     3
        245   258   3     145   1458  6     7     9     18
        9     1     2     345   3458  58    6     38    7
        3     7     4     2     6     18    9     5     18
        8     6     5     7     13    9     13    2     4
        1     4     8     6     9     3     2     7     5
        7     35    9     15    2     4     138   138   6
        25    235   6     8     15    7     134   134   9",
};

/// HoDoKu `sk02`. Skyscraper: 4 in r2c5,r8c4 (connected by r28c1) => r1c4<>4
pub const SKYSCRAPER_2: Example = Example {
    grid: "..1.28759.879.5132952173486.2.7..34....5..27.714832695....9.817.78.5196319..87524",
    candidates: "
        346   346   1     46    2     8     7     5     9
        46    8     7     9     46    5     1     3     2
        9     5     2     1     7     3     4     8     6
        568   2     569   7     16    69    3     4     18
        368   36    369   5     146   469   2     7     18
        7     1     4     8     3     2     6     9     5
        23456 346   356   2346  9     46    8     1     7
        24    7     8     24    5     1     9     6     3
        1     9     36    36    8     7     5     2     4",
};

/// HoDoKu `u101`. Uniqueness Test 1: 8/9 in r2c23,r6c23 => r2c2<>8, r2c2<>9
pub const UNIQUE_RECTANGLE_1: Example = Example {
    grid: "5.2..89671..7..452.675..3812136578496548912737....46158219...343.6....98..5.83.26",
    candidates: "
        5    34   2    134  134  8    9    6    7
        1    389  89   7    36   69   4    5    2
        49   6    7    5    24   29   3    8    1
        2    1    3    6    5    7    8    4    9
        6    5    4    8    9    1    2    7    3
        7    89   89   23   23   4    6    1    5
        8    2    1    9    67   56   57   3    4
        3    47   6    124  1247 25   157  9    8
        49   479  5    14   8    3    17   2    6",
};

/// HoDoKu `u102`. Uniqueness Test 1: 2/3 in r2c46,r9c46 => r9c6<>2, r9c6<>3
pub const UNIQUE_RECTANGLE_2: Example = Example {
    grid: "197846532..8.5.719352917648831769425524138976..9524183.864....1..36....79.5....64",
    candidates: "
        1   9   7   8   4   6   5   3   2
        46  46  8   23  5   23  7   1   9
        3   5   2   9   1   7   6   4   8
        8   3   1   7   6   9   4   2   5
        5   2   4   1   3   8   9   7   6
        67  67  9   5   2   4   1   8   3
        27  8   6   4   79  235 23  59  1
        24  14  3   6   89  125 28  59  7
        9   17  5   23  78  123 238 6   4",
};

/// HoDoKu `w01`. W-Wing: 5/9 in r4c4,r8c9 connected by 9 in r48c8 => r4c9<>5
pub const W_WING_1: Example = Example {
    grid: "9251346878..65943243672895164..1.8..15.48..6.3.8.6..145..276348263841...784395126",
    candidates: "
        9   2   5   1   3   4   6   8   7
        8   17  17  6   5   9   4   3   2
        4   3   6   7   2   8   9   5   1
        6   4   279 59  1   237 8   79  359
        1   5   279 4   8   237 27  6   39
        3   79  8   59  6   27  257 1   4
        5   19  19  2   7   6   3   4   8
        2   6   3   8   4   1   57  79  59
        7   8   4   3   9   5   1   2   6",
};

/// HoDoKu `w02`. W-Wing: 4/1 in r1c9,r8c7 connected by 1 in r18c3 => r123c7,r89c9<>4
pub const W_WING_2: Example = Example {
    grid: "6..95..7...9.2.....58.31...164389752...175946597246..8925417683...562.....6893...",
    candidates: "
        6     134   123   9     5     48    12348 7     14
        347   1347  9     67    2     48    13458 136   145
        247   5     8     67    3     1     24    269   49
        1     6     4     3     8     9     7     5     2
        238   38    23    1     7     5     9     4     6
        5     9     7     2     4     6     13    13    8
        9     2     5     4     1     7     6     8     3
        3478  13478 13    5     6     2     14    19    1479
        47    147   6     8     9     3     1245  12    1457",
};

/// HoDoKu `xy01`. XY-Wing: 5/7/2 in r1c36,r2c1 => r2c6<>2
pub const Y_WING_1: Example = Example {
    grid: "8..36.9....9.1.863.63.89..5924673158386951724571824396432196587698537......248639",
    candidates: "
        8   145 57  3   6   25  9   147 12
        27  45  9   47  1   25  8   6   3
        127 6   3   47  8   9   24  147 5
        9   2   4   6   7   3   1   5   8
        3   8   6   9   5   1   7   2   4
        5   7   1   8   2   4   3   9   6
        4   3   2   1   9   6   5   8   7
        6   9   8   5   3   7   24  14  12
        17  15  57  2   4   8   6   3   9",
};

/// HoDoKu `xy02`. XY-Wing: 1/6/9 in r4c14,r5c2 => r4c23,r5c456<>9
pub const Y_WING_2: Example = Example {
    grid: "714.6.5388..453..7356718429....24.854.....3.2285376941978631254.....7..6........3",
    candidates: "
        7    1    4    29   6    29   5    3    8
        8    29   29   4    5    3    16   16   7
        3    5    6    7    1    8    4    2    9
        16   369  1379 19   2    4    67   8    5
        4    69   179  1589 89   59   3    67   2
        2    8    5    3    7    6    9    4    1
        9    7    8    6    3    1    2    5    4
        15   234  123  2589 489  7    18   19   6
        156  246  12   2589 489  259  178  179  3",
};

/// HoDoKu `xyz01`. XYZ-Wing: 4/5/7 in r27c2,r7c1 => r9c2<>7
pub const XYZ_WING_1: Example = Example {
    grid: "869453721...9215682158..4396215349874.761.352...2..146...1.28.39327856141..34.2.5",
    candidates: "
        8   6   9   4   5   3   7   2   1
        37  47  34  9   2   1   5   6   8
        2   1   5   8   67  67  4   3   9
        6   2   1   5   3   4   9   8   7
        4   89  7   6   1   89  3   5   2
        35  589 38  2   79  789 1   4   6
        57  457 46  1   69  2   8   79  3
        9   3   2   7   8   5   6   1   4
        1   78  68  3   4   69  2   79  5",
};

/// HoDoKu `xyz02`. XYZ-Wing: 4/7/6 in r23c4,r3c7 => r3c56<>6
pub const XYZ_WING_2: Example = Example {
    grid: "673.8.2.9514.92..3982.....5167359..28.9.1.536..58..791..614.928..1..8354..8...167",
    candidates: "
        6     7     3     45    8     145   2     14    9
        5     1     4     67    9     2     68    78    3
        9     8     2     467   367   13467 46    147   5
        1     6     7     3     5     9     48    48    2
        8     24    9     247   1     47    5     3     6
        234   234   5     8     26    46    7     9     1
        37    35    6     1     4     357   9     2     8
        27    29    1     2679  267   8     3     5     4
        234   23459 8     259   23    35    1     6     7",
};
