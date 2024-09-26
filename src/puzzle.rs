use std::fmt::Debug;
use std::hash::Hash;

pub use square::*;

mod square;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeKey(pub usize);

// point just to either side of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointKey(pub usize);

/// constraints for a well-defined puzzle:
/// - define `nth_point_on_piece(n,x)` as `if n == 0 {x} else {nth_point_on_piece(n-1,next_point_on_piece(x))`
/// - `point_piece(arbitary_point_on_piece(x)) == x` for all `x`
/// - `point_piece(nth_point_on_piece(n, arbitary_point_on_piece(x))) == x` for all `x`, `n`
/// - `nth_point_on_piece(n, x) == x` for all `x` for some `n`
/// - `point_piece(x) == point_piece(y)` implies `nth_point_on_piece(n, x) == y` for all `x`, `y` for some `n`
/// - `point_edge(arbitrary_point_on_edge(x)) == x` for all `x`
/// - `point_edge(other_point_on_edge(arbitrary_point_on_edge(x))) == x` for all `x`
/// - `other_point_on_edge(other_point_on_edge(x)) == x` for all `x`
/// - `point_edge(x) == point_edge(y)` implies `x == y || other_point_on_edge(x) == y` for all `x`, `y`
/// - `can_exchange(x,x)` for all `x`
/// - `can_exchange(x,y) && can_exchange(y,z)` implies `can_exchange(x,z)` for all `x`, `y`, `z`
/// - `can_exchange(x,y)` implies `can_exchange(next_point_on_piece(x),next_point_on_piece(y))` for all `x`, `y`
/// - if `x` and `y` are exchanged then `next_point_on_piece(x)` and `next_point_on_piece(y)` are exchanged (slightly stronger than above but not automatically verifiable)
///
/// in theory this could be reduced to `num_points`, `next_point_on_piece`, `other_point_on_edge`, and `can_exchange`.
/// but puzzle implementations can more efficiently deal with the `Piece` and `Edge` equivalence classes since they are most likely used internally anyway
pub trait Puzzle {
    fn num_pieces(&self) -> usize;
    fn num_edges(&self) -> usize;
    fn num_points(&self) -> usize {
        2 * self.num_edges()
    }

    fn arbitrary_point_on_piece(&self, piece: PieceKey) -> PointKey;
    fn next_point_on_piece(&self, point: PointKey) -> PointKey;
    fn point_piece(&self, point: PointKey) -> PieceKey;

    fn arbitrary_point_on_edge(&self, edge: EdgeKey) -> PointKey;
    fn other_point_on_edge(&self, point: PointKey) -> PointKey;
    fn point_edge(&self, point: PointKey) -> EdgeKey;

    fn can_exchange(&self, point_a: PointKey, point_b: PointKey) -> bool;

    fn format_point(&self, point: PointKey) -> String;
    fn format_edge(&self, edge: EdgeKey) -> String;
}

pub fn puzzle_pieces(puzzle: &impl Puzzle) -> impl Iterator<Item = PieceKey> {
    (0..puzzle.num_pieces()).map(PieceKey)
}

pub fn puzzle_edges(puzzle: &impl Puzzle) -> impl Iterator<Item = EdgeKey> {
    (0..puzzle.num_edges()).map(EdgeKey)
}
pub fn puzzle_edge_pairs(puzzle: &impl Puzzle) -> impl Iterator<Item = (EdgeKey, EdgeKey)> + '_ {
    (0..puzzle.num_edges()).flat_map(move |a| (0..a).map(move |b| (EdgeKey(a), EdgeKey(b))))
}

pub fn puzzle_points(puzzle: &impl Puzzle) -> impl Iterator<Item = PointKey> {
    (0..puzzle.num_points()).map(PointKey)
}
pub fn puzzle_point_pairs(puzzle: &impl Puzzle) -> impl Iterator<Item = (PointKey, PointKey)> + '_ {
    (0..puzzle.num_points()).flat_map(move |a| (0..a).map(move |b| (PointKey(a), PointKey(b))))
}
pub fn puzzle_exchange_points(
    puzzle: &impl Puzzle,
    point: PointKey,
) -> impl Iterator<Item = PointKey> + '_ {
    (0..puzzle.num_points())
        .map(PointKey)
        .filter(move |&other| puzzle.can_exchange(point, other))
}

// pub fn puzzle_points_on_piece(
//     puzzle: &impl Puzzle,
//     piece: PieceKey,
// ) -> impl Iterator<Item = PointKey> + '_ {
//     let start_point = puzzle.arbitrary_point_on_piece(piece);
//     std::iter::successors(Some(start_point), move |&point| {
//         if point == start_point {
//             None
//         } else {
//             Some(puzzle.next_point_on_piece(point))
//         }
//     })
// }
