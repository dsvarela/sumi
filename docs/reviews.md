# Reviews

Wrap-up reviews of finished work, newest first. Each lists what was found by
its mechanism, what was fixed and where, what was closed by decision and why,
and where deferred items went. The class register at the end names the
defect shapes this codebase repeats; hunt them by name in later reviews.

## 2026-10-08: milestone 5, second read (a7cab37..0168f9b)

The milestone was built and first reviewed on Sonnet 5.5, so the same
range was read again on Opus 5.5, one reader, no lanes, then checked with
a mutation sweep of 21 changes to the lines the commits added. No defect
reached a player; the findings are in the commits, the record and the
tests.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Two things must agree | A hint stayed open unless the key was on a hand-kept list of actions that leave the board alone. An editing action added to the list would let Enter apply a step to a board it was not found on, and placing overwrites a digit | The game keeps a revision bumped by every change, undo and redo included; a hint closes as soon as the game's revision differs from its own (structural, 87b1284) |
| No instrument | The panel's height assumed every rung wraps to two lines at the board's width; it held only through the core's four-lines-at-30-columns test | Hints followed through Hard and Expert bank puzzles are wrapped as ratatui draws them and must fit (d4237f7) |
| A test that cannot fail | Mutants survived in the step count's containment direction, the number of cells rung 4 names, and "sees" for one cell | A test for each, watched failing under its mutant (2ebf930) |
| Small | "Look for a claiming"; a removal phrase joined groups that could only be one; an assert repeating the filter above it | Fixed (c6bdbb7, 75fd487, 28776e7); a test pins the second rung's opening for all 20 techniques |

### Corrections to the first review

- **Three commits do more than one thing**, against the commits rules: b8ac738
  (the hint ladder, and `c` and `r`, whose marks were not drawn until
  5f6f615), e96cb5b (three wording and counting fixes), and 99a49a6 (five
  fixes, its subject naming one: the hidden-board guard, check marks' life,
  the notice, the status line's width, and `apply_hint` writing a cell as
  fill does). Their bodies also describe the old code in the past tense.
  The history is published, so this record stands in for a rewrite.
- **"Each has a test that fails under its mutant"** was written before half
  of those mutants were run. The sweep has now run them all: every one
  fails a test.
- **"CLAUDE.md's layout still says hint is to come"** stopped being true
  with 6367065.
- **Process slips during the milestone:** several tests were written after
  the code they test and first ran green (the mutation sweep stands in for
  watching them fail); `cargo fmt -p sumi-core` and `rustfmt` on `lib.rs`
  reformatted the bank session's uncommitted `bank.rs` and `grade.rs`
  (`cargo fmt --all --check` is clean now, so nothing is left to undo); and
  scratch files were deleted with `rm` rather than `trash-put`.

## 2026-10-08: milestone 5 (81a57a3..5f6f615)

Four read-only lanes: the wording of all 20 techniques against their steps, the
screen and session run in a real terminal, the core's call sites, and the
comments and record. They reported 37 findings, with overlap (the missing key
bar entries and hints over hidden candidates were each found twice). No lane
came back clean. Key-cell order, removal digits, row and column forms and
articles were right in every step the wording lane tried, over 31 examples
and 120 generated puzzles; the hint never broke a solution in about 70,000
applied hints.

**Prediction against result.** Predicted, and found: wording claims in the
less-tested techniques, stale state, and record drift. Predicted, and wrong:
key-cell order in the wings and Skyscraper (all correct). Not predicted: the
status line overwriting the title, keys changing the view while the board was
hidden, the hint count including steps another step contains, and eight
tests that could not fail.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Two things must agree | A naked subset's effect line named all the subset's digits though a digit with no other candidate in the house is not removed (68% of such steps on generated puzzles); the naked single reason said the grid alone rules out the rest when the player's candidates had narrowed it | The effect names the removed digits; the reason is true either way. A test checks every digit an effect line names against the step's removals on Hard and Expert solving walks (structural) |
| Two things must agree | The status line, sized in one place and the title in another, pushed the frame's title off with a hint, a notice, a filter and hidden candidates | The line drops its least useful parts until it fits beside the title; the mode is never dropped (structural) |
| Two things must agree | Rung 1 counted a step another step contains as a separate step (27 to 32% of hints at Medium and above) | A step whose removals another step makes is not counted |
| Two things must agree | A hint's removal on a cell with no candidates written left out the hidden notes `fill` records, so erasing a digit gave back different candidates depending on whether `a` had been pressed | The removal writes the cell as `fill` does |
| Input from where the player isn't looking | Keys closed the hint, cleared the notice and answered the filter prompt while the window was too small to draw the board | `handle` returns before touching anything unless the key quits (structural) |
| Input from where the player isn't looking | Check marks outlived a new game and came back when the digit was typed again; the notice stayed after a key that does nothing | A mark is dropped as soon as its cell no longer holds the checked digit; a new game starts a clean view but for hidden candidates |
| A test that cannot fail | Mutants survived in the hint whitelist (v, f, c, Esc), notice clearing, Esc with marks, the new game's view, the notice in the status line, colouring's second group, StruckOut's rungs and rung clamping | Each has a test that fails under its mutant |
| Record | Four comments the range made false or misplaced (the NEEDED doc, the module doc on never showing an answer, ORDER's future tense, Step's "is to"); the module and test names cited the private plan | Corrected, citing research/hint-wording.md |

### Closed by decision

- **Hints over hidden candidates.** A hint reads, and Enter edits, the
  player's candidates whether or not `v` shows them; the hint's own marks are
  drawn regardless.
- **The wording nits.** "Look for a claiming", a reason starting with a
  numeral, "rows 2 and 6 and columns 2 and 3", and the colouring wrap not
  saying that B is then true are accurate; left.
- **Hidden single's "No other cell in row 3 can hold 5"** is true of the
  player's candidates, not necessarily of the board; the same reading as the
  hint's input.
- **`known_unique` for every game.** True of every puzzle sumi builds today
  and not enforced by the types; see the deferral.
- **b8ac738 bundles `c` and `r` with the ladder.** The history is not
  published; rewriting it would cost more than it tells.
- **CLAUDE.md's layout** still said hint was to come; changed in 6367065.
- **Equivalent mutants:** the clue filter in `wrong_digits` and the clamp in
  `set_hint_start`.

### Deferred

| Item | Goes to |
| --- | --- |
| `c`, `r` and Esc are in no key bar; a mistake hint's key bar offers no action | Milestone 6, key bar and help screen |
| The full layout needs 75x44 until the dense layout exists: the hint panel sits under the board (the 2026-10-07 row said 75x39) | Milestone 6, dense layout |
| Loaders that build a `Game` (autosave, a typed-in puzzle, the daily puzzle) must prove one solution before hints may use unique rectangle; nothing enforces it | Milestone 7 |
| Stats must not count `c` and `r` | Milestone 7 |

## 2026-10-08: milestone 4 (8b42e02..f1b26d7)

Four read-only lanes: the untested surfaces run for real (the CLI, a
pseudo-terminal), the techniques and their call sites, the record against
the code, and the comments. They reported 76 findings, with overlap (`sumi
rate` grading puzzles without one solution was found by three lanes). No
lane came back clean. All 20 techniques were sound on correct candidates.
Six fixes were structural.

**Prediction against result.** Predicted, and found: `all` listing one
step once per house, the two forms of `sumi rate` disagreeing, erase going
wrong after other actions, and record drift. Predicted, and wrong: a
definition gap in a technique the proptest never reaches (only the
degenerate unique rectangle turned up). Not predicted: `grade` calling
grids with clashing clues Easy, bare `sumi` serving Beyond puzzles, three
tests that could not fail, and the bank's stated reason being false for
Expert.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Two things must agree | Each placement kept a record of the notes it took. Erasing out of order lost notes, since a peer holding a digit was skipped; two clashing digits lost a note for good; clearing notes or `t` left the record behind, so erasing brought box marks back | Each cell keeps the notes placed digits rule out as hidden notes, shown again once no placed digit rules them out; clearing clears them; a property erases several digits in any order (structural, 1115101) |
| Two meanings, one value | `grade` solved any grid, so clashing clues the techniques could fill came out Easy, and `sumi rate <puzzle>` printed a level for puzzles with no solution | `grade` returns `None` without exactly one solution; both forms of `rate` say why (structural, 4d55144) |
| Two meanings, one value | "Beyond: 2-String Kite (1 step at that level)" counted Expert steps under a Beyond heading | Beyond is described as the techniques running out (4d55144) |
| Two things must agree | `all` listed a step once per row, column or box it could be read from: hidden singles 2.1 times on average, so hint rung 1's count would be wrong | Techniques implement `search`; `all` keeps each effect once, simplest form (structural, 1190faf) |
| Two things must agree | `Position::with_candidates` kept a candidate a placed digit rules out, and a hidden single then placed a second copy of the digit | Such candidates are dropped (structural, 668c398) |
| Plan vs code | Bare `sumi` and `sumi gen` gave Beyond puzzles, about 1 in 10, which the plan never generates | `generate_solvable` skips them; the unrated generator is crate-private (structural, dc2d25e) |
| A test that cannot fail | The soundness property checked only the first technique that found a step at each position; it passed with unique rectangle accepting four boxes | Every technique at every position; it now fails on that mutant (structural, f217709) |
| A test that cannot fail | The tie-break test's pairs were already met in top-left order; the Hard generation test passed with moving clues off | Fixtures that tell the branches apart, each watched failing (1190faf, f217709) |
| Small | Unique rectangle with all four corners {a,b} emptied all four | Needs a fourth corner with more (ff493bf) |
| Small | `rate` stopped at a non-UTF-8 line, called `#` lines errors, waited silently on a terminal; level names were case-sensitive; `--help` never said what bare `sumi` does; parse errors named cells "cell 11" | Fixed (4d55144, c44bd59, f1b26d7) |
| Record | Comments gave rarity as the bank's reason for Expert, which generates in tens of milliseconds; "every step every technique finds" overstated the property; pointing and claiming did not state their inference; ORDER_VERSION's rule missed scores and the tie-break | Corrected (24b7ad2); four local commit messages reworded before publishing (tree unchanged) |

### Closed by decision

- **`t` replaces candidates crossed out by hand.** That is what Fill means,
  decided 2026-10-08; `u` brings them back. Its key bar wording is milestone
  6's.
- **Step counts disagree with research/grader_b.py on 156 bank lines.** The
  count follows the solving path, and the two graders differ in the order of
  the Easy techniques. The data README says the count is sumi's own.
- **A failed clue move falls back to a fresh random puzzle.** It costs
  time, not correctness; the generator's docs say so.
- **Hidden single picks the top-left cell, not a box first.** The plan's
  rule is fewest cells, then the top left.
- **Long sentences in a few technique comments.** Under the comment
  checker's threshold; not rewritten.

### Deferred

| Item | Goes to |
| --- | --- |
| After a wrong digit, `a` fills candidates without the digits it rules out, and erasing it doesn't add them | Question to Daniel in the plan, on the Notes list |
| Expert puzzles come from the bank though they generate in tens of milliseconds | Question to Daniel in the plan, on the Puzzles decision |
| Grading compared only with research/grader_b.py, not HoDoKu or SE as the plan's Testing list asks | Needs HoDoKu downloaded; question to Daniel on the Testing list |
| Generated Hard puzzles are near-duplicates: median 4 cells from the nearest, 111 solution grids for 250 puzzles | Milestone 7, with transform and canonical-form duplicates |
| A session can repeat a bank puzzle | Milestone 7, which remembers played puzzles with the stats |
| Hidden notes must be saved with the game | Milestone 7, autosave |
| The level isn't shown while playing | Milestone 6, status line |
| "Pointing triple" for three cells; one placement counted once across Easy techniques | Milestone 5, hint wording and rung 1 |

## 2026-10-07: milestones 1 to 3 (394b758..c29aa73)

Four read-only lanes: the untested surfaces run in a real terminal, call
sites of the core, the record against the code, and the comments. They
reported 71 findings, with overlap (the `x` label was found by three lanes).
No lane came back clean. Six fixes were structural, making a class of defect
fail a test or impossible to write.

**Prediction against result.** Predicted, and found: `is_complete` counting
clashing digits, failing contrast, record drift, terminal key quirks, and a
clean solver and generator. Predicted, and wrong: stale comments from the
`write` rewrite (none were left). Not predicted: the note model giving "no
notes" and "all ruled out" the same value, keys acting on a hidden board or
from pasted text, and five tests that could not fail.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Two meanings, one value | An empty candidate set meant both "no notes written" and "every note ruled out", so a cell that ran out of candidates fell back to all possible digits and hid the contradiction | Candidates are `Option<Digits>`; `Some` of an empty set is a contradiction, and fill leaves it showing (structural) |
| Two meanings, one value | `is_complete` counted copies of a digit without checking clashes, so nine 5s with one clash dimmed as done; on a solved board every digit dimmed | Count only copies that don't clash; no dimming once solved |
| Two things must agree | The key bar said `x` "clear all" while `x` clears one layer; key text was typed separately from the bindings | `input::key_label` gives the key text, and a test checks every label against `input::action` (structural) |
| Two things must agree | The app acted on keys while `ui::draw` showed only the too-small message, so `a` filled every cell unseen | One `ui::fits` decides both what is drawn and whether keys act (structural) |
| Input from where the player isn't looking | Pasted text arrived as keys, so a paste containing `q` quit and one containing digits placed them | Bracketed paste is on; a paste is one event the game ignores |
| Input from where the player isn't looking | Ctrl+C after `f` only cancelled the filter; Alt+h moved like h; terminals sending ^H for Backspace did nothing | Ctrl+C is checked first; Alt keys do nothing; Ctrl+H clears |
| A test that cannot fail | The highlight-ranking test never put two highlights on one cell; pairs filter, redo's cursor, `clear` on box marks and cleanup sparing non-peers were untested; five one-line mutations survived | Rewritten tests, each watched failing under its mutation; a proptest property that undoing every step restores the start and redoing restores the end (structural) |
| A test that cannot fail | The event loop had no tests at all | Key handling moved into `Session::handle`, tested without a terminal (structural) |
| No instrument | No contrast check: `dimmed` text was 3.39:1, notes on the cursor 2.59:1, the cursor 1.95:1 against black | New palette; a test fails any text pair under 4.5:1 or graphic under 3:1 (structural). The cursor also gets corner marks, since no background can pass 3:1 against black and still carry 4.5:1 text |
| Process edges | `sumi gen \| head` panicked on the closed pipe; `sumi` without a terminal panicked | Closed pipe ends output with exit 0; no terminal prints an error and exits 1 |
| Wording | The status line said "notes hidden" against the plan's rule | "candidates and box marks hidden" |
| Small | Hiding notes reset on every new game; the too-small message was cut off below 31 columns | `v` carries over; the message wraps |

Comments corrected or added: undo's and redo's cursor, `note_clashes`,
`working_candidates`, the won-state rule, Press-only events, row and box
numbering, wrapping at the edge, the solver's scope, the deadly-rectangle
test, and the shared-screen helpers. None described the pre-rewrite API.

### Closed by decision

- **A hand-typed candidate that clashes still counts for the filter.** The
  filter shows what the player wrote; the clash is already drawn in red.
- **`sumi rate` exits 0 for no solution and for several.** Those are answers,
  not errors; only input that isn't a grid fails.
- **Caps Lock turns hjkl into box jumps.** Terminals don't report Caps Lock.
- **Commit bodies of ecee317 and c29aa73 leave out some features.** The
  history is published; this record lists them instead (box jumps, the
  too-small screen, Ctrl+C, the pairs filter, clashing notes in red, dimmed
  complete digits).

### Deferred

| Item | Goes to |
| --- | --- |
| Erasing a wrong digit doesn't give back the candidates its placement removed (undo does) | Decided 2026-10-08: erasing or overwriting gives back exactly what the placement removed. Start of milestone 4 |
| `t` merges box marks into candidates; in a cell without candidates the marks become its only ones | Decided 2026-10-08: `t` fills full candidates and clears box marks by default; a setting chooses merging instead. Start of milestone 4; the setting's page in milestone 6 |
| SIGTERM or SIGHUP leaves the terminal on the alternate screen, and loses the game | Milestone 7, with autosave |
| `q` quits without saving | Milestone 7 |
| No phone layout: the game needs 75×39 | Milestone 6, dense layout |
| Row and column numbers on the board edges | Milestone 5, before hints print r2c3 |
| Clues and digits differ only by hue; clashes and highlights are colour only | Milestone 6, accessibility check |
| No help screen; `v`, `X`, `HJKL`, Ctrl+R appear in no key bar | Milestone 6, key bar |
| Scroll wheel sends arrow keys in terminals with alternate scroll | Milestone 6, mouse |
| `sumi rate` reads one puzzle, not a list from stdin | Milestone 4, bank tooling |
| `--seed` repeats puzzles only with the same rand version | Later, the daily puzzle: needs its own shuffle |

## Class register

| Class | Seen | Hunt for |
| --- | --- | --- |
| Two things must agree, nothing makes them agree | 12 (a list of keys vs what changes the board, status line vs title, effect line vs removals, count vs containment, fill vs hint removal; before: key bar, board shown vs input, plan vs code, placement records vs notes, one step per house, stale candidates, comments vs measurements) | Text, sizes or rules stated in two files; a record of a change that later changes don't update |
| Two meanings, one value | 4 (empty notes, complete digit, a grade for an invalid grid, Beyond's step count) | An empty set, zero or `None` standing for more than one state |
| Input from where the player isn't looking | 5 (hidden board, paste, modifiers, view changing under a hidden board, marks across games) | Any path from an event to a game change that skips what is on screen |
| A test that cannot fail | 20 | Tests whose fixture can't tell the order or the branch apart; a loop that stops checking early; mutate the line the name claims |
| No instrument | 3 (contrast, outside grader, panel fit) | A rule in the plan or CLAUDE.md that no test checks |
