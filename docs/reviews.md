# Reviews

Wrap-up reviews of finished work, newest first. Each lists what was found by
its mechanism, what was fixed and where, what was closed by decision and why,
and where deferred items went. The class register at the end names the
defect shapes this codebase repeats; hunt them by name in later reviews.

## 2026-10-09: milestone 7 (e089339..1d4e620)

Four read-only lanes: the live terminal, driven in tmux with scratch data
and config directories (it found that tmux can send raw SGR mouse
sequences, so taps were tried too); what is written to and read from
disk, with fixtures and mutants in a copy of the workspace; the logic in
app, game, learn, entry, menu and ui and the call sites of the changed
core modules; and the record and the comments. They reported about 90
findings, about 70 distinct once overlaps were merged (a typed clue over
a paper digit, the false "nothing shows it yet" and the Beyond level
after a win were each found by two or three lanes). No lane came back
clean. Drills started in under 0.1 s for every technique tried; lessons
and drills left the saved game and the stats untouched, under signals too.

**Prediction against result.** Predicted 35 to 60 findings; about 70
came. Predicted, and found: the signal thread's edges (the cursor left
hidden, a handler set up after the terminal changed), clicks reaching a
lesson's board, the plan's stale sizes and module lists, tests whose
fixtures can't fail. Predicted, and wrong: torn writes between the signal
thread and the loop (the lock held), the saved-game key missing fields.
Not predicted, and the most serious: a save or stats file that couldn't
be read as text was overwritten by the next write, an unreadable
played.json always was; two sumis undid each other's stats, puzzles
played and moves; a transform could leave an Expert puzzle needing one
Expert step; and a lesson told players a right digit "nothing on the
board shows" when another technique did.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Process edges | A file that couldn't be read as text (a stray byte, no permission) was left in place and overwritten by the next write; an unreadable played.json was never moved aside; a second unreadable file replaced the first one moved aside | Every load goes through one `load`: any failure moves the file aside under a new name, and if it can't be moved nothing is written that run (structural, c95368c) |
| Process edges | Two sumis: each wrote back its whole stats and played set, and quitting rewrote a game with nothing changed over the other's moves | Stats and played puzzles are added to what the file holds; a game unchanged with its clock stopped isn't written, on quitting or on a signal (c95368c) |
| Process edges | A signal during a stats or played write could stop sumi with a temporary file left; the cursor stayed hidden after a signal; a failure setting up the handler left raw mode on, and without a data directory no handler ran | One process-wide write lock the signal thread holds to the end (structural); the cursor is shown; the handler is set up before the terminal changes and always runs (c95368c) |
| Process edges | A save's mode was copied even when it left the file unreadable; a dangling symlink was replaced; a won save loaded as a game to continue; a newer save named a field rather than its version; counts in a hand-edited file overflowed; a stop between writing the stats and removing the save could count a win twice | Owner read and write kept, links followed to a file not made yet, the directory synced, a won save refused, the version read first, counts saturate, the save removed before the win is counted (c95368c) |
| Two things must agree | Enter after a win panicked for a saved Beyond game and started the last level asked for after a typed-in one | Enter starts the game's level when one can be made (ac205e2) |
| Input from where the player isn't looking | A won game's pause menu offered bookmarks, which un-won it; f then Esc in a lesson left the lesson; r answered a placing lesson; a tap on a note's spot during See it did nothing | No bookmarks after a win; the prompt has its keys; r, b and B don't reach a lesson's board; a tap there is a tap on the cell (ac205e2, 1c6fff5) |
| Two things must agree | The cursor's colour hid the last move's shading on resume; going back from a screen reset the menu to its first item | The cursor stays where it was left; back lands on the item opened (ac205e2) |
| No instrument | A lesson said "nothing on the board shows it yet" or "nothing rules out" for a step another technique makes | Answers are judged against every technique: another's step is right, named, and not a miss (structural, 1c6fff5) |
| Two things must agree | Lesson boards never animated (the loop ticked only the game's session); a drill board that was its puzzle's last cell showed "Solved" instead of the lesson; feedback stayed after the board changed; key names were typed by hand in lesson words; "house" was on screen | One `board_session` for the loop (structural); the lesson outranks the win in the panel; feedback clears on a change; `{enter}` and `key_label`; a test keeps the banned words out of every lesson (structural, 1c6fff5) |
| Two meanings, one value | `NotAGrid(n)` meant a wrong count and a bad character ("Pasted 81 cells, not 81.") | Two verdicts; borders other programs draw are passed over (structural, a04f9db) |
| Two things must agree | A clue typed over a paper digit hid it; pasted digits that fill the board started a game won at 0:00 and counted in the stats | A clue takes the digit's place; a full board is refused; typing moves on, . clears, clashing clues are marked (a04f9db) |
| Two things must agree | At a dead end B was offered when it would do nothing, check marks lost their Esc, 1-9 said place in a note mode; only the newest bookmark was checked for duplicates | `can_go_back`; Esc kept; toggle; any bookmark (2a47e49) |
| No instrument | Transforms kept the level but could change the grader's path: 118 of 1,500 transformed Expert bank puzzles needed one Expert step; the comments said "rating" | The rule is `Grade::offers_as`, used by the generator, the bank builder and the app (structural); the app re-grades what it shows (7ea421e) |
| Small | Stats needed 43 columns, so the phone's menu opened the too-small screen; a long name or count could break the table | 37 columns: Won, Best, Average, Hints; times from 10 h in hours; columns held (d812675) |
| Two meanings, one value | `sumi rate` called a Beyond puzzle no technique starts on "no step needed", as for a solved board | Says no technique finds a first step (0e63aa5) |
| A test that cannot fail | The store's write test passed with a plain `fs::write`; permission copying, step and bookmark cursors, short arrays, the recorded time and the menu-size test (which left out Learn and a technique's menu) were unchecked; my own tests navigated menus by counting keys | Each checks what its name says, watched failing under the lane's mutation; a `choose` helper picks items by label (c95368c, ac205e2, afb7dc4) |
| Record | The data README's rebuild no longer reproduced the bank (ca9d849 changed the generator) and said sumi never generates Hard; comments on the bank, menus, game fields, the saved game's checks, `proven`, the app's level; a stacked test doc; an empty impl; CLAUDE.md's module list | Corrected; the README names the commit its files came from (afb7dc4) |
| Record | The plan's Learn, Bookmarks and Controls text, the architecture tables (entry, stats, learn, store, cli, transform), where files live, main libraries, the Hint core signature, the Algorithms note on typed-in puzzles, sizes in milestone 6's notes, the Principles' panel, the Y-Wing lesson | Corrected in the plan the same day |

### Closed by decision

- **A hint applied with Enter answers a lesson or drill, and the drill's
  last line doesn't tell found from shown.** Hints are help, not a
  penalty; the plan leaves out scores.
- **Strikes from two instances of the technique count as one answer.**
  Each struck candidate is justified, which is the research's rule 2.
- **The key list in a lesson is the game's whole list.** r, b and B no
  longer act there; the list says what keys do in a game.
- **q on the entry screen quits and drops the typed clues.** q quits from
  every screen.
- **"Replace the saved game?" doesn't name the game, and Esc goes back to
  where the request came from.** Continue in the main menu names it; going
  back where asked is the plan's rule.
- **Unreadable files are named only on stderr after quitting.** Each is
  moved aside, so nothing is lost, and the warning says where.
- **A win in one sumi removes the save another sumi is playing.** That
  sumi writes it again on its next key.
- **Stats key techniques by their names.** The names are the techniques'
  identity in bank.txt too.
- **played.json keeps clues the bank no longer holds.** They are never
  dealt, so they cost a few bytes.
- **The last move on resume is the last step kept, not the last undone.**
  Undo is a choice to drop that move.
- **"Continue · 1 left" counts a wrong digit as filled.** It counts cells
  to fill.
- **Pasting into a game does nothing.** Bracketed paste is there so a
  paste can't become keys.
- **The stage after Find it is "Close" in the plan and "Done" on screen;
  lessons say "strike" where hints say "remove".** The plan chose both.
- **App's `Saved` key restates what `to_json` writes that a key can
  change.** The round-trip test holds the save; the key only decides when
  to write.

### Deferred

| Item | Goes to |
| --- | --- |
| The settings page needs 54 columns, so on a 38-column phone its new switches (Digit pad, Stats) can't be reached | Daniel's Termux check, with the phone work it asks for |
| The save is flushed to the disk on every key that changes the game, which may be slow on the phone's storage | Daniel's Termux check, measured there |
| The settings group and the key list's heading say "Notes", a word the hint wording keeps off the screen | Question to Daniel, in the plan's milestone 7 notes |
| On macOS the saved game, played puzzles and stats share ~/Library/Application Support/sumi with config | Milestone 8, with the macOS build (the row from milestone 6, now with three files) |

The milestone 6 and earlier rows naming milestone 7 are closed: menus
sized by their items (51a001d), the pad switch (202262a), loaders proving
one solution (f7a6008), stats not counting c and r (c20dcf7), near-
duplicate Hard puzzles (ca9d849, by exact grid), repeated bank puzzles
(82f6ef8), hidden notes saved (77b13eb), signals and q saving (3d1dfbe,
77b13eb).

## 2026-10-08: milestone 6 (9ae2633..2cc6503)

Four read-only lanes on the milestone's own commits (the milestone-5
refactors in the same range were left out): the live terminal, driven in
tmux; what is read from and written to disk; the logic in app and ui and
its call sites; and the record and the comments. They reported 98
findings, about 70 distinct once overlaps were merged (the paused hint and
the settings page's width were each found by three lanes). No lane came
back clean. The theme generator reproduced the shipped themes byte for
byte, every theme test failed under a mutation of its theme, and the
terminal was left clean after q and Ctrl+C.

**Prediction against result.** Predicted 25 to 40 findings; about 70
came. Predicted, and found: values shown in two places disagreeing (the
settings page's sizes and its hint_start, the plan's sizes), the hint's
close button acting behind the pause screen, mouse reporting left on
after a panic, CLAUDE.md's module list, comments a later phase made
false, tests that could not fail. Predicted, and wrong: the key bar's
width formula disagreeing with its drawing, stale click areas after a
resize, snapshots accepted without reading. Not predicted, and the most
serious: one wrongly typed value in config.toml threw away every setting
while later saves looked successful; Esc doing nothing after a win; the
status line dropping hidden notes first; the faint half of a pulse
matching the shading already there.

### Fixed

| Class | Finding and mechanism | Fix |
| --- | --- | --- |
| Two meanings, one value | Defaults stood for both "nothing chosen" and "the file didn't load": one bad value gave the defaults, and a setting saved afterwards was lost again at the next load | config.toml is read key by key: a bad value keeps its default and an unknown key is skipped, each named on stderr; hint_start is held to 1 to 5 (structural, acae2fa) |
| Process edges | Saving truncated then wrote the file, so a crash could leave half a file; a symlinked config.toml was not followed; an inline highlights table could not be saved | The file is written beside the original and renamed over the file a symlink points to; inline tables save (acae2fa) |
| No instrument | A misspelt `[colours]` in a theme file, or an unknown config key, was silently ignored | Theme files deny unknown fields; unknown config keys are named (7736153, acae2fa) |
| Small | A theme could only extend one whose file sorted before it; `#+1+2+3` was a colour; an unreadable themes folder and a missing theme were silent | Themes load in rounds until none is left; six hex digits; both warn (7736153, acae2fa) |
| Input from where the player isn't looking | Pausing blanked a hint's words but left its × drawn and clickable and its rung in the status line | `View::shown_hint` is the one test of whether a hint shows, used by drawing, the status line and clicks (structural, a5c4572) |
| Input from where the player isn't looking | A click on nothing left the prompt after f open, though any key ends it | Every click reaches the session; one on nothing ends the prompt and the notice (a5c4572) |
| Two things must agree | After a win Esc did nothing, though the key list says it opens the menu; Resume would then have restarted a won game's clock | Esc after a win opens the pause menu, and Resume starts the clock only if the game should be running (a5c4572) |
| Two things must agree | The clock ran while the window was too small to show the board, and the loop decided whether to redraw each second from a restatement of the rule rather than the clock | The clock stops while the board can't be seen; the loop asks the game's clock (a5c4572) |
| Two things must agree | The status line dropped hidden notes and the filter first, and "candidates and box marks hidden" never fitted the dense line | What the player turned on outranks the level and time, and hidden notes shorten to "notes hidden" before anything is dropped (eb74007) |
| Two things must agree | The settings page's selected Theme row needed 46 columns where 44 fitted; its test allowed 46 | The page is 54 wide, longer names are shortened with …, and the test measures the width the layout gives (eb74007) |
| Two things must agree | The faint half of a house pulse used the peer shading, which the cursor's house already had, so the pulse looked half as long | The faint light uses the hint area's colour (eb74007) |
| Small | A save error was cut to its path; stderr repeated it per press; a hidden key bar left a gap in the frame; a menu row took clicks only on its label; the settings page didn't say when the terminal overrides the theme | The page shows the reason, stderr names it once, the gap is gone, the whole row takes the click, and the page says why the theme isn't drawn (eb74007) |
| Process edges | A panic left mouse reporting and bracketed paste on; mouse reporting asked for every movement, waking the loop; with stdout redirected sumi drew into the file | A panic hook turns both off first; only presses are reported; sumi refuses a non-terminal stdout (786989d) |
| No instrument | The plan and 4ca5ce3 said tests keep the cursor apart from the same-digit highlight; nothing did. The contrast floors were written twice, in the tests and the generator, and a test comment claimed highlights stay under 3:1 | The floors live once in `colour`, which both use; an APART pair holds the cursor and same-digit apart; highlights under 3:1 is asserted (structural, eed95bc) |
| A test that cannot fail | `after_a_win_only_enter_and_quit_act…` passed with h and v acting after a win; `q_quits_from_any_menu` tried one menu; the main menu's Medium default and the paused key bar were untested | Each checks what its name says, watched failing under the lane's mutation (eed95bc) |
| Two things must agree | The filter prompt's key text was typed by hand beside `filter_key` | `input::filter_label`, tested against `filter_key` (structural, eed95bc) |
| Two meanings, one value | `App`'s level was an Option whose None meant "open the menu" in one place and "any level" in another, leaving the any-level path unreached | The level is always a level, Medium until one is picked (structural, eed95bc) |
| Small | With no theme set, the settings page cycled from whichever theme was listed first, matching the default by coincidence | It starts from the default (197f4dc) |
| Record | Four doc comments stacked on the wrong item; comments a later phase made false (the reversed cues, the colour roles' uses, the background order, Esc after a win, settle, the game module, Action::Close, frame_block, NEEDED); `sumi --help` said bare sumi plays; CLAUDE.md's module list | Corrected (61dce50) |
| Record | The plan's architecture tables, main libraries, colour roles ("panel"), layouts table ("to the right"), side panel, key bar sketch, sizes in older notes (75×39, 75×44, 52×28), "tested in Termux", and the risk "check early" | Corrected in the plan the same day |

### Closed by decision

- **toml_edit turns CRLF into LF and drops a BOM when it saves.** The
  content is kept; a config written on Windows is rare in a terminal game.
- **The 16-colour theme has no peer shading, and the same colour for the
  same-digit highlight and a hint's area.** Sixteen named colours on a
  black background leave no third subtle shade; the plan calls that theme
  best effort, and the settings page now says when it is in use.
- **Undo then redo of the move that completed a house pulses it again.**
  Completing it again is completing it.
- **The clock runs while the key list is open.** It is a list over the
  game, like the hint; pausing is one key away.
- **A 74×46 window gets the too-small screen.** The dense layout needs 47
  rows and the full one 75 columns; the message names the smaller.
- **With mouse = false, a terminal may send the scroll wheel as arrow
  keys again.** That is what turning the mouse off means.
- **The key list's prose names other keys ("arrows too", "Space",
  "b pairs, 0 off"), and the menus use `up_down_label` and
  `left_right_label` beside `key_label`.** The list is prose, not a key
  bar, and both labels are tested against the bindings.
- **382b1f4 changed two theme colours by one step, and bd0c51f's body
  credits it with NO_COLOR.** Unpublished, but interleaved with another
  session's commits; rewriting would cost more than it tells, so this
  record says it instead.

### Deferred

| Item | Goes to |
| --- | --- |
| On macOS the directories crate puts config in ~/Library/Application Support/sumi, and the future saves in the same folder | Milestone 8, with the macOS build |
| The dense hint's × is a one-character tap target at the window's edge | Daniel's Termux check of the phase 10 build |
| `MENU_NEEDED` assumes at most four items; Bookmarks, Learn and Stats will add more | Milestone 7, with those items |
| The plan's principle that every helper has a switch: the pad and the hint panel have none | Question to Daniel, in the plan's milestone 6 notes |

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
| Two things must agree, nothing makes them agree | 30 (a level after a win, the last move under the cursor, the menu item after going back, a lesson's frozen animations and stale feedback, hand-typed keys in lessons, a clue over a paper digit, B offered when it can't act, the data README's rebuild; before: settings page width vs its rows, hint_start shown vs used, status drop order vs the plan, a won game's keys vs the key list, a clock rule stated twice, the faint pulse vs the shading under it, hand-typed filter labels, the plan's sizes; before: a list of keys vs what changes the board, status line vs title, effect line vs removals, count vs containment, fill vs hint removal; before: key bar, board shown vs input, plan vs code, placement records vs notes, one step per house, stale candidates, comments vs measurements) | Text, sizes or rules stated in two files; a record of a change that later changes don't update |
| Two meanings, one value | 8 (a paste verdict for a count and a character, no step for solved and stuck; before: config defaults as "unset" and "failed to load", `Option<Level>` as "menu" and "any level"; before: empty notes, complete digit, a grade for an invalid grid, Beyond's step count) | An empty set, zero or `None` standing for more than one state |
| Input from where the player isn't looking | 11 (bookmarks un-winning a game, a lesson taking the filter's keys, r in a lesson, a tap in See it; before: a paused hint's ×, a click on nothing leaving a prompt open; before: hidden board, paste, modifiers, view changing under a hidden board, marks across games) | Any path from an event to a game change that skips what is on screen |
| A test that cannot fail | 37 | Tests whose fixture can't tell the order or the branch apart; a loop that stops checking early; mutate the line the name claims |
| No instrument | 9 (a lesson's claim nothing shows a step, the transform's rating, banned words in lessons; before: unknown config and theme keys, a separation claimed and unchecked, floors written twice; before: contrast, outside grader, panel fit) | A rule in the plan or CLAUDE.md that no test checks |
| Process edges | 9 (a file unreadable as text overwritten, two sumis undoing each other, a stop mid-write, the cursor after a signal, a handler set up too late, a won save continued; before: a panic leaving terminal modes on, a half-written config file, drawing into a redirected stdout; before: closed pipes and no terminal) | What the terminal, the disk or the pipe is left holding when sumi stops early or isn't where it expects |
