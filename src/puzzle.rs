pub mod square;
pub use square::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceKey(pub usize);
impl PieceKey {
    pub fn iter(len: usize) -> impl Iterator<Item = PieceKey> {
        (0..len).map(PieceKey)
    }
    pub fn edges(self, len: usize) -> impl Iterator<Item = PieceKeyEdge> {
        (0..len).map(move |e| PieceKeyEdge::new(self, e))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceOrbitKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeKey(pub usize);
impl EdgeKey {
    pub fn iter(len: usize) -> impl Iterator<Item = EdgeKey> {
        (0..len).map(EdgeKey)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeOrbitKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceKeyEdge {
    pub piece: PieceKey,
    pub edge: usize,
}
impl PieceKeyEdge {
    pub fn new(piece: PieceKey, edge: usize) -> Self {
        Self { piece, edge }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceOrbitInfo {
    /// how many ways a piece can be put into a single location
    ///
    /// must be >= 1
    pub rotations: usize,
    /// how many edges a piece has
    ///
    /// must be a multiple of rotations
    pub edges: usize,

    /// e' = (e + r*edge_increment_per_rotation) % edges
    /// gets e' by rotating e r times
    ///
    /// conceptually is `edges / rotations` but could be something else if that makes the puzzle's implementation simpler
    pub edge_increment_per_rotation: usize,
}

pub trait Puzzle {
    fn num_pieces(&self) -> usize;
    fn piece_orbit(&self, piece: PieceKey) -> PieceOrbitKey;
    fn piece_orbit_info(&self, orbit: PieceOrbitKey) -> PieceOrbitInfo;

    fn num_edges(&self) -> usize;
    fn edge_orbit(&self, edge: EdgeKey) -> EdgeOrbitKey;

    fn piece_edge(&self, piece_edge: PieceKeyEdge) -> EdgeKey;
    fn piece_neighbor(&self, piece_edge: PieceKeyEdge) -> PieceKeyEdge;

    fn edge_pieces(&self, edge: EdgeKey) -> [PieceKeyEdge; 2];
}
