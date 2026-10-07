# Hint wording for sumi: names, definitions, templates

Research date: 7 October 2026.

## Recommendations

**Notation**
- Cells: `r2c3`, lowercase. Sudopedia calls it the most widely used format. Avoid `A1`: SudokuWiki puts letters on rows, Sudoku Explainer puts them on columns.
- Whole lines and boxes in words: "row 2", "column 8", "box 5". Never a bare `c3`.
- Print row and column numbers on the board edges, and gloss `r2c3` once in Learn and help.
- List up to 3 cells. Past that, give a count and the house ("4 cells in column 8") and let the highlight show them.

**Glossary.** Use one word for each idea everywhere:
- digit, cell, row, column, box
- candidate: a digit that may go in a cell
- spot: a cell where a digit may go
- sees: shares a row, column or box with
- place, remove
- pair, triple, quad

Never print house, unit, region, block, note or pencil mark.

**Style**
1. Put the effect first ("Remove 4 from r5c3."), then one "because" sentence.
2. Use present tense and active voice. Don't use "we", "obviously" or "simply".
3. Name the digit every time.
4. Fit each rung in 4 lines at 30 columns.
5. Don't use colour words. Call the colouring groups A and B.
6. Write names in sentence case, keeping the fixed forms X-Wing, Y-Wing and so on.

## 1. Naming table

| # | Technique | Recommended name | Alternatives to mention once | Sources |
|---|---|---|---|---|
| 1 | full house | **Full house** | "last digit" (sudoku.coach), "last free cell" (Sudoku.com). Warning: HoDoKu and Sudopedia use "Last Digit" for something else, the 9th copy of a digit, so don't use it as the name | HoDoKu singles; sudoku.coach last-digit; Sudoku.com rules p1; Sudopedia |
| 2 | hidden single | **Hidden single** | "last remaining cell" (Sudoku.com); "pinned digit" (Sudopedia); finding method "cross-hatching" | HoDoKu; sudoku.coach; Sudoku.com p2; Sudopedia |
| 3 | naked single | **Naked single** | "obvious single", "last possible number" (Sudoku.com); "forced digit" (Sudopedia) | HoDoKu; sudoku.coach; Sudoku.com p1; Sudopedia |
| 4 | pointing | **Pointing pair** (say "pointing triple" when there are 3 cells; the internal ID stays `pointing`) | "pointing" (HoDoKu, Sudoku Explainer); "locked candidates type 1" (HoDoKu, Sudopedia); "box-to-line reduction" (sudoku.coach) | SudokuWiki Intersection Removal; HoDoKu intersections; Sudopedia; sudoku.coach locked-candidate; Sudoku.com p2; SE `LockingHint.java` |
| 5 | claiming | **Claiming** | "box/line reduction" (SudokuWiki, Sudopedia); "locked candidates type 2" (HoDoKu). Warning: sudoku.coach's "box-to-line reduction" means *pointing*, so "box/line" names mean opposite things on different sites | same as row 4 |
| 6–7 | naked pair, hidden pair | **Naked pair / Hidden pair** | "obvious pair" (Sudoku.com); "naked/hidden subset", "locked pair" (a special case) (Sudopedia) | SudokuWiki; HoDoKu; sudoku.coach; Sudoku.com; Sudopedia |
| 8–9, 11, 13 | triples, quads | **Naked triple / Hidden triple / Naked quad / Hidden quad** | "obvious triple" (Sudoku.com) | as above |
| 10 | X-Wing | **X-Wing** | Sudopedia calls it a size-2 "fish" | all sources agree |
| 12 | Swordfish | **Swordfish** | Sudopedia calls it a size-3 "fish" | all sources agree |
| 14 | Skyscraper | **Skyscraper** | a kind of "turbot fish" (HoDoKu, Sudopedia; SE "Turbot Crane" family). SudokuWiki has no page for it; a commenter there asked why | HoDoKu sdp; SE `TurbotFishHint.java`; sudoku.coach; SudokuWiki Strategy Families |
| 15 | 2-String Kite | **2-String Kite** | "Two-string kite" (sudoku.coach, SE); turbot fish family | HoDoKu sdp; sudoku.coach; SE |
| 16 | W-Wing | **W-Wing** | none | HoDoKu wings; SudokuWiki |
| 17 | XY-Wing | **Y-Wing** (see note) | "XY-Wing" (HoDoKu, SE, Sudopedia) | SudokuWiki Y-Wing; sudoku.coach y-wing; Sudoku.com p3; HoDoKu wings |
| 18 | XYZ-Wing | **XYZ-Wing** | none | HoDoKu; SudokuWiki; sudoku.coach |
| 19 | UR type 1 | **Unique rectangle** (type 1 only in sumi, so the "type" can go) | "deadly pattern" (the 4-cell swap it prevents); "uniqueness" family | SudokuWiki Unique Rectangles; Sudopedia |
| 20 | simple colouring | **Simple colouring** | "singles chains" (SudokuWiki); "simple colors" (HoDoKu); its two rules, "colour trap" and "colour wrap" (HoDoKu, Sudopedia); US spelling "coloring" (sudoku.coach) | SudokuWiki Singles Chains; HoDoKu col; sudoku.coach; Sudopedia |

**Note on Y-Wing.** The beginner-facing sources all say "Y-Wing": SudokuWiki, sudoku.coach and Sudoku.com, which has the biggest audience. The solver and forum sources say "XY-Wing". The only reason to keep "XY-Wing" is that it pairs with XYZ-Wing. My view is that the larger audience matters more than the pairing, but this one is a judgement call.

## 2. Definitions and templates

There are four rungs:
- Rung 1: the shared difficulty line, e.g. "A Hard step is available. Nothing easier."
- Rung 2: "Look for a {name}." followed by the definition.
- Rung 3: the area line.
- Rung 4: the cell highlights.
- Rung 5: the effect line, then the reason line.

Every definition is 61 characters or fewer, measured by script. Every example rendering fits in 4 lines or fewer at 30 columns, also checked by script.

**Variables**

| Variable | Meaning |
|---|---|
| `{digit}`, `{d1}`, `{d2}`, `{z}` | digits |
| `{digits}` | "1, 4 and 6" |
| `{house}`, `{line}` | "row 4", "column 2" or "box 5" |
| `{box}` | a box |
| `{rows}`, `{cols}` | "rows 2 and 7" |
| `{cell}` | one cell, e.g. r2c3 |
| `{cells}` | up to 3 coordinates, or a count plus the house |
| `{pattern}` | the pattern cells |
| `{pivot}`, `{wing1}`, `{wing2}`, `{end1}`, `{end2}` | named pattern cells |

**Row and column versions.** X-Wing, Swordfish and Skyscraper need a row version and a column version. Choose between them from the actual pattern. That gives away the orientation at rung 2, which seems acceptable. If it doesn't, write "lines" and explain the word once.

| # | Name | Rung 2 definition (≤61 chars) | Rung 3 area | Rung 5 result (effect / reason) |
|---|---|---|---|---|
| 1 | Full house | A row, column or box with one empty cell left. | `{house}.` | Place {digit} in {cell}. / It is the only digit {house} lacks. |
| 2 | Hidden single | A digit with only one possible cell in a row, column or box. | `On digit {digit}, {house}.` | Place {digit} in {cell}. / No other cell in {house} can take a {digit}. |
| 3 | Naked single | A cell with only one candidate left. | `{cell}: its row, column and box.` | Place {digit} in {cell}. / Its row, column and box rule out every other digit. |
| 4 | Pointing pair | In a box, all spots for a digit lie in one row or column. | `On digit {digit}, {box} and {line}.` | Remove {digit} from {cells}. / {box} must put its {digit} in {line}, so the rest of {line} can't. |
| 5 | Claiming | In a row or column, all spots for a digit lie in one box. | `On digit {digit}, {line} and {box}.` | Remove {digit} from {cells}. / {line} must put its {digit} in {box}, so the rest of {box} can't. |
| 6 | Naked pair | Two cells in one row, column or box share just 2 candidates. | `{house}.` | Remove {d1} and {d2} from {cells}. / {pattern} must hold {d1} and {d2}, so no other cell in {house} can. |
| 7 | Hidden pair | Two digits whose only spots are the same two cells. | `Digits {d1} and {d2}, {house}.` | Remove the other candidates from {pattern}. / {d1} and {d2} need both cells in {house}. |
| 8 | Naked triple | Three cells whose candidates, together, are only 3 digits. | `{house}.` | Remove {digits} from {cells}. / {pattern} hold {digits} between them, so no other cell in {house} can. |
| 9 | Hidden triple | Three digits whose only spots are the same three cells. | `Digits {digits}, {house}.` | Remove the other candidates from {pattern}. / {digits} need all three cells in {house}. |
| 10 | X-Wing | Two rows where a digit fits only in the same two columns. | `On digit {digit}, {rows}.` | Remove {digit} from {cells}. / {rows} put their {digit}s in {cols}, so the rest of those columns can't. |
| 11 | Naked quad | Four cells whose candidates, together, are only 4 digits. | `{house}.` | Remove {digits} from {cells}. / {pattern} hold {digits} between them, so no other cell in {house} can. |
| 12 | Swordfish | Three rows where a digit fits only in the same three columns. | `On digit {digit}, {rows}.` | Remove {digit} from {cells}. / {rows} put their {digit}s in {cols}, so the rest of those columns can't. |
| 13 | Hidden quad | Four digits whose only spots are the same four cells. | `Digits {digits}, {house}.` | Remove the other candidates from {pattern}. / {digits} need all four cells in {house}. |
| 14 | Skyscraper | Two rows with 2 spots each for a digit, lined up at one end. | `On digit {digit}, {rows}.` | Remove {digit} from {cells}. / {end1} or {end2} must be {digit}, and these cells see both. |
| 15 | 2-String Kite | A digit's 2-spot row and 2-spot column that meet in a box. | `On digit {digit}, {row}, {col} and {box}.` | Remove {digit} from {cell}. / {end1} or {end2} must be {digit}, and {cell} sees both. |
| 16 | W-Wing | Two matching 2-candidate cells, joined by a digit's 2 spots. | `{d1} and {d2}; the link is {d1} in {line}.` | Remove {d2} from {cells}. / {line} puts its {d1} in {end1} or {end2}, so {wing1} or {wing2} must be {d2}. |
| 17 | Y-Wing | A cell with 2 candidates sees two cells that share a 3rd. | `Middle cell {pivot}; digit {z}.` | Remove {z} from {cells}. / Whatever {pivot} is, {wing1} or {wing2} must be {z}, and these cells see both. |
| 18 | XYZ-Wing | A Y-Wing whose middle cell also holds the shared digit. | `Middle cell {pivot}; digit {z}.` | Remove {z} from {cells}. / One of {pivot}, {wing1}, {wing2} must be {z}, and these cells see all three. |
| 19 | Unique rectangle | One solution means two digits can't swap in a rectangle. | `Digits {d1} and {d2}, {rows}, {cols}.` | Remove {d1} and {d2} from {cell}. / Otherwise {d1} and {d2} could swap in the other corners: two solutions. |
| 20a | Simple colouring (trap) | Link a digit's 2-spot pairs; one half of the chain is true. | `On digit {digit}: groups A and B.` | Remove {digit} from {cells}. / Either all A cells or all B cells are {digit}, and these cells see both. |
| 20b | Simple colouring (wrap) | (same) | (same) | Remove {digit} from every A cell. / Two A cells share {house}, so group A can't be the {digit}s. Then place {digit} in every B cell, as its own step if that is clearer. |

**Example renderings** (these are what the script checked)
- Pointing: "Remove 5 from r3c7 and r3c9. Box 1 must put its 5 in row 3, so the rest of row 3 can't." (3 lines at 30 columns)
- X-Wing: "Remove 4 from r5c3 and r9c8. Rows 2 and 7 put their 4s in columns 3 and 8, so the rest of those columns can't." (4 lines at 30 columns, 4 at 37)
- Y-Wing: "Remove 8 from r2c2. Whatever r2c7 is, r2c3 or r6c7 must be 8, and r2c2 sees both." (3 lines)

**Notes**
- **Naked subsets.** The definitions say "together" or "between them" on purpose. SudokuWiki readers kept asking which candidate combinations count as a naked triple. The common mistake is thinking every cell must hold all three digits.
- **Unique rectangle.** This step is only safe because sumi's puzzles have one solution. Learn mode should say that once.
- **Rung 3 for uniqueness and Y-Wing.** The area gives away less than the cells do, so keep it to the digit or digits and the lines involved.

## 3. What beginners find confusing

1. **A claim with no "why".** Reviews of the Killer Sudoku CTC app say its hints save space at the cost of logic and assert things that aren't evident. A Steam thread shows a developer admitting that many hint steps were missing. A CTC Steam thread is a player asking why a cell "must be" one value. Fix: every result rung ends with a reason sentence.
2. **Too much text.** Some Sudoku.com reviewers complain about "excessive explanation" and want a way to skip it. The rung ladder solves this as long as each rung stays short.
3. **Hints that are too hard.** Sudoku.com reviewers disliked being handed "Bowman's Bingo", a trial-and-error method, when easier steps existed. This supports sumi's easiest-step-first rule.
4. **Names that collide.**
   - "Pointing pair" against "naked pair": sudoku.coach warns that they have nothing to do with each other.
   - "Box/line reduction" means claiming on SudokuWiki but pointing on sudoku.coach (as "box-to-line").
   - "Last digit" means a full house on sudoku.coach but the 9th copy of a digit on HoDoKu and Sudopedia.

   Fix: one name, and the alternatives mentioned once.
5. **Naked and hidden work in opposite directions.** Naked subsets remove candidates from other cells. Hidden subsets remove candidates from inside the pattern. The effect lines above keep these visibly different: "from {cells}" for naked, "the other candidates from {pattern}" for hidden.
6. **Notation.**
   - The A1 style is ambiguous: SudokuWiki uses letters for rows, Sudoku Explainer uses letters for columns.
   - The sources written for beginners (sudoku.coach, Sudoku.com, Good Sudoku) mostly avoid coordinates and point at cells with highlights.
   - I found no study or forum thread on whether beginners read `r2c3` correctly. The `r2c3` recommendation is therefore an inference: it is standard, short enough for a 30-column panel (writing "row 2, column 3" takes 15 characters per cell), and its meaning shows if the board edges are numbered.
7. **Colour and tone.**
   - The colouring technique is described with colours, but a terminal theme may be monochrome, and some players are colour-blind. Groups A and B work in any theme.
   - The Vice writer liked that Good Sudoku never made her "feel stupid". Avoid words like "easy", "obviously" and "just".
8. **Prior art.** The Hintoku iOS app uses almost the same ladder as sumi: difficulty first, then name and tip, then a full visual guide. Its reviewers praise it, which supports the rung model. It also means sumi's wording should be distinct from Hintoku's.

## 4. Terminology choices

| Choice | Use | Avoid | Why |
|---|---|---|---|
| candidate | everywhere, including the mode name ("candidate mode", as NYT does) | note, pencil mark | Every technique source uses it, and the technique names depend on it ("locked candidates"). sudoku.coach says "pencil marks" is a looser word. Mention "pencil marks" once in help. |
| box | everywhere | block, square, region, nonet | Used by SudokuWiki and sudoku.coach, and matches "box/line". HoDoKu and Sudoku.com say "block". |
| row, column or box | definitions and reasons | house, unit, region, group | All of these mean the same thing (Sudopedia lists them as aliases). In templates, name the specific one: "row 4". |
| digit | everywhere | number, value | "Number" suggests arithmetic. HoDoKu and Sudopedia say "digit". |
| spot | for digit-focused techniques | position, conjugate pair, strong link | Plain English. Turns "strong link" into "only 2 spots". |
| sees | wings, kites, colouring | peer, buddy | Short. sudoku.coach and HoDoKu use it. Gloss it once. |
| remove / place | effect lines | eliminate, set, solve | Two plain verbs used consistently. |

## Sources

All pages were opened unless the entry says otherwise. Quotes from them are a few words at most.

**SudokuWiki (Andrew Stuart)**
- Intersection Removal: https://www.sudokuwiki.org/Intersection_Removal (comments: https://www.sudokuwiki.org/Intersection_Removal?talk)
- Singles Chains: https://www.sudokuwiki.org/Singles_Chains
- Getting Started: https://www.sudokuwiki.org/Getting_Started
- Y-Wing: https://www.sudokuwiki.org/Y_Wing_Strategy
- Unique Rectangles: https://www.sudokuwiki.org/Unique_Rectangles
- Strategy Families: https://www.sudokuwiki.org/Strategy_Families
- Hidden Candidates: https://www.sudokuwiki.org/Hidden_Candidates
- Naked Candidates comments: https://www.sudokuwiki.org/Naked_Candidates?talk
- *The Logic of Sudoku* (Andrew Stuart's book): https://www.sudokuwiki.org/The_Logic_of_Sudoku. I did not get the table of contents. The book is understood to use SudokuWiki's names.

**HoDoKu**
- Singles: https://hodoku.sourceforge.net/en/tech_singles.php
- Intersections: https://hodoku.sourceforge.net/en/tech_intersections.php
- Colouring: https://hodoku.sourceforge.net/en/tech_col.php
- Single-digit patterns: https://hodoku.sourceforge.net/en/tech_sdp.php
- Wings: https://hodoku.sourceforge.net/en/tech_wings.php

**Sudopedia (archived mirror)**
- Terminology: https://web.archive.org/web/2017/http://sudopedia.enjoysudoku.com/Terminology.html
- Locked Candidates: https://web.archive.org/web/2017/http://sudopedia.enjoysudoku.com/Locked_Candidates.html
- Cell: https://sudopedia.sudocue.net/index.php?title=Cell

**sudoku.coach.** The text was read through https://sudoku.coach/beapi/get-page/en/learn/{slug}.
- https://sudoku.coach/en/learn/last-digit
- https://sudoku.coach/en/learn/hidden-single
- https://sudoku.coach/en/learn/naked-single
- https://sudoku.coach/en/learn/locked-candidate
- https://sudoku.coach/en/learn/naked-pair
- https://sudoku.coach/en/learn/y-wing
- https://sudoku.coach/en/learn/simple-coloring
- https://sudoku.coach/en/learn/candidates
- https://sudoku.coach/en/learn/eliminate-candidates
- Skyscraper and kite: https://sudoku.coach/en/learn/skyscraper and https://sudoku.coach/en/learn/two-string-kite (search listings only; the page bodies did not load)

**Sudoku Explainer**, read through the SukakuExplainer fork (LGPL; only names were checked):
- `Cell.java` (A1 with a column letter by default, r1c1 as an option): https://github.com/SudokuMonster/SukakuExplainer/blob/master/diuf/sudoku/Cell.java
- `LockingHint.java` and `TurbotFishHint.java`: https://github.com/SudokuMonster/SukakuExplainer/tree/master/diuf/sudoku/solver/rules

**Sudoku.com**
- https://sudoku.com/sudoku-rules/
- https://sudoku.com/sudoku-rules/page/2/
- https://sudoku.com/sudoku-rules/page/3/
- Reviews: https://justuseapp.com/en/app/1193508329/sudoku-com-sudoku-puzzle/reviews

**Reviews and threads**
- Good Sudoku: https://www.vice.com/en/article/good-sudoku-is-making-me-good-at-sudoku and https://www.pocketgamer.com/articles/083617/good-sudoku-aims-to-achieve-the-impossible-by-teaching-me-how-to-play-and-enjoy-sudoku/
- Hintoku (App Store): https://apps.apple.com/app/6744828400
- Killer Sudoku CTC reviews: https://worldsapps.com/reviews-killer-sudoku-ctc
- Steam threads: https://steamcommunity.com/app/1471910/discussions/0/3005556719751237915 and https://steamcommunity.com/app/2085970/discussions/0/3758850346172087668
- NYT Sudoku candidate mode: https://thenewyorktimeshelpcenter.helpjuice.com/360011158491-New-York-Times-Games/30467696592148-Sudoku (search listing only)

**Gaps**
- Reddit blocked automated access, so there are no r/sudoku threads here.
- The live sudopedia.enjoysudoku.com site refused connections; I used the archive instead.
