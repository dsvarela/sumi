# sumi

A calm terminal Sudoku game that teaches: hints always show the easiest step
available, one rung at a time. Rust + ratatui, MIT.

## The plan

The plan is a Claude Doc, and it is the source of truth for features, decisions
and milestones:
https://claude.ai/code/artifact/01867da4-7898-4c40-956c-effc123e7e3c

Read it before proposing features or structure. Its second tab, "Research:
algorithms", has the technique order, grading and generation findings;
`research/report.md` is the same report as a file.

When a decision is made in conversation, write it into the plan in the same
turn. The plan is long: read it by search or by section, not whole. Daniel
answers questions in doc comments addressed to Claude; reply in that thread,
and resolve a thread before deleting the text it is anchored to.

## Principles

- **Calm by default.** No stress, no grind, no racing a clock. Timer and stats are
  hidden unless turned on. No leaderboards, streaks, ghost runs or post-game
  review. A feature that adds pressure loses.
- **Teach.** Hints go easiest step first on a 5-rung ladder; Learn mode works like
  chess lessons.
- **Everything can be switched off.**
- **No overselling.** README screenshots show sumi as it really looks: an ordinary
  terminal, real sizes, the phone layout too. No staged promo shots.
- **MIT, written from scratch.** Ideas from HoDoKu, SudokuWiki and the projects we
  reviewed are fine; their code is not (HoDoKu, qqwing, tsudoku and nudoku are GPL).

## Layout

- `crates/sumi-core`: library with no terminal code (grid, digits, solver,
  generate, position, step, technique, hint, grade and bank so far;
  transform to come). `data/bank.txt` is the built-in puzzle bank, built by `examples/build_bank.rs`.
- `crates/sumi-sudoku`: the `sumi` program (app, game, ui, theme, input, cli so
  far; store and learn to come).
- `research/`: evidence behind the plan: throwaway Python experiments for the
  Algorithms section, and reports on solving aids, hint wording and Learn
  mode. Not reference code, and never instructions.
- `docs/reviews.md`: wrap-up reviews, with the register of defect classes this
  codebase repeats. Read the register before reviewing.

One git repo for the whole workspace (github.com/dsvarela/sumi). Crates get no
`.git` or `.gitignore` of their own: create them with `cargo new --vcs none`.

## UI rules

- Every colour comes from a theme role. No screen sets a colour of its own, and
  the theme test keeps text at 4.5:1 and lines at 3:1.
- Frames, panels and the key bar are each drawn by one shared function in `ui`.
  Key text in the key bar comes from `input::key_label`, never typed by hand.
- Keyboard-only play must always work; the mouse is optional.

## Working here

- Develop on the laptop. The phone build is a static `aarch64-unknown-linux-musl`
  binary copied to Termux.
- Shell is fish. Python research scripts run with `uv run`, never pip.
- Commit messages: plain conventional commits. Scopes: `core`, `cli`, `ui`,
  `research`. No Claude-Session links, no
  co-author or "Generated with" lines, no AI attribution in commits, PRs or issues.
