# sumi: puzzle generation, technique order and rating

Researched 2026-10-06. **[V]** means verified from a source I opened or an experiment I ran. **[I]** means my own inference.

## 1. Generating hard puzzles

### Published distributions

- **[V]** Berthier (arXiv 1111.4083, Table 3) gives the NRCZT rating distribution for random minimal puzzles. Level 0 (roughly "singles only" [I]) covers 41.8% of top-down puzzles, 35.1% of controlled-bias puzzles and an estimated 29.2% of the real population. 99% are solvable with whips of length ≤5. Mean SER is 3.77 for top-down, 4.29 for controlled-bias and 4.73 estimated real. Top-down and bottom-up generators are biased towards easy puzzles with few clues.
- **[V]** Stuart (SudokuWiki): most random puzzles are easy, with a long tail. About 1 in 5,000 of his random puzzles cannot be solved by his whole strategy set.
- I found no published share for "X-Wing / wing / colouring as the hardest step". The forum was unreachable (connection timed out), so I measured it myself.

### Experiment [V]

- **Generator:** qqwing 1.3 (GPL-2+), top-down, no symmetry, 20,000 puzzles.
  - Mean 15.5 ms per puzzle. Mean 25.2 clues, close to Berthier's top-down 24.4.
  - qqwing's own levels: Simple 2.3%, Easy 46.1%, Intermediate 16.6%, Expert 35.0%.
  - qqwing's techniques are not ours. It knows only singles, pairs and intersections, and its "Expert" means "needed guessing".
- **Grader:** a throwaway Python grader with exactly sumi's technique list (`scratchpad/grader.py`).
  - It always applies the easiest step first.
  - Checked against brute-force solutions on 300 puzzles: 0 wrong eliminations. I did not measure how many patterns it misses.

Results with the ordering recommended in §2:

| sumi level | qqwing 20k random | sudoku-exchange `hard.txt` (SE 2.5–4.9), 2,000-puzzle sample |
|---|---|---|
| Easy (singles) | 48.4% | 0% |
| Medium (LC, pairs) | 16.6% | 40.3% |
| Hard (triples, quads, X-Wing, Swordfish) | **0.49%** | 10.7% |
| Expert (advanced list) | **18.3%** | 43.0% |
| Not solvable with sumi's set | 16.2% | 6.1% |

- Most common hardest step in the 20k set: Skyscraper 7.9%, W-Wing 5.1%, XY-Wing 2.5%, 2-String Kite 0.85%, UR1 0.9%, Simple Colouring 0.5%, XYZ-Wing 0.5%. X-Wing (0.12%) and Swordfish (0.04%) are rarely the bottleneck.
- 57% of Expert puzzles (2,088 of 3,655) need exactly one Expert step.
- With your original level split (Hard = triples and quads only), Hard is 0.34% (68 of 20,000).
- **Expert is not rare. Hard is the scarce level.** [I] Generate-and-filter in Rust gives about 5–6 candidates per Expert puzzle. At qqwing speed that is under 0.1 s each, so a few thousand take minutes.

### Mutation test [V]

- Method: start from the 68 Hard seeds. Move one clue (delete one, add one cell from the solution), keep the result if it is still unique, re-minimise it, then re-grade.
- Result: 254 of 906 neighbours (28%) were Hard, against 0.34% at random. That is about 80 times the yield.
- [I] Neighbours of one seed look alike. Cap neighbours per seed, and remove duplicates by canonical form, so that symmetry-equivalent copies count once.

### How existing tools target difficulty

- **[V] HoDoKu** generates and filters. It fills a grid, removes clues (optionally symmetric), rates the result and discards it on mismatch. Its manual warns that puzzles needing some advanced techniques are "likely never found".
- **[V] Stuart** removes symmetric quads, then pairs, then single cells while the solution stays unique, then grades. He rejects puzzles that are trivial apart from one bottleneck.

**Practical for a laptop [I]:** use your own Rust generator with generate-and-filter for every level. Add one-clue mutation from Hard seeds to fill the Hard bank. All of it is your own code with no licence questions.

### Banks and licences

| Source | Licence | Ship in MIT? |
|---|---|---|
| grantm/sudoku-exchange-puzzle-bank (≈1M puzzles, qqwing-generated, SE-rated; `hard.txt` 321,592, `diabolical.txt` 119,681) | Public-domain dedication [V] | **Yes** [V licence; I legal]. Re-grade with sumi. By the sample rates above, `hard.txt` alone holds about 34k Hard and 138k Expert puzzles [I]. |
| Kaggle radcliffe "3 million Sudoku puzzles with ratings" | CC0 [V, Kaggle API] | Yes. Its ratings measure search-tree depth, not human technique, so re-grade. |
| HoDoKu | GPL-3.0 code [V] | No code. Puzzles you generate with it are probably not covered [I, not legal advice]. |
| Berthier controlled-bias collection, Sudoku-classif | GPL-3.0 repos [V] | Avoid. Also mostly too hard. |
| tdoku `data.zip` | BSD-2 code; the data mixes Kaggle and forum collections [V] | Avoid: provenance unclear. |
| qqwing | GPL-2+ [V] | Fine as an offline tool. Don't copy its code. |
| Forum lists (top95, top1465, …) | No licence found (not verified) | No. Also far beyond sumi's techniques. |

## 2. Technique order and rating

### Comparison table

- HoDoKu columns, from `Options.java`: order index, base score, level. **[V]**
- SE ratings, from Sudoku Explainer source (1to9only fork): `*Hint.getDifficulty()`. **[V]**
- SudokuWiki, from its solver checklist at sudoku.htm as fetched today. **[V]**

| Technique | HoDoKu index / score / level | SE | SudokuWiki solver # |
|---|---|---|---|
| Full House | 100 / 4 / Easy | 1.0 | 1 Singles |
| Naked Single | 200 / 4 / Easy | **2.3** | 1 |
| Hidden Single | 300 / 14 / Easy | 1.2 box, 1.5 line | 1 |
| Pointing | 1200 / 50 / Medium | 2.6 | 3 |
| Claiming | 1210 / 50 / Medium | 2.8 | 4 |
| Naked Pair | 1300 / 60 / Medium | 3.0 | 2 |
| Hidden Pair | 1500 / 70 / Medium | 3.4 | 2 |
| Naked Triple | 1400 / 80 / Medium | 3.6 | 5 |
| Hidden Triple | 1600 / 100 / Medium | 4.0 | 5 |
| Naked Quad | 2000 / 120 / Hard | **5.0** | 14 |
| Hidden Quad | 2100 / 150 / Hard | **5.4** | 14 |
| X-Wing | 2200 / 140 / Hard | **3.2** | 7 |
| Swordfish | 2300 / 150 / Hard | 3.8 | 15 |
| Skyscraper | 3000 / 130 / Hard | no named rule: X-cycle, 6.5+ [I from code] | not listed; part of X-Cycles (17) |
| 2-String Kite | 3100 / 150 / Hard | same as Skyscraper [I] | same |
| W-Wing | 3200 / 150 / Hard | chain, 6.5+ [I] | 11 |
| XY-Wing | 3300 / 160 / Hard | 4.2 | 10 |
| XYZ-Wing | 3400 / 180 / Hard | 4.4 | 16 |
| UR type 1 | 3500 / 100 / Hard | 4.5 | **8** |
| Simple Colouring | **5330** / 150 / Hard | X-cycle or chain, 6.5+ [I] | 13 |

### Where they disagree

1. **Naked versus hidden.** SE rates a naked single (2.3) above a hidden single (1.2–1.5) and above pointing (2.6 is close). HoDoKu has the reverse for singles, and puts naked subsets before hidden subsets of the same size.
2. **Pairs versus triples.** HoDoKu searches Naked Triple (1400) before Hidden Pair (1500), but scores Hidden Pair lower. SudokuWiki does all pairs before any triple.
3. **X-Wing.** SE puts X-Wing (3.2) below Naked Triple (3.6). HoDoKu and SudokuWiki put it after all triples.
4. **Quads.** SE rates them (5.0/5.4) above XY-Wing and UR1, which means harder than most of your "Expert" list.
5. **Single-digit patterns.** SE has no rule for Skyscraper or Kite, so it rates them as chains (6.5+). HoDoKu calls them Turbot-Fish variants (4-link X-chains), scores 130/150, and puts them in the same level as X-Wing.
6. **Simple Colouring.** HoDoKu searches it near the end (after finned fish). SudokuWiki puts it mid-"tough", before Quads and Swordfish. SudokuWiki's own Strategy Families page orders it before Y-Wing, which disagrees with its own solver.
7. **UR1.** SudokuWiki runs it early (#8, right after X-Wing). HoDoKu and SE put it after XYZ-Wing.
8. **Levels.** HoDoKu's levels are Easy, Medium, Hard, Unfair and Extreme, with score caps of 800, 1000, 1600 and 1800. The puzzle level is the higher of two things: the level of its hardest step, or the band its summed score falls in. The generator also rejects puzzles that score too low for the target level.

### Recommended order and levels

The hint search runs in this order and stops at the first technique that changes something:

1. **Easy:** Full House, Hidden Single (box, then line), Naked Single.
2. **Medium:** Pointing, Claiming, Naked Pair, Hidden Pair.
3. **Hard:** Naked Triple, Hidden Triple, **X-Wing**, Naked Quad, **Swordfish**, Hidden Quad.
4. **Expert:** Skyscraper, 2-String Kite, W-Wing, XY-Wing, XYZ-Wing, UR1, Simple Colouring.

Why:
- **Easy order.** Full House first matches all three tools. Hidden Single before Naked Single matches SE and pen-and-paper practice (you can spot it without notes). HoDoKu's opposite order is a cheap-to-compute choice, and either is defensible.
- **Fish in Hard.** Moving basic fish into Hard follows HoDoKu (same level as quads) and SE (X-Wing 3.2 < triples). It also widens the thin Hard band from 0.34% to 0.49% at random and to 10.7% in the sudoku-exchange sample. Within Hard, sort by HoDoKu score.
- **Expert order.** It follows HoDoKu's index order. Skyscraper and Kite are a Simple Colouring trap on a 4-cell chain, so searching colouring first would hide them. With colouring last it only fires on longer chains, which are harder to see. That matches HoDoKu and the chain ratings in SE.
- **Check X-Wing first.** A Skyscraper whose roofs line up is an X-Wing, so X-Wing must be tried before Skyscraper.

### Is "hardest technique" enough?

- **[V]** HoDoKu adds base scores for every step and lets the sum promote the level. Stuart weights eliminations and how many opportunities there are each round, and rejects puzzles with one bottleneck.
- **[V]** 57% of sumi Expert candidates contain exactly one Expert step.
- **Recommendation [I]:**
  - Keep the level as the hardest technique. It is easy to explain and matches your level definitions.
  - Store a secondary score: the sum of HoDoKu-like base scores per step. Use it to sort within a level, to show stars, and to filter: an Expert bank puzzle should need at least 2 Expert steps, or reach a score threshold.
  - The rating depends on the search order. Swapping Skyscraper and X-Wing moved hundreds of puzzles between categories, and moving W-Wing earlier changed the XY-Wing count from 838 to 494. Version the order and re-grade the bank whenever it changes.

## 3. Algorithm notes [I unless marked]

- **UR1**
  - Sound only for puzzles with a unique solution. **[V]** HoDoKu's UR page says uniqueness "is not part of the Sudoku rule" and that givens break avoidable rectangles.
  - Requirements:
    - All 4 cells are unsolved and span exactly 2 rows, 2 columns and 2 boxes.
    - 3 of them are exactly {a,b}.
    - The 4th contains a and b.
  - Turn UR off for imported puzzles until uniqueness has been verified.
  - Before any hint, check the user's placements against the solution. A wrong digit can make uniqueness logic "prove" false things.
- **Efficiency**
  - Fish: use per-digit 9-bit line masks. There are only C(9,2)=36 and C(9,3)=84 line combinations per digit and orientation.
  - Wings: keep a bivalue-cell list and peer masks as `u128`. The naive Python version graded 20k puzzles in about 3 minutes on 4 cores, so Rust needs no clever indexing.
- **Determinism**
  - Use a fixed technique order and scan canonically: digit, then unit type, then unit index, then cell.
  - Within a technique, tie-break by most eliminations, then lowest cell index.
  - Never iterate a Rust `HashMap` or `HashSet` when choosing a hint, because their iteration order is randomised.
  - Seed the generator explicitly.
- **User notes**
  - Run the engine on its own full candidate grid: candidates implied by the placements, minus eliminations the engine itself justified. Never run it on the user's notes. Patterns like "only two places" are unsound when the notes are incomplete.
  - Hint order:
    1. A wrong placement, compared against the solution.
    2. A note that removed the true digit.
    3. The first engine step whose eliminations or placement are not already shown in the user's notes. Apply steps the user has already done silently.
  - With empty notes, offer only singles, or suggest filling in candidates.
- **Pattern details**
  - Kite: the two box-end cells must be distinct, and the other ends must lie outside that box.
  - W-Wing: neither end of the strong link may be one of the wing cells, and the two wing cells must not see each other.
  - Colouring: handle both wrap (two same-coloured cells see each other) and trap (an uncoloured cell sees both colours).
  - Always require at least one elimination before counting a step.

## Sources

- HoDoKu source: `Options.java` (DEFAULT_SOLVER_STEPS, difficulty levels), `SudokuSolver.java`, `SudokuGenerator.java`. https://github.com/PseudoFish/Hodoku
- HoDoKu manual: https://hodoku.sourceforge.net/en/docs_solv.php · https://hodoku.sourceforge.net/en/docs_cre.php
- HoDoKu technique pages, opened: https://hodoku.sourceforge.net/en/tech_sdp.php (Skyscraper, 2-String Kite, Turbot) · https://hodoku.sourceforge.net/en/tech_ur.php · https://hodoku.sourceforge.net/en/tech_intro.php
- HoDoKu technique pages from the site index, not opened: tech_singles.php, tech_intersections.php, tech_naked.php, tech_hidden.php, tech_fishb.php, tech_wings.php, tech_col.php (all under https://hodoku.sourceforge.net/en/)
- Sudoku Explainer source (LGPL): `LockingHint`, `NakedSetHint`, `HiddenSetHint`, `XYWingHint`, `UniqueLoopHint`, `CycleHint`, `ChainingHint`. https://github.com/1to9only/SudokuExplainer
- SudokuWiki solver checklist: https://www.sudokuwiki.org/sudoku.htm · Strategy Families: https://www.sudokuwiki.org/Strategy_Families · X-Cycles: https://www.sudokuwiki.org/X_Cycles · Rectangle Elimination: https://www.sudokuwiki.org/Rectangle_Elimination
- SudokuWiki definitions (checked that they exist): Naked_Candidates, Hidden_Candidates, Intersection_Removal, X_Wing_Strategy, Sword_Fish_Strategy, Y_Wing_Strategy, XYZ_Wing, W_Wing_Strategy, Simple_Colouring, Unique_Rectangles (all under https://www.sudokuwiki.org/)
- A. Stuart, "Sudoku Creation and Grading": https://www.sudokuwiki.org/Sudoku_Creation_and_Grading.pdf
- D. Berthier, "Unbiased Statistics of a CSP – A Controlled-Bias Generator": https://arxiv.org/pdf/1111.4083
- Banks: https://github.com/grantm/sudoku-exchange-puzzle-bank · https://www.kaggle.com/datasets/radcliffe/3-million-sudoku-puzzles-with-ratings · https://github.com/denis-berthier/Controlled-bias_Sudoku_generator_and_collection · https://github.com/t-dillon/tdoku
- Experiment files in the scratchpad: `grader.py`, `grader_b.py`, `mutate.py`, `check.py`, `qq20k.graded`, `qq20k_b.graded`
