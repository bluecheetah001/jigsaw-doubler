# Perfect Doubled Jigsaw Search

Response to Matt Parker's terrible Python code https://youtu.be/b5nElEbbnfU?si=13-o5x8be2MVYovR&t=760

## How to use
- Modify `src/main.rs`'s `make_puzzle` function to specify the puzzle size to search for
- run `cargo run --release`

## How it works
Build on a [Boolean Satisfiability](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem) Solver

A core notion for efficiently encoding constraints is [Tsytin Transformations](https://en.wikipedia.org/wiki/Tseytin_transformation)
where intermediate computations are encoded in variables.

This uses the following terms to refer to parts of a jigsaw puzzle:
- Piece - an individual jigsaw piece
- Edge - the seam between two jigsaw pieces when connected
- Point - a Piece Edge pair, you can think of this as a point just on either side of an Edge

This encodes the problem into SAT with the following logic:
- a variable for each possible destination for a given Piece
  - the upper left corner of a puzzle is assumed to not move to reduce the search space, multiply number of solutions below by 4 if you want to ignore this symmetry
- a constrait that each source Piece only has a single destination Piece
- a constraint that each destination Piece only has a single source Piece
- a variable computing for each pair of Points if they are adjacent after moving Pieces
- a constraint that Points that start adjacent don't end adjacent
- a variable computing for each pair of Edges if they match due to Points on one edge ending adjacent to Points on the other Edge
- a constraint that each Edge matches exactly one other Edge

## Results on my machine so far
- 2x2 - Found only solution in 1ms
- 2x4 - Found only solution in 2ms
- 3x3 - Found all 7 solutions in 5ms
- 2x6 - Found all 23 solutions in 20ms
- 3x5 - Found all 45 solutions in 84ms
- 4x4 - Found all 63 solutions in 166ms
- 2x8 - Found all 363 solutions in 754ms
- 3x7 - Found all 887 solutions in 8s
- 4x6 - Found > 2000 solutions in 1m
- 5x5 - Found > 4000 solutions in 1m
