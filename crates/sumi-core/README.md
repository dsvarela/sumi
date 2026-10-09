# sumi-core

The Sudoku engine behind [sumi](https://github.com/dsvarela/sumi), a
terminal Sudoku game that teaches. It has no terminal code. It solves,
generates and grades puzzles. It also finds the easiest next step as a
hint in five rungs, from how hard the step is to the digit it places.

Grading solves a puzzle the way a person would. It always takes the
easiest step among 20 techniques, from full house through fish and wings
to simple colouring. The level is that of the hardest technique it needed: Easy,
Medium, Hard or Expert, or Beyond when the techniques run out.

```rust
use sumi_core::grade::grade;
use sumi_core::grid::Grid;
use sumi_core::hint::{self, Hint};
use sumi_core::puzzle::Puzzle;

let clues: Grid = "1...34.....6....17.8.2..4..5.4...97...8.5.1...61...5.3..5..7.4.81....6.....16...5"
    .parse()
    .unwrap();

// A puzzle is built only from clues with exactly one solution.
let puzzle = Puzzle::new(clues).unwrap();

let grade = grade(&clues).unwrap();
println!("{:?}, hardest step: {}", grade.level, grade.hardest.unwrap().name());

// The hint for the board as it stands, with no candidates written yet.
let notes = [None; 81];
let hint = hint::find(&clues, &puzzle, &notes);
assert!(matches!(hint, Hint::Step(_)));
for rung in 1..=hint.rungs() {
    println!("{}", hint.rung(rung).text);
}
```

Licensed under the MIT licence.
