# Research

Throwaway experiments from planning on 6 October 2026. They back the plan's
Algorithms section; none of this is reference code for sumi.

| File | What it is |
|---|---|
| `report.md` | The full research report: generation, technique order, rating, sources |
| `grader.py` | Python grader with sumi's v1 technique list, easiest step first. `uv run grader.py puzzles.txt out.txt` |
| `grader_b.py` | The same grader with the technique order the plan adopted |
| `mutate.py` | Clue-moving search for Hard puzzles. Reads seeds from a graded file named `qq20k.graded` (one puzzle, level, technique per line); point it at `data/hard-seeds.txt` before reuse |
| `data/stuck.txt` | 3,240 puzzles (of 20,000 from qqwing) that v1's techniques can't finish. Material for v2 |
| `data/hard-seeds.txt` | The 99 Hard puzzles from the same 20,000, with their hardest technique |

Puzzles are 81 characters, row by row, `.` for an empty cell. They were
generated with qqwing as an offline tool; qqwing's code is not part of sumi.
