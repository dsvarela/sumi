# Reviews

Wrap-up reviews of finished work, newest first. Each lists what was found by
its mechanism, what was fixed and where, what was closed by decision and why,
and where deferred items went. The class register at the end names the
defect shapes this codebase repeats; hunt them by name in later reviews.

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
| Erasing a wrong digit doesn't give back the candidates its placement removed (undo does) | Daniel's decision, then milestone 5 |
| `t` merges box marks into candidates; in a cell without candidates the marks become its only ones | Daniel's decision |
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
| Two things must agree, nothing makes them agree | 3 (key bar, board shown vs input, plan vs code) | Text, sizes or rules stated in two files |
| Two meanings, one value | 2 (empty notes, complete digit) | An empty set, zero or `None` standing for more than one state |
| Input from where the player isn't looking | 3 (hidden board, paste, modifiers) | Any path from an event to a game change that skips what is on screen |
| A test that cannot fail | 6 | Tests whose fixture can't tell the order or the branch apart; mutate the line the name claims |
| No instrument | 1 (contrast) | A rule in the plan or CLAUDE.md that no test checks |
