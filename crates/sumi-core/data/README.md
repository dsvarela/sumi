# Data

`bank.txt` is built into sumi: graded Hard and Expert puzzles, which are too
rare to generate while a player waits. Edit it only by rebuilding it, since a
test re-grades every line and fails on any difference.

Each line is a puzzle and its grade, as `sumi rate` prints them: the puzzle,
level, score, steps at that level, and hardest technique. The first line
names the technique order the grades were made with. Every puzzle is
symmetric, has one solution, and if Expert needs two Expert steps or more.

## Where the puzzles come from

- 500 from [grantm/sudoku-exchange-puzzle-bank](https://github.com/grantm/sudoku-exchange-puzzle-bank),
  which is dedicated to the public domain (the Unlicense). The symmetric
  puzzles in its `hard.txt` and `diabolical.txt` were graded with
  `sumi rate`. Then 250 Hard and 250 Expert ones were drawn at random.
- 500 from sumi itself: `sumi gen --level hard -n 250 --seed 1` and
  `sumi gen --level expert -n 250 --seed 2`, graded with `sumi rate`.

Lines are sorted by level, then score.
