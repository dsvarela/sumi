# Data

`bank.txt` is built into sumi: graded Hard and Expert puzzles, which sumi
plays instead of generating them. Edit it only by rebuilding it, since a test
re-grades every line and fails on any difference.

The first line names the technique order the grades were made with, and the
second names the columns. Each line after them is a puzzle and its grade, as
`sumi rate` prints them when reading standard input: the puzzle, level,
score, steps at the hardest technique's level, and hardest technique. Every
puzzle is symmetric and has one solution. Every Expert one needs two Expert
steps or more, counted along sumi's own solving path.

## Where the puzzles come from

- 500 from [grantm/sudoku-exchange-puzzle-bank](https://github.com/grantm/sudoku-exchange-puzzle-bank),
  which is dedicated to the public domain (the Unlicense). The symmetric
  puzzles in its `hard.txt` and `diabolical.txt` were graded with
  `sumi rate`. Then 250 Hard and 250 Expert ones were drawn at random.
- 500 from sumi itself: `sumi gen --level hard -n 250 --seed 1` and
  `sumi gen --level expert -n 250 --seed 2`, graded with `sumi rate`.

Lines are sorted by level, then score.
