# Research

Evidence behind the plan: throwaway experiments and reports from planning on
6 and 7 October 2026. They justify decisions and never instruct; none of this
is reference code for sumi.

| File | What it is |
|---|---|
| `report.md` | The full research report: generation, technique order, rating, sources |
| `solving-aids.md` | Note styles and player aids in other apps (Snyder marks and its family), with a v1/later/skip call for each |
| `hint-wording.md` | Technique names, cell notation, a glossary, and draft definitions and result templates for the hint ladder |
| `learn-mode.md` | How Sudoku and chess teaching products structure lessons and drills, and a proposed Learn mode design |
| `grader.py` | Python grader with sumi's v1 technique list, easiest step first. `uv run grader.py puzzles.txt out.txt` |
| `grader_b.py` | The same grader with the plan's levels and almost its order: naked single comes before hidden single, and pointing and claiming are one "Locked Candidates" |
| `hodoku-compare.py` | Compares sumi's levels with HoDoKu's on 2,000 puzzles. Result on 8 October 2026: of 1,776 puzzles HoDoKu solved with sumi's techniques only, 1,766 got the same level; the other 10 are Medium in sumi and Hard in HoDoKu, which tries triples before pairs. The other 224 are Expert in sumi, and HoDoKu used a technique sumi lacks (remote pair 84, empty rectangle 81, BUG+1 38, turbot fish 35) |
| `mutate.py` | Clue-moving search for Hard puzzles. Reads seeds from `qq20k.graded` (not kept), one puzzle, level and technique per line, and keeps lines whose level matches its first argument. `data/hard-seeds.txt` holds puzzle and technique only, so reusing it means changing that filter, not just the file name |
| `theme-palettes.py` | Builds the built-in themes from a few colours of each source palette: lifts text to 8:1, then moves each highlight toward its own hue until a target contrast, and checks that highlights and coloured text stay apart in OKLab, and under simulated colour blindness for the colour-blind theme. `uv run theme-palettes.py ../crates/sumi-sudoku/themes` |
| `data/stuck.txt` | 3,240 puzzles (of 20,000 from qqwing) that v1's techniques can't finish. Material for v2 |
| `data/hard-seeds.txt` | The 99 Hard puzzles from the same 20,000, with their hardest technique |

Puzzles are 81 characters, row by row, `.` for an empty cell. They were
generated with qqwing as an offline tool; qqwing's code is not part of sumi.
