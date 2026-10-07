# sumi Learn mode: how teaching products structure lessons and drills, and what sumi should do

Research date: 7 October 2026. Every claim has a URL in the text or in the Sources list. Lesson text in other products was read to understand structure only; nothing here is copied, and sumi's lesson text should be written from scratch.

## Short answer

**Lesson (one per technique, about 5 minutes):** four parts, each one asking the player to do a little more.
1. **See it:** a worked example drawn on the board one frame at a time, with one sentence per frame, moved forward with Space.
2. **Finish it:** 1 or 2 positions where sumi shows the pattern and the player enters the result, or sumi shows the result and the player picks the pattern cells.
3. **Find it:** 3 positions with no help.
4. **Done:** the lesson is marked as seen and its drill opens.

**Answer:** the player makes the move with the normal game keys: place a digit, or strike candidates and press Enter. Pattern cells can be picked with Space if the player wants to explain the step. Naming the technique is never required.

**Checking:** check the logic, not one stored answer. Accept any valid instance of the technique on the board. Accept partial eliminations as correct and show the rest. When something is wrong, name the exact conflict, never "failed", and let the player try again.

**Drills:** a few boards per session drawn from techniques that are due, on day-based intervals (1, 3, 7, 21 days). Mix techniques once two are learned. Show no due counts, keep no backlog, and give no scores. A technique counts as learned after three unaided finds on at least two days, and finds in normal games count too.

**Hint link:** when the second rung (technique name) names a technique the player has not learned, offer its lesson. Leave out stars, par, streaks, timers, XP and percentages.

---

## 1. Sudoku teaching products

### sudoku.coach (Jan Feldmann): the closest model, and the best regarded on r/sudoku

Opened directly in a browser (the pages are rendered by JavaScript).

- **Campaign structure.** The campaign is a map of chapters ([campaign](https://sudoku.coach/en/campaign)):
  - Main chapters carry a count of items, usually 10 to 13 for a technique chapter. Examples are "Your first Technique" (10), "Lines are King" (12), "Teaming Up" (12), "X-Wing" (12) and "Skyscraper" (12).
  - "BONUS" chapters branch off the path for each difficulty band (Very Easy, Easy, Moderate, Hard, Fiendish and so on).
  - Inside a chapter the items are typed: **TIP** cards, one **LESSON**, numbered **puzzles** (01–04), a **BOSS** puzzle and an **ENDLESS** mode. In the first chapter, a "Hints" tip comes between the puzzles.
  - So each technique gets one explanation, then whole puzzles that need it, then a harder capstone, then unlimited practice.
- **Library order** ([learn index](https://sudoku.coach/en/learn)):
  - Rules, then notation (box, cell and full candidate notation), then techniques by tier.
  - Basic tier: Last Digit, Hidden Single, Hidden Single (Line), Naked Single.
  - Simple tier: **Hidden Pair, Locked Candidate, Hidden Groups, Naked Pair**.
  - Hidden single comes before naked single, and hidden pair before naked pair.
- **How a technique is presented** ([hidden single](https://sudoku.coach/en/learn/hidden-single), [naked pair](https://sudoku.coach/en/learn/naked-pair)):
  - "Lesson mode" is a slideshow of board frames, each with a single short sentence, moved forward with "Continue". The naked pair lesson has 37 frames.
  - It builds the idea from a technique already taught (reducing a cell's candidates, then the hidden pair).
  - It spends many frames on **how to find** the pattern in a real grid, for example starting from the cells that see the most givens. It also covers **which notation** the technique needs. Naked pairs need cell candidates, which box-corner (Snyder-style) marks do not hold.
  - It closes with example boards and a colour legend: pattern cells in one colour, eliminations in another.
  - The library page itself recommends learning through the campaign, so the lesson and practice in context come together.
- **Practice per technique** ([practice index](https://sudoku.coach/en/practice), [naked pair practice](https://sudoku.coach/en/practice/naked-pair)):
  - A board is stopped where the technique applies, with the instruction "Find the Naked Pair" and a difficulty from 1 to 4.
  - **The grid is not interactive.** The player checks themselves with "Show Solution".
  - "Hint" names the technique and the digits involved, then "Show Next Tip" goes further. That is a ladder much like sumi's.
- **Takeaway for sumi:** the slide-per-sentence worked example and the "how to find it" frames are worth copying as structure. The self-checked practice is the weak point. sumi has an engine and can check answers properly.

### SudokuWiki (Andrew Stuart)

- **Getting started** ([Getting Started](https://www.sudokuwiki.org/Getting_Started)):
  - Begin by scanning by eye, then move through the tiers Basic, Tough, Diabolical and Extreme.
  - The solver's "Take Step" button reveals one deduction at a time.
- **Strategy pages** ([Intersection Removal](https://www.sudokuwiki.org/Intersection_Removal), [Hidden Candidates](https://www.sudokuwiki.org/Hidden_Candidates)):
  - Each page has a short statement of the idea and 2–3 worked examples.
  - Each example has a diagram that marks the pattern and the eliminations.
  - Each example has two links: **"Load Example"**, which opens that exact board in the solver, and "From the Start".
  - Back and next links make the pages a linear course.
- **No practice checking.** It is a reference with a stepping solver, not a course.
- **Takeaway:** every example should open as a live board. sumi can do this from the lesson file.

### Good Sudoku (Zach Gage and Jack Schlesinger)

- **Techniques and practice.** The App Store text says the game lists exactly which techniques each difficulty needs. It also says players can practise techniques individually outside puzzles, and that it keeps track of which ones they have learned ([App Store](https://apps.apple.com/us/app/good-sudoku-by-zach-gage/id1489118195)).
- **Hints read the player's board.** The hint solver runs during play and infers what the player knows from their placements and notes, then points to the next technique ([Pocket Gamer](https://www.pocketgamer.com/articles/083617/good-sudoku-aims-to-achieve-the-impossible-by-teaching-me-how-to-play-and-enjoy-sudoku/), [Tools and Toys](https://toolsandtoys.net/good-sudoku-for-ios-by-zach-gage-and-jack-schlesinger/)).
- **Busywork tools.** It removes counting busywork, such as auto-filling a cell with one candidate left, so attention goes to technique structures ([Tools and Toys](https://toolsandtoys.net/good-sudoku-for-ios-by-zach-gage-and-jack-schlesinger/)).
- **It also has pressure features.** It has daily modes that get harder through the week and **global leaderboards** ([App Store](https://apps.apple.com/us/app/good-sudoku-by-zach-gage/id1489118195)). sumi should leave these out.
- **Takeaway:** grading puzzles by technique, a technique list that tracks what is learned, and hints that respect the player's own notes. I could not find a public description of its tutorial screens in detail. That part stays unverified.

### Sudoku Academy (iPad)

- "11 techniques explained, demonstrated and practiced" ([App Store](https://apps.apple.com/us/app/-/id1622267052)).
- Its six tiers are defined by the techniques they need.
- A coach shows the next logical move and explains why.
- **A technique counts as mastered after 5 successful exercises.** This is the only explicit mastery rule I found in a Sudoku product.

### Sudoku Still: Learn Sudoku (iOS, recent)

- It has 20 lessons from scanning to X-Wing ([App Store](https://apps.apple.com/app/id6792647620)).
- **After each move it knows which technique was due.** It records whether the player spotted that technique alone or had it shown, and fills a bar per technique until the technique is solid.
- It runs offline.
- This is the nearest product to sumi's spirit. Its main idea is to count evidence from normal play, not only from drills.

### Cracking the Cryptic app

- The App Store listing does not describe a teaching structure ([App Store](https://apps.apple.com/us/app/-/id1629992934)).
- Reviews there complain that the hints jump steps that are not obvious to a novice. That is a reminder that hint text must go one small step at a time.

### Logic Wiz and other "learn Sudoku" apps

- I found no primary page describing Logic Wiz's teaching structure. I am leaving it out instead of guessing.
- Other listings are generic ("structured training", "smart hints") and add nothing ([Sudokus – Practice and Solver](https://apps.apple.com/lc/app/sudokus-practice-and-solver/id1669236229)).

### Summary of the Sudoku products

| Product | Order | Presentation | Answer check | Wrong answer | "Learned" when |
|---|---|---|---|---|---|
| sudoku.coach | Notation → singles (hidden first) → hidden pair → locked candidates → naked pair → … | Slideshow, one sentence per frame; "how to find it" frames | Practice: none (self-check via Show Solution). Campaign: whole puzzles | Hint, then next tip | Chapter items completed |
| SudokuWiki | Basic → Tough → Diabolical → Extreme | Text + 2–3 diagrams + Load Example | None | n/a | n/a |
| Good Sudoku | By difficulty tier | Short lessons; hints in play | Practice outside puzzles (details unverified) | Hint reads notes | Tracks learned techniques |
| Sudoku Academy | 11 techniques, tiered | Explain, demonstrate, practise | Exercises | Coach explains | 5 successful exercises |
| Sudoku Still | 20 lessons, scanning → X-Wing | Lessons + hints | Every move in play | Shown vs spotted | Per-technique bar fills from play |

---

## 2. Chess teaching

### Lichess Learn (beginners)

- **Structure.** Categories hold stages ([stage/list.ts](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/stage/list.ts)):
  - Categories are Chess Pieces, Fundamentals, Intermediate and Advanced.
  - Each stage is one concept, for example rook, capture or castling, and has a title, subtitle, intro text, a list of levels and a completion message.
  - Each level has a one-line **goal**, a position, target squares ("apples") and an **expected move count** ([rook stage](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/stage/rook.ts)).
  - The rook stage has six levels, from a one-move capture to multi-piece routes.
- **How success is checked.** Success comes from a **goal check** (all apples collected, or a custom success function), not by comparing against one stored move ([levelCtrl.ts](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/levelCtrl.ts)). Any route that reaches the goal is accepted. This is the pattern sumi should copy.
- **Wrong moves.** A wrong move ends the level with "Puzzle failed!" and a **Retry** button that restarts it ([runView.ts](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/run/runView.ts), [learn.xml strings](https://raw.githubusercontent.com/lichess-org/lila/master/translation/source/learn.xml)).
- **Stars.** Stars score speed in moves, measured against par ([score.ts](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/score.ts), [progressView.ts](https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/progressView.ts)):
  - Each level is worth 500 points if done within par, 300 if a little over, and 100 if well over, plus points per apple.
  - 3, 2 or 1 stars follow from the total.
  - The stage also has a rank, and a "What next?" screen points to further study.
- **For sumi:** keep the goal sentence, checking by goal and the short series of levels that get harder. Drop the stars and par. They reward efficiency, which is pressure. Drop the forced restart too. A Sudoku position does not "fail", and the player can just try again on the same board.

### Lichess Practice (intermediate)

- **Content.** It started as 18 topics in 4 sections (checkmates, tactics, pawn endings), with 4 to 24 exercises per topic that get harder ([announcement](https://lichess.org/blog/WKhcaCUAAIVBvnv5/announcing-practice)). The current sections are Checkmates, Fundamental Tactics, Advanced Tactics and Endgames ([practice](https://lichess.org/practice)).
- **How an exercise works.**
  - Each exercise has a **stated objective**, which varies: mate, a winning position in N moves, draw, or promote safely.
  - The player plays against an engine from a position designed to teach.
  - Short comments explain the idea, and the next exercise follows automatically.
  - Progress is saved for logged-in users, and a percentage shows per section.
- **For sumi:** the stated objective is the useful part, for example "Find a pointing pair and make its eliminations". It tells the player what kind of thing to look for without giving the answer.

### chess.com lessons

- **Structure** ([support](https://support.chess.com/en/articles/8609703-how-do-lessons-work-on-chess-com)):
  - Lessons are interactive and led by a Coach, mixing explanation and challenges. They replaced the older video lessons.
  - There are two ways in: a **path** through levels (New to Chess → Advanced), where lessons are locked, available or completed and courses end in "graduation cap" icons, or a **library** that can be browsed freely.
  - Basic members get one non-beginner lesson per day.
- **For sumi:** keep the idea of a path plus a library. Lock nothing. A player who already knows a technique should be able to open any lesson or drill.
- **Answers that are right but not the stored one.** chess.com's older Chess Mentor rejected correct alternative moves as an "Alternate Correct Move", and players called it mind-reading ([forum](https://www.chess.com/forum/view/chess-mentor/mind-reading-required)). Lichess puzzles keep a single expected line except when there are several mates in one, and explain why when asked ([forum](https://lichess.org/forum/lichess-feedback/error-in-new-puzzle-played-only-50-times)). This is the main argument for sumi checking the logic rather than one stored answer.

### Chessable MoveTrainer (chess.com)

- **Learn and review.** In "Learn", the player reads a comment on one move, plays the move, and repeats; at the end the whole line is played again from memory ([Matuschak notes](https://notes.andymatuschak.org/zDr94hP6bG3jJYrdYy8B5hx)).
- **Spaced review.** Review uses **8 levels** with gaps of 4 h, 1 d, 3 d, 1 w, 2 w, 1 mo, 3 mo and 6 mo ([chess.com support](https://support.chess.com/en/articles/10319322-how-does-the-spaced-repetition-scheduling-work)):
  - A correct answer moves the item up a level. A mistake resets its timing.
  - Each move has its own timer.
  - The help page says that falling behind is not a disaster, because the item just goes back to level 1.
- **Critique.** Matuschak notes that automatic marking can only check moves, so it reinforces recall of moves and cannot reinforce concepts ([notes](https://notes.andymatuschak.org/zDr94hP6bG3jJYrdYy8B5hx)).
- **For sumi:** day-based intervals per technique fit well. sumi's drills are *new* boards, so it tests recognising the pattern in a new board, not remembering one fixed position. That is better than Chessable's case. The optional "pick the pattern cells" step answers Matuschak's critique: it checks the reason as well as the move.

### Chess: what carries over and what to leave out

| Carry over | Leave out (pressure) |
|---|---|
| One stage per concept, short intro, 3–6 positions that get harder (Lichess Learn) | Stars and par-move scoring (Lichess Learn) |
| A one-line goal per position (Lichess Learn and Practice) | "Puzzle failed!" with a forced restart |
| Checking by goal, so any valid route counts (Lichess Learn) | Daily lesson limits, locked paths (chess.com) |
| Path plus open library (chess.com) | Due-count badges and review backlogs (SRS apps) |
| Interval levels per item (Chessable) | Leaderboards (Good Sudoku dailies) |
| Interleaved explanation and doing (Chessable "Learn") | Percent-complete bars that invite completionism, unless switched on |

---

## 3. Learning science that applies

- **Worked examples first, then faded guidance.**
  - Studying worked examples beats solving problems early in learning a skill ([Sweller and Cooper 1985](https://notes.andymatuschak.org/zYHdLJ7TFdpcwGtqDChMNbm), [LearnLab wiki](https://learnlab.org/research/wiki/index.php/Worked_examples)).
  - Renkl and Atkinson's *fading* removes worked steps one at a time. First a complete example, then one with the last step left for the learner, then two, until the learner solves alone ([Renkl and Atkinson 2003, Educational Psychologist 38(1)](https://asu.elsevierpure.com/en/publications/structuring-the-transition-from-example-study-to-problem-solving--2)).
  - The IES practice guide recommends alternating worked solutions with problems (moderate evidence) ([Pashler et al. 2007](https://ies.ed.gov/ncee/wwc/PracticeGuide/1)).
  - → The lesson goes See it → Finish it → Find it.
- **The expertise reversal effect.** Help that benefits novices can hurt experienced learners ([Kalyuga, Ayres, Chandler and Sweller 2003](https://ro.uow.edu.au/edupapers/136)).
  - → Let players skip a lesson straight to "Find it".
  - → Once a technique is learned, make hints for it terser.
- **Retrieval practice (the testing effect).** Being tested beat restudying on delayed tests, even though restudying made students more confident ([Roediger and Karpicke 2006](https://psychology.ecu.edu/wp-content/pv-uploads/sites/216/2019/03/Roediger-Karpicke-2006.pdf)). Dunlosky et al. rate practice testing and spaced practice the two most useful study techniques ([APS release on Dunlosky et al. 2013](https://www.psychologicalscience.org/news/releases/which-study-strategies-make-the-grade.html)).
  - → Drills, where the player finds the step, are worth more than reading a lesson again.
- **Spacing.**
  - Spaced study beats massed study, and the best gap grows with how long the learner needs to remember ([Cepeda et al. 2006](https://pubmed.ncbi.nlm.nih.gov/16719566/)).
  - The best gap is about 20–40% of a one-week retention interval and 5–10% of a one-year interval ([Cepeda et al. 2008](https://pubmed.ncbi.nlm.nih.gov/19076480/)).
  - → Gaps of days, growing over time, are fine. The exact numbers are not critical.
- **Interleaving.**
  - Mixed practice helps people learn to *tell categories apart*. Mixing paintings by different artists beat blocking them, yet 78% of participants believed blocking had worked better ([Kornell and Bjork 2008](https://www.gwern.net/docs/spaced-repetition/2008-kornell.pdf)).
  - In maths, blocked practice tells students which strategy to use before they read the problem. Interleaving makes them choose, and improved test scores at 1 and 30 days (d = 0.42 and 0.79) ([Rohrer, Dedrick and Stershic 2015](https://www.Gwern.net/doc/psychology/spaced-repetition/2014-rohrer-2.pdf)).
  - → In Sudoku the real skill is "which technique applies here?". Single-technique drills never train it. Mixed drills do.
- **Desirable difficulty.**
  - Spacing, interleaving, generating answers and testing all slow down performance during practice while improving lasting learning ([Bjork and Bjork 2011](https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/04/EBjork_RBjork_2011.pdf)).
  - The same paper warns that a difficulty becomes undesirable when the learner lacks the background to respond.
  - → Mix only techniques the player has already seen.
  - → Expect mixed drills to *feel* worse. Say once, plainly, why they are mixed.
- **Feedback** ([Shute 2008, Review of Educational Research 78(1)](https://myweb.fsu.edu/vshute/pdf/shute%202008_b.pdf), Tables 3–4):
  - Give immediate feedback when the task is new and hard for the learner, and delayed feedback for tasks that are simple for them.
  - Don't interrupt a learner who is working.
  - Avoid comparisons with other people, and be careful with overall grades. Comments alone produced gains, while grades, with or without comments, did not.
  - Use praise sparingly. It directs attention to the self.
  - Avoid **progressive hints that always end in the answer**, because they get abused ([Aleven et al. 2003](https://journals.sagepub.com/doi/abs/10.3102/00346543073003277)).
  - → Check on Enter, not on each keystroke.
  - → Feedback text names the board fact, not the player.
  - → No scores.
  - → When a hint reaches the answer, that board is recorded as "shown", not "found".
- **Rewards.** In a meta-analysis of 128 studies, expected rewards that depend on performance undermined intrinsic motivation (d = −0.28), while positive *informational* feedback raised it (d ≈ 0.3) ([Deci, Koestner and Ryan 1999](https://pure.ewha.ac.kr/en/publications/a-meta-analytic-review-of-experiments-examining-the-effects-of-ex/)).
  - → This is the evidence for "no stars or XP".
  - → "Right: those two cells hold only 4 and 7, so 4 and 7 go from the rest of row 3" is good feedback. "★★★" is not.
- **Mastery.** Cognitive Tutors judge mastery separately for each skill from every chance the student had to use it (Bayesian knowledge tracing), and move on when the estimate is high ([Corbett and Anderson 1995](https://link.springer.com/doi/10.1007/BF01099821), [Wikipedia: Cognitive tutor](https://en.wikipedia.org/wiki/Cognitive_tutor)).
  - → A per-technique "learned" flag built from evidence is well grounded. It does not need the full statistical model.
- **Explanation prompts.** Prompting learners to answer deep "why" questions has strong evidence ([Pashler et al. 2007](https://ies.ed.gov/ncee/wwc/PracticeGuide/1)).
  - → An optional "show why" step where the player picks the pattern cells.

---

## 4. How a player answers in a lesson position

### Options considered

| Input | Keyboard and terminal fit | What it proves | Verdict |
|---|---|---|---|
| Place the digit (singles) | Same keys as play | The result, and that the player found the cell | **Default for placing techniques** |
| Strike eliminations, then Enter | Cursor + digit toggles a "strike" mark, Enter submits | The result of an elimination technique | **Default for eliminating techniques** |
| Pick the pattern cells (Space toggles, Enter) | Easy with a cursor | The *reason* (the pair, the line, the box) | **Optional "show why" step**, and the main input in the faded "Finish it" positions |
| Name the technique (menu 1–7) | Easy | Telling techniques apart | **Optional, in mixed drills only.** Never required, since it feels like a quiz |
| Free text or coordinates ("r3c5") | Fiddly on a phone keyboard | Same as the above | No. Show coordinates as labels only |

### Recommendation

- **The answer is the move.** Use the normal play controls, so learning transfers straight to play.
  - For placing techniques (full house, hidden single, naked single), the move is one digit in one cell.
  - For eliminating techniques (pointing, claiming, naked pair, hidden pair), it is a set of candidate strikes followed by Enter.
  - In a terminal this is cursor keys or hjkl, digits, Space and Enter. All of it works in a small Termux window with no mouse.
- **"Show why" is optional and off by default.** After a correct move, `w` lets the player highlight the pattern cells. This checks the concept as well as the move ([Matuschak](https://notes.andymatuschak.org/zDr94hP6bG3jJYrdYy8B5hx)), and it is the prompt the evidence on explanation supports.
- **Boards in lessons show full candidates.** Pairs need cell candidates ([sudoku.coach naked pair lesson](https://sudoku.coach/en/learn/naked-pair)). Lesson positions should carry the notes the technique needs, so that writing in notes is not part of the exercise. The pointing, claiming and hidden pair lessons can also be shown with Snyder box marks, since those techniques read straight off box-corner marks. The naked pair lesson needs cell candidates.

### Checking rules

Check the logic, not one stored answer.

1. **Find every instance up front.** Before the answer, the engine lists every instance of the target technique on the board, with each instance's pattern and its eliminations or placement.
2. **Correct.** The submitted action equals one instance's full result, or is a subset of the union of all instances' results with every struck candidate justified.
   - If it is a *subset* of one instance, accept it and show the rest: "Right. The same pair also removes 7 from r3c9."
3. **Valid, but a different technique.** On lesson boards this should not happen, because the boards are checked at build time (rule 7). On drill boards it can, if the player uses a harder technique:
   - Accept the move.
   - Then say which technique it was and that an easier one, the one being drilled, was available. Offer to show it.
   - This is the only gentle redirect. It never counts against the player.
4. **The digit is right in the solution, but the board does not prove it yet.**
   - Say so: "4 is the answer there, but nothing on the board shows it yet."
   - Undo it, and do not count it.
   - This is the point where checking against the solution would mislead.
5. **Wrong.** Name the board fact that rules it out, worked out by the engine. For example: "r2c4 can't be 6: there's a 6 in column 4", or "7 is still possible in r5c1, so the box's 7s aren't confined to row 5".
   - Then let the player try again on the same board.
   - After a second miss, offer the hint ladder starting at the area rung. Never "failed", never a restart.
   - Keep the error text to the first conflict. Shute advises against long error diagnoses.
6. **Several valid steps.** All are accepted (rule 2). The goal line in the lesson file states the *kind* of step ("Make the eliminations from a pointing pair"), never "the" step, as Lichess Learn does with goals.
7. **Lesson files are checked by the build.** A test loads every `lessons/*.toml` and checks three things:
   - Each "Find it" board has at least one instance of the target technique.
   - It has no instance of an easier technique, in sumi's technique order.
   - Any `expect` field in the file matches one of the instances the engine computes.
   - This replaces "expected steps" as the source of truth. The file may name *the instance to show* in See it or Finish it, and the checker accepts any instance.

### Proposed `lessons/*.toml` shape (sketch)

```toml
id = "naked-pair"
technique = "naked_pair"
needs = ["naked-single"]           # recommended prerequisite, not a lock
notes = "full"                     # full | snyder | none, for the lesson boards

[[see]]                            # worked example, one frame per step
board = "…81 chars…"
frames = [
  { text = "…", show = { cells = ["r3c2","r3c7"] } },
  { text = "…", show = { strike = ["r3c4:4","r3c4:7"] } },
]

[[finish]]                         # faded: sumi shows part, player does the rest
board = "…"
given = "pattern"                  # or "result"
goal  = "…one line…"

[[find]]                           # 3 boards, no help
board = "…"
goal  = "…one line…"
```

---

## 5. Picking and pacing drills without streaks or scores

### Selection

- **What a drill board is.** Each drill board comes from a generated puzzle, stopped just before the drilled technique is the easiest step. This is the plan's idea, and it matches sudoku.coach practice.
  - Grade with sumi's own grader, so "easiest" means sumi's technique order.
  - Prefer boards where the technique has exactly one instance at the start, then mix in boards with several.
- **One record per technique:** `last_seen`, `interval_days`, `found` (unaided finds), `found_days` (distinct days with a find) and `shown` (the hint reached the answer).
- **Intervals.** Use a Leitner-style ladder of **1, 3, 7, 21, 60 days**.
  - An unaided find moves the technique up one step.
  - A "shown" result or a second miss moves it down one step, not back to the start. A reset to level 1 like Chessable's feels like a punishment.
  - Day-based gaps sit close to the 10–40% rule of thumb for retention over weeks to months ([Cepeda et al. 2008](https://pubmed.ncbi.nlm.nih.gov/19076480/)). Exact values matter little.
- **A session** is about **5 boards** that the player starts. It never starts itself and has no time limit.
  - Pick boards from techniques that are due, most overdue first.
  - Once two or more techniques have been seen, **mix them**, but don't announce which technique each board needs ([Rohrer et al.](https://www.Gwern.net/doc/psychology/spaced-repetition/2014-rohrer-2.pdf), [Kornell and Bjork](https://www.gwern.net/docs/spaced-repetition/2008-kornell.pdf)).
  - Add at most one board from a recently taught technique.
- **Single-technique drills stay available.** "Drill: naked pair" is for a player who wants focused practice right after a lesson. The default "Practice" choice is the mixed, spaced one.

### Pressure the scheduler must not create

- **No due counts.** No badges and no "12 reviews waiting".
- **No backlog.** After a month away, a session is still about 5 boards. Older due items simply wait.
- **No streaks, daily goals or reminders.**
- **No session score.** At the end of a session, show one quiet line per board: which technique it was and whether the player found it or it was shown. That is information, not a grade ([Shute 2008](https://myweb.fsu.edu/vshute/pdf/shute%202008_b.pdf)). It can be switched off.

### When a technique counts as learned

- **Rule:** **3 unaided finds on at least 2 different days**, the latest one unaided. Sudoku Academy uses 5 successes ([App Store](https://apps.apple.com/us/app/-/id1622267052)). Spreading the finds over days stops the flag from coming from one sitting of blocked practice.
- **Finds in normal games count.** If a placement or elimination in a normal game is best explained by technique T, and no hint was used for it, that is an unaided find of T. Sudoku Still uses this idea ([App Store](https://apps.apple.com/app/id6792647620)).
  - It makes drills optional for people who learn by playing.
  - It needs a cheap "what technique explains this move" check after each move, which the hint engine mostly has already.
- **Display:** the flag shows as a plain "learned" mark in the technique list (Good Sudoku keeps a similar list). No bars and no percentages by default.

### When to suggest the next lesson

- **On the Learn screen,** suggest the next lesson in order when every technique before it is learned.
- **On the drill summary,** say so when the last two sessions had no "shown" boards.
- **When normal play reaches an unlearned technique.** This is the strongest trigger (section 6).
- **Always as an offer.** Never a gate, never a popup in the middle of a game.

---

## 6. Connecting Learn to normal play and the hint ladder

- **A hint names an unlearned technique.** When rung 2 names a technique the player has not learned, add one quiet line under it: "New technique. `L` opens its lesson."
  - The lesson opens in a panel or a separate screen, and **returns to the same game position**.
  - The live board then becomes the player's first real "Find it", which is the best transfer task there is.
- **Hints that adapt to what the player knows,** following the expertise reversal effect ([Kalyuga et al.](https://ro.uow.edu.au/edupapers/136)):
  - For an *unlearned* technique, rung 2 adds a single-sentence definition.
  - For a *learned* one, rung 2 is just the name.
  - Rung 5 (the result) records the step as "shown" for that technique. It is not a penalty, just no evidence.
- **Hints that respect the player's notes,** as Good Sudoku's do ([Pocket Gamer](https://www.pocketgamer.com/articles/083617/good-sudoku-aims-to-achieve-the-impossible-by-teaching-me-how-to-play-and-enjoy-sudoku/)). If the player's notes are empty or Snyder-only, and the next step needs cell candidates, the hint says that, rather than naming a pair the player cannot see. The research report already proposes something similar for empty notes (research/report.md §3).
- **Puzzle choice by technique.** "New puzzle" can offer "uses only techniques you've learned" and "includes one you're learning". This is Good Sudoku's grading by technique, applied to one player.
- **Explicitly not added:** post-game reviews, "you could have used X here" notices after a game, technique counts at the end of a game, or nudges to come back. These clash with sumi's calm principles and its no-review rule.

---

## 7. Concrete proposal

### Learn screen

- A list of the 20 techniques in sumi's order, grouped Easy, Medium and so on.
- Each row shows: lesson available or not, *seen* (lesson done), *learned* (the evidence rule). There are no locks.
- Two actions: **Lesson** (v1: the 7 Easy and Medium techniques) and **Drill this** (all 20).
- A top action, **Practice**, runs the mixed, spaced session.
- Everything here can be hidden in settings, and Learn as a whole can be switched off.

### Lesson flow

About 5 minutes, and it can be left and resumed.

1. **See it.** 4–10 frames, one sentence each, drawn on the board. Space moves forward, Backspace moves back, Esc leaves.
   - At least one frame shows how to *spot* the pattern, not only what it is (sudoku.coach does this well).
   - The last frame says which notes the technique needs.
2. **Finish it** (1–2 boards).
   - In the first, sumi highlights the pattern and the player enters the result.
   - In the second, sumi shows the result and the player picks the pattern cells.
   - Both are faded steps ([Renkl and Atkinson](https://asu.elsevierpure.com/en/publications/structuring-the-transition-from-example-study-to-problem-solving--2)).
3. **Find it** (3 boards, getting harder, the last with a decoy near-pattern).
   - Feedback comes immediately on Enter, using the checking rules in section 4. After a second miss, the hint ladder is offered.
   - A player can press `s` on the first screen to skip straight here.
4. **Close.** One line: what the technique does, and that it will turn up in Practice. No stars.

Each lesson has 3–5 positions in all, which keeps the plan's number.

### Drills and Practice

- As in section 5: boards from the grader stopped just before the technique, 1/3/7/21/60-day steps, sessions of about 5 boards, mixed once two techniques are seen, and the "learned" rule counting finds from normal play.

### Leave out, and why

| Leave out | Why |
|---|---|
| Stars, par, points, XP | Performance-dependent expected rewards undermine intrinsic motivation ([Deci et al.](https://pure.ewha.ac.kr/en/publications/a-meta-analytic-review-of-experiments-examining-the-effects-of-ex/)). Grades crowd out comments ([Shute](https://myweb.fsu.edu/vshute/pdf/shute%202008_b.pdf)). Against "calm by default" |
| "Failed" and forced restarts | A wrong Sudoku step can be put right on the same board. Restarting adds cost and stress, not information |
| Streaks, daily goals, reminders, due-count badges, review backlogs | Pressure, and sumi's principles forbid them. Spacing works without visible counters |
| Locked lessons | Expertise reversal: people who already know a technique should skip ([Kalyuga et al.](https://ro.uow.edu.au/edupapers/136)) |
| Requiring the technique's name as the answer | Quiz-like, and it adds little over the move. Offered as an option in mixed drills |
| Checking against one stored step | "Mind-reading" complaints ([chess.com forum](https://www.chess.com/forum/view/chess-mentor/mind-reading-required)). Several valid steps are normal in Sudoku |
| Timers in drills | sumi's principles, and nothing in the evidence above needs speed |
| Percent-complete bars (default) | Pushes completionism. Available as a setting at most |

---

## Open questions

1. **Hidden pair before naked pair?** The research report orders naked pair first (research/report.md §2). sudoku.coach teaches hidden pair first and builds naked pair from it ([learn index](https://sudoku.coach/en/learn)). With Snyder box marks as sumi's note layer, hidden pairs are visible without full candidates. This choice affects lesson prerequisites.
2. **Notes in lessons and drills:** auto-filled full candidates, Snyder marks only, or whatever the player has set? Auto-fill keeps the exercise on the technique. Player notes make it like real play. Possibly decide per technique.
3. **"Learned" threshold:** is 3 finds over 2 days right? I found no evidence for a particular number. Sudoku Academy uses 5; Cognitive Tutors use a probability cutoff. Start with 3/2 and adjust after testing it.
4. **Finds in normal play:** is "best explained by technique T" cheap and reliable enough after each move? If the player's move also follows from an easier technique, it should count for the easier one.
5. **Should the end-of-session line exist at all,** or should drill results be visible only in the technique list?
6. **Lesson text on a phone-sized terminal:** one sentence per frame suits narrow windows. Should the frame text sit below the board, or in a toggled side panel on wide terminals?
7. **Drill boards for rare techniques** (for example the later 13 of the 20) may be slow to generate. Pre-generate a bank, or generate in the background?

---

## Sources

Sudoku products
- sudoku.coach campaign: https://sudoku.coach/en/campaign
- sudoku.coach learn index: https://sudoku.coach/en/learn
- sudoku.coach hidden single lesson: https://sudoku.coach/en/learn/hidden-single
- sudoku.coach naked pair lesson: https://sudoku.coach/en/learn/naked-pair
- sudoku.coach practice index: https://sudoku.coach/en/practice
- sudoku.coach naked pair practice: https://sudoku.coach/en/practice/naked-pair
- SudokuWiki Getting Started: https://www.sudokuwiki.org/Getting_Started
- SudokuWiki Intersection Removal: https://www.sudokuwiki.org/Intersection_Removal
- SudokuWiki Hidden Candidates: https://www.sudokuwiki.org/Hidden_Candidates
- Good Sudoku, App Store: https://apps.apple.com/us/app/good-sudoku-by-zach-gage/id1489118195
- Good Sudoku, Pocket Gamer: https://www.pocketgamer.com/articles/083617/good-sudoku-aims-to-achieve-the-impossible-by-teaching-me-how-to-play-and-enjoy-sudoku/
- Good Sudoku, Tools and Toys: https://toolsandtoys.net/good-sudoku-for-ios-by-zach-gage-and-jack-schlesinger/
- Good Sudoku, iMore: https://www.imore.com/zach-gage-releases-good-sudoku-reimagined-and-fun-take-classic
- Sudoku Academy, App Store: https://apps.apple.com/us/app/-/id1622267052
- Sudoku Still, App Store: https://apps.apple.com/app/id6792647620
- Cracking the Cryptic app, App Store: https://apps.apple.com/us/app/-/id1629992934
- Sudokus – Practice and Solver, App Store: https://apps.apple.com/lc/app/sudokus-practice-and-solver/id1669236229

Chess
- Lichess Learn stage list: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/stage/list.ts
- Lichess Learn rook stage: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/stage/rook.ts
- Lichess Learn scoring: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/score.ts
- Lichess Learn stars: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/progressView.ts
- Lichess Learn level control: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/levelCtrl.ts
- Lichess Learn failure view: https://raw.githubusercontent.com/lichess-org/lila/master/ui/learn/src/run/runView.ts
- Lichess Learn strings: https://raw.githubusercontent.com/lichess-org/lila/master/translation/source/learn.xml
- Lichess Practice announcement: https://lichess.org/blog/WKhcaCUAAIVBvnv5/announcing-practice
- Lichess Practice: https://lichess.org/practice
- Lichess puzzle alternative-move thread: https://lichess.org/forum/lichess-feedback/error-in-new-puzzle-played-only-50-times
- chess.com lessons help: https://support.chess.com/en/articles/8609703-how-do-lessons-work-on-chess-com
- chess.com Chess Mentor "mind reading" thread: https://www.chess.com/forum/view/chess-mentor/mind-reading-required
- MoveTrainer spaced repetition (chess.com support): https://support.chess.com/en/articles/10319322-how-does-the-spaced-repetition-scheduling-work
- Andy Matuschak, Chessable MoveTrainer: https://notes.andymatuschak.org/zDr94hP6bG3jJYrdYy8B5hx

Learning science
- Pashler et al. (2007), IES practice guide: https://ies.ed.gov/ncee/wwc/PracticeGuide/1
- Sweller and Cooper (1985), via notes and LearnLab: https://notes.andymatuschak.org/zYHdLJ7TFdpcwGtqDChMNbm and https://learnlab.org/research/wiki/index.php/Worked_examples
- Renkl and Atkinson (2003): https://asu.elsevierpure.com/en/publications/structuring-the-transition-from-example-study-to-problem-solving--2 (the page shows a 2016 reprint DOI. The original is Educational Psychologist 38(1), 15–22, 2003)
- Kalyuga, Ayres, Chandler and Sweller (2003), expertise reversal: https://ro.uow.edu.au/edupapers/136
- Roediger and Karpicke (2006): https://psychology.ecu.edu/wp-content/pv-uploads/sites/216/2019/03/Roediger-Karpicke-2006.pdf
- Dunlosky et al. (2013), APS summary: https://www.psychologicalscience.org/news/releases/which-study-strategies-make-the-grade.html
- Cepeda et al. (2006): https://pubmed.ncbi.nlm.nih.gov/16719566/
- Cepeda et al. (2008): https://pubmed.ncbi.nlm.nih.gov/19076480/
- Kornell and Bjork (2008): https://www.gwern.net/docs/spaced-repetition/2008-kornell.pdf
- Rohrer, Dedrick and Stershic (2015): https://www.Gwern.net/doc/psychology/spaced-repetition/2014-rohrer-2.pdf
- Bjork and Bjork (2011): https://bjorklab.psych.ucla.edu/wp-content/uploads/sites/13/2016/04/EBjork_RBjork_2011.pdf
- Shute (2008), Focus on Formative Feedback: https://myweb.fsu.edu/vshute/pdf/shute%202008_b.pdf
- Aleven et al. (2003), help seeking: https://journals.sagepub.com/doi/abs/10.3102/00346543073003277
- Deci, Koestner and Ryan (1999): https://pure.ewha.ac.kr/en/publications/a-meta-analytic-review-of-experiments-examining-the-effects-of-ex/
- Corbett and Anderson (1995), knowledge tracing: https://link.springer.com/doi/10.1007/BF01099821 and https://en.wikipedia.org/wiki/Cognitive_tutor
