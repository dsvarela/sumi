# Solving aids, notations and player tools: what exists beyond sumi's plan

Researched 2026-10-07. Every app claim links to the page or source file it came from. Where I read an app's own shipped code (SudokuPad, f-puzzles, sudoku.coach, SudokuExchange), I read it for feature names and help text only; nothing here is code to copy.

## Single case or a family?

Snyder notation belongs to a family. Sudoku notes make one of four kinds of claim, and every convention found is one of those claims drawn in a particular way:

1. **Cell-centric:** "these digits can still go in this cell." Full candidates, centre marks, pair and triple marks.
2. **Unit-centric:** "digit d goes in one of these cells of this box or line." Snyder, corner marks, Good Sudoku's accent notes, box candidates on sudoku.coach.
3. **Negative:** "d is ruled out here." Cross-out notes, complementary pencilmarks.
4. **Relational:** "these candidates are linked." Colour pairs, colour dots, letters, drawn strong and weak links.

Snyder is the narrowest unit-centric form: box only, exactly two spots. Apps that support it do so as a second note layer next to cell candidates. sudoku.coach calls this "Cell&Box Notation" and says box candidates "are sometimes referred to as Snyder notation". SudokuExchange has "outer" marks for Snyder and "inner" marks for pairs and triples. SudokuPad has corner and centre modes. sudoku.coach also teaches it as the first notation lesson, before cell candidates and full notation. So the gap in sumi's plan is a whole layer, not one feature. sumi has the cell-centric layer (the 3x3 mini-grid) and no unit-centric, negative or relational layer.

## Summary table

Key: **T** = teaching, **C** = calm play. Full = 7x3-character cells; Dense = 3x3-character cells. "Variant of plan" means it extends something sumi already plans.

| # | Item | What it is | Used by | Serves | Conflict with principles? | Full 7x3 | Dense 3x3 | Effort | Rec. |
|---|------|-----------|---------|--------|---------------------------|----------|-----------|--------|------|
| 1 | **Box marks (Snyder / corner layer)** | Second note layer that says "d goes in one of these cells of the box". It is typed by hand and drawn in a different style from cell candidates. | sudoku.coach Box Candidates [1]; SudokuExchange outer marks [3]; SudokuPad corner mode [5]; Good Sudoku accent notes [8] | T + C | No. It is the player's own reasoning. | Easy: same positional digit, bold/colour vs dim | Feasible: same 9 positions, style only (needs colour or bold) | Medium | **v1** (pull forward from LATER "corner notes") |
| 2 | Extended Snyder (rows/cols, 1 or 2 spots) | The same idea applied to rows and columns, or to digits with a single spot. | Guides only [10][11] | T | No | Same as #1 | Same as #1 | None beyond #1 (a Learn topic) | later (lesson text) |
| 3 | **Convert box marks to cell candidates** | One key turns the selected cells' box marks into candidates, for example once a pair turns out to be a naked pair. | SudokuExchange "." key [3]; sudoku.coach convert button [1] | C | No | Easy | Easy | Small | **v1** with #1 |
| 4 | Centre marks / pair-triple marks | Cell-centric notes, often limited to 2 or 3 digits. | SudokuPad centre mode [5]; SudokuExchange inner marks [3] | C | No | Already covered by the positional mini-grid | Same | n/a | Variant of plan: skip as separate layer |
| 5 | Cross-out / eliminated marks | Keep a ruled-out candidate visible as struck or dimmed instead of deleting it. | Good Sudoku cross-out notes [8]; Sudopedia "complementary pencilmarks" [12] | T | No. It shows the player's own work. | Dim digit or `·` at position; strike-through (SGR 9) is not reliable everywhere | Dim only | Medium | later |
| 6 | **Check notes against placed digits** | Flag a note whose digit is already placed in the same row, column or box. | SudokuPad "Check Pencilmarks" [6]; sudoku.coach "Highlight Impossible Candidates" [1] | C | No. It is rule-level, like conflict marking, and does not use the solution. | Easy (colour) | Easy | Small | **v1** (extend conflict marking, own toggle) |
| 7 | Digit-completion dimming on grid / pad | Fade or mark a digit once all nine are placed, optionally fading gradually with the count. | SudokuPad Mark Solved Digits: On / Gradual / Count [6]; sudoku.coach "Dim Digits when all are placed" [1] | C | No | Easy | Easy | Small | **v1** as a variant of the digit pad (dim on completion; skip "gradual") |
| 8 | **Bi-value cell highlight** | Highlight cells whose notes hold exactly two digits. | sudoku.coach Highlight pad [1]; SudokuWiki "Show bi-value cells" [14] | T | Low. It reads only the player's notes. | Easy (background) | Easy | Small | **v1** as a mode of the candidate filter |
| 9 | Bi-local / strong-link highlight | For digit d, mark units where d has exactly two spots, which is an automatic Snyder. | sudoku.coach "Colour Bi-Locals" [1]; SudokuWiki strong links [14] | T | Yes, if shown automatically at low rungs: it does the Snyder pass for the player. | Colour pair cells; no lines | Colour only | Medium | later (Learn + hint rung display only) |
| 10 | Cell colouring | Background colour on whole cells. | SudokuPad colour mode [5]; SudokuExchange [3]; sudoku.coach Cell Colours [1]; HoDoKu [15]; Simple Sudoku [16] | T + C | No | Easy (bg) | Easy (bg) | Small to medium | Already LATER: keep |
| 11 | Candidate colouring in **pairs** | Colour individual candidates, where each colour has an "opposite" for two-colour (simple) colouring. | HoDoKu five colour pairs, Shift for opposite [15]; sudoku.coach Candidate Colours [1] | T | No | Foreground colour per candidate | Same, but cramped | Medium | later (after cell colouring) |
| 12 | Colour dots / letters | Mark "these cells hold the same unknown digit". | sudoku.coach Colour Dots and letters [1]; SudokuPad letter tool [5] | C | No | Possible | Poor | Medium | skip (mainly for variant puzzles) |
| 13 | Drawn links / arrows / pen lines | Draw strong and weak links, or freehand lines, between candidates or cells. | sudoku.coach Links/Arrows, Quick Draw [1]; SudokuPad pen tool [5] | T | No | Very hard (box-drawing between sub-positions) | Not feasible | Large | skip (show chains as cell colours plus text in hints) |
| 14 | **Hide all notes (temporary)** | One key hides notes and colours so the bare grid shows, and pressing it again restores them. | SudokuExchange "Hide/Show Pencilmarks" (P) [3] | C | No | Trivial | Trivial; most useful on phone | Small | **v1** |
| 15 | Clear notes by layer / clear all notes | Delete only the active layer in a cell, or wipe every note. | sudoku.coach "Delete Key first deletes according to active Pad", "Delete all candidates" [1]; SudokuExchange "Clear all pencil marks" [3] | C | No | Easy | Easy | Small | **v1** (per layer once #1 exists; clear-all behind undo) |
| 16 | Multi-cell selection | Select several cells and toggle a note in all of them. | SudokuPad [5]; SudokuExchange [3]; sudoku.coach [1]; Simple Sudoku [16] | C | No | Shift+arrows; selection bg | Same | Medium (input model change) | later (worth it once #1 lands; Snyder is entered two cells at a time) |
| 17 | Seen-by-selection highlight | Shade the cells that every selected cell sees, i.e. the intersection. | f-puzzles "Highlight Cells Seen by Selection" [7]; SudokuPad "Show Seen Cells" [6] | T | No | Easy | Easy | Small (after #16) | later; for a single cell this is the planned row/col/box shading |
| 18 | Filter polarity + multi-digit AND/OR | The filter shows either allowed cells (green) or excluded cells (red), and can combine digits with AND/OR. | HoDoKu F1–F9, Shift and Ctrl modifiers [15] | T | Low | Easy | Easy | Small | later (variant of planned filter) |
| 19 | Good Sudoku "Focus mode" | Highlight where d is placed and every square those placements affect, which is cross-hatching made visible. | Good Sudoku [9][17] | T | Low | Easy | Easy | Small | Variant of planned candidate filter: decide basis (see notes) |
| 20 | Focus / dim-all-but-selection | Dim everything except chosen cells, so you can point at part of the grid. | sudoku.coach Focus pad ("usually used to teach others") [1] | T | No | Easy | Easy | Small | later (reuse inside Learn and hint rendering, not as player tool) |
| 21 | Legal-digit pad | Selecting a cell greys out pad digits that cannot go there. | Good Sudoku [17] | — | **Yes**: does the "what fits here" scan | Easy | Easy | Small | skip |
| 22 | Auto-place singles / last digit in unit | Fill every naked single at once, or the last missing digit of a unit. | HoDoKu "Set all singles" (F11) [15]; sudoku.coach Auto Naked Singles [1]; Good Sudoku last digit [18] | — | **Yes**: removes the thinking sumi teaches | Easy | Easy | Small | skip |
| 23 | Progress / remaining-difficulty indicator | Show the share of puzzle difficulty still left and the hardest step remaining. | HoDoKu progress indicator [15] | — | Leaks hint info; reads like a stat | Easy | Tight | Small | skip |
| 24 | Compact centre marks ("1-5") | Collapse long centre-mark runs into ranges. | SudokuPad Compact Pencilmarks [6] | — | No | n/a (positional grid) | n/a | — | skip |
| 25 | Dot notation | Candidates drawn as dots in a 3x3 pattern instead of digits. | Sudopedia [12] | C | No | Possible | Possible, but digits already fit | Small | skip |
| 26 | Hint modes: strategy / pattern / eliminations | Hints show only the technique, only the pattern, or only the eliminations. | SudokuWiki Hints Mode [13]; HoDoKu vague / concrete / show step [15] | T | No | — | — | — | Already planned (5-rung ladder): confirms design |
| 27 | Hint reads the player's notes | The hint engine uses the player's cell candidates. Box marks are ignored, and an empty cell counts as "nothing ruled out". | sudoku.coach Hint System FAQ, Box Candidates text [1] | T | — | — | — | Design decision | **v1 decision** (see notes) |

## Notes on recommended items

**1. Box marks (Snyder layer), v1.** This is the item the user asked about, and the most valuable addition. It is a variant of the "corner notes" already on the LATER list. I recommend pulling a narrow form of it into v1 for one reason: sudoku.coach's Basics teach box candidates first, then cell candidates, then full notation [1]. That is the bridge from "no notes" to "full notes" that a teaching game needs. Keep it narrow:
- one extra layer
- the same 3x3 positional digits as candidates, distinguished only by style (for example bold or accent for box marks, dim for candidates)
- one key to switch the active layer

In the dense layout, position plus style is the only option, so the layer cannot work on a monochrome terminal. Fall back to reverse video, or check how Termux renders bold. Pushback: this doubles the input modes. Do not also add a separate centre layer (#4), because sumi's mini-grid already is the centre layer.

**3. Convert box marks to candidates, v1 with #1.** One key, a small change, and it matches how solvers actually work: a box pair turns out to be a naked pair [3].

**6. Check notes against placed digits, v1.** This extends rule-conflict marking to notes, and SudokuPad ships it as a sub-option of its conflict checker [6]. It uses only the rules, never the solution, so it stays calm and honest. Give it its own toggle.

**7. Digit-completion dimming, v1 (small).** sumi's digit pad already counts what remains. SudokuPad's "On" mode fades a digit only when all nine are placed, and that is enough [6]. Skip "Gradual": a fading grid makes progress feel like a meter.

**8. Bi-value highlight, v1.** It is one more mode on the planned candidate filter, it reads only the player's own notes, and it supports later lessons on naked pairs, XY-Wing and BUG [1][14].

**14. Hide notes, v1.** One key, trivial to build, and the most calming tool on a cramped phone screen [3].

**15. Clear by layer / clear all, v1.** Once there are two layers, Delete must clear only the active one, as on sudoku.coach [1]. "Clear all notes" should be one undo step.

**27. Decision for the hint ladder, v1.** sudoku.coach's hints use the player's cell candidates and ignore box marks. A cell with no notes counts as "nothing ruled out", and a wrong note breaks the hints, so the app says the solution must always stay among the candidates [1]. sumi needs the same rules written down. Box marks never feed the solver. When notes contradict the solution, the hint ladder should say so first (for example "a note in r4c2 rules out the real digit") before it gives any technique.

**19. Planned candidate filter: pick its basis deliberately.** Good Sudoku's Focus shows placements and the squares they affect, which is cross-hatching made visible [9]. HoDoKu's filter shows allowed or excluded cells and can combine digits [15]. A filter based on placed digits is the calmer, teaching-safe default: it shows what the player could scan, not what their notes claim. A filter based on notes suits advanced play. Offer both later, with polarity (#18).

## Sources

- [1] sudoku.coach. App labels and English help strings, from https://sudoku.coach/en/play, its bundle https://sudoku.coach/js/jCfXtZf-1776367508519.js and its translation file https://sudoku.coach/transClient/en.json (PAD_DESCRIPTION_BOX_CANDIDATES, _CELL_CANDIDATES, _HIGHLIGHT, _CANDIDATE_COLOURS, _LINKS_ARROWS, _COLOUR_DOTS, _FOCUS, _BOOKMARKS, _CANDIDATE_HELPERS, HINT_SYSTEM_FAQ; option labels "Highlight Impossible Candidates", "Dim Digits when all are placed", "Colour Bi-Locals", "Cell&Box Notation", "Delete Key first deletes according to active Pad"; Basics lesson order "Box Candidate Notation", then "Cell Candidate Notation", then "Full Candidate Notation").
- [3] SudokuExchange (grantm/sudoku-web-app): README https://github.com/grantm/sudoku-web-app and in-app help https://github.com/grantm/sudoku-web-app/blob/master/src/components/help/help.js (outer = Snyder, inner = pairs/triples, "." converts outer to inner, P hides pencil marks, auto-clean, highlight matching/conflicting digits, clear all pencil marks). Settings: https://github.com/grantm/sudoku-web-app/blob/master/src/components/modal/modal-settings.js
- [5] SudokuPad (Sven Neumann; used by Cracking the Cryptic): controls page https://sudokupad.app/ (Z/X/C/V digit/corner/centre/colour modes) and app tool modules https://sudokupad.app/tool-pen.js, https://sudokupad.app/tool-letter.js, https://sudokupad.app/tool-color.js
- [6] SudokuPad feature modules: https://sudokupad.app/feature-conflictchecker.js ("Check Pencilmarks"), https://sudokupad.app/feature-marksolveddigits.js (On / Gradual / Count, default off), https://sudokupad.app/feature-showseencells.js, https://sudokupad.app/feature-compactmarks.js
- [7] f-puzzles settings: https://www.f-puzzles.com/Settings.js ("Highlight Cells Seen by Selection", "Highlight Conflicts", "Treat Pencil Marks as Given")
- [8] MacStories on Good Sudoku (accent notes, cross-out notes, highlights): https://www.macstories.net/?p=63912
- [9] VICE on Good Sudoku (Focus mode, automatic notes): https://www.vice.com/en/article/good-sudoku-is-making-me-good-at-sudoku
- [10] Minimal Sudoku, Snyder guide (box-only, switch to full notes when stuck): https://www.minimal-sudoku.com/learn/snyder
- [11] Quatrian, pencil marks (Snyder for singles and naked pairs, then full marks): https://quatrian.com/sudoku/pencilmarksalg.html
- [12] Sudopedia, Pencilmark (3x3 dot pattern; complementary pencilmarks): https://sudopedia.sudocue.net/index.php?title=Pencilmark
- [13] SudokuWiki Colour Sudoku solver page (Hints Mode, bi-value cells, strong links): https://www.sudokuwiki.org/colorsudoku.htm
- [14] SudokuWiki solver (Show bi-value cells, strong links): https://www.sudokuwiki.org/sudoku.htm
- [15] HoDoKu playing manual (filters F1–F9 with Shift/Ctrl, five colour pairs A–E with Shift for opposite, Set all singles F11, three hint levels, Show deviations, progress indicator): https://hodoku.sourceforge.net/en/docs_play.php
- [16] Simple Sudoku (filters, colour markers, multi-cell selection): https://www.angusj.com/sudoku/
- [17] TapSmart review of Good Sudoku (only playable digits shown for a selected square, auto-note): https://www.tapsmart.com/games/good-sudoku-sudoku-finally-finds-ideal-form/
- [18] Tools and Toys on Good Sudoku (last digit in a unit offered on tap; hint shows next logical move): https://toolsandtoys.net/good-sudoku-for-ios-by-zach-gage-and-jack-schlesinger/
- [19] Thomas Snyder (three-time World Sudoku Champion; The Art of Sudoku, 2012): https://en.wikipedia.org/wiki/Thomas_Snyder

### Gaps and caveats

- **Andoku:** I could not open a feature description beyond store blurbs ("numerous assistance functions"), so I make no Andoku-specific claims.
- **Logic Masters Deutschland:** it hosts puzzles rather than a player tool of its own, and I did not verify which players it links to.
- **Cracking the Cryptic's corner/centre convention:** this is attributed through SudokuPad's modes and SudokuExchange's help, not a CtC primary page. The search results for it were low-quality SEO pages, which I left out.
- **Snyder's scope:** guides disagree on whether it covers boxes only [10] or also rows and columns. Treat the box-only form as canonical.
- **Strike-through and bold:** whether these render in Termux and other terminals is not verified here. Test them before relying on them.
