# Data

`bank.txt` is built into sumi: 500 Hard and 500 Expert puzzles, which sumi
plays instead of generating them. Edit it only by rebuilding it, since a test
re-grades every line and fails on any difference.

The first line names the technique order the grades were made with, and the
second names the columns. Each line after them is a puzzle and its grade, as
`sumi rate` prints them when reading standard input: the puzzle, level,
score, steps at the hardest technique's level, and hardest technique. Lines
are sorted by level, then score.

## The rule the puzzles are chosen by

Each level's puzzles spread over its techniques rather than leaning on the
commonest one. The tests in `src/bank.rs` check every part of this.

- **Every puzzle** is symmetric and has one solution. No two share a
  solution grid. Every Expert one needs two Expert steps or more, counted
  along sumi's own solving path.
- **Hardest technique.** Each of a level's techniques is the hardest step
  in an even share of its puzzles, give or take one. A technique short of
  puzzles leaves the rest of its share to the others.
- **Rows, columns and boxes.** Pairs, triples, quads and W-Wing links lie
  along boxes, rows and columns. For each, no kind of house holds more than
  half of its steps, or less than a fifth. X-Wings, Swordfish and
  Skyscrapers lie along rows or columns, and neither holds more than 60%.
  A technique with fewer than 50 steps in a level is not checked.
- **Singles.** Every puzzle's solving path reaches a position where no box
  has a hidden single but a row or column does. Scanning boxes alone never
  finishes it. The hidden singles on offer along the paths need no
  choosing: about a third each are in boxes, rows and columns.
- **What comes with the hardest step.** Take the Expert puzzles with one
  hardest technique. No other Expert technique is in more than 60% of them,
  and neither is taking no other. Skyscraper comes first in the order, so
  its puzzles take no other.

The builder picks one puzzle at a time, within the shares. Each pick is
the candidate whose house kinds and companion techniques the bank holds
least of so far.

## What the bank holds

Candidates are the puzzles the rule allows, counted by distinct solution
grid.

| Hard: hardest technique | Puzzles | Candidates |
| --- | --- | --- |
| Naked triple | 94 | 18,329 |
| Hidden triple | 94 | 3,622 |
| X-Wing | 94 | 31,652 |
| Naked quad | 94 | 388 |
| Swordfish | 93 | 4,681 |
| Hidden quad | 31 | 31 |

| Expert: hardest technique | Puzzles | Candidates |
| --- | --- | --- |
| Skyscraper | 72 | 1,594 |
| 2-String Kite | 72 | 6,270 |
| W-Wing | 72 | 14,715 |
| Y-Wing | 71 | 7,370 |
| XYZ-Wing | 71 | 6,243 |
| Unique rectangle | 71 | 4,279 |
| Simple colouring | 71 | 1,710 |

Hidden quad is the one technique short of its share. It is the hardest step
of 6 of the 270,411 symmetric puzzles in grantm's two files. It is also the
hardest of 64 of the 100,824 Hard puzzles generated below. Those 70 have 31
solution grids between them, and the bank holds all 31.

## Where the puzzles come from

- [grantm/sudoku-exchange-puzzle-bank](https://github.com/grantm/sudoku-exchange-puzzle-bank),
  which is dedicated to the public domain (the Unlicense): its `hard.txt`
  and `diabolical.txt`. 83 Hard and all 500 Expert puzzles are from there.
- sumi itself: 417 Hard puzzles from `sumi gen --level hard`.

## How to rebuild it

From the repository root, in fish, with the files kept in `/tmp/bank`:

```fish
mkdir -p /tmp/bank
for f in hard diabolical
    curl -sSfL -o /tmp/bank/$f.txt https://raw.githubusercontent.com/grantm/sudoku-exchange-puzzle-bank/master/$f.txt
end
cargo build --release
for seed in (seq 101 112)
    ./target/release/sumi gen --level hard -n 8402 --seed $seed > /tmp/bank/gen-$seed.txt &
end
wait
for seed in (seq 101 112)
    cat /tmp/bank/gen-$seed.txt
end > /tmp/bank/gen-hard.txt
cargo run --release -p sumi-core --example build_bank -- /tmp/bank/hard.txt /tmp/bank/diabolical.txt /tmp/bank/gen-hard.txt > crates/sumi-core/data/bank.txt
```

The twelve `sumi gen` runs took about two hours on a 12-thread laptop, and
the builder about three minutes. The builder prints the counts above on
standard error. It breaks ties with a fixed seed, so the same files in the
same order give the same bank. A puzzle in two files is credited to the
first. More generated puzzles bring more hidden quads.
