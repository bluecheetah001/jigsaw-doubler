pub mod square;
pub use square::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceKey {
    pub orbit: usize,
    pub index: usize,
}
impl PieceKey {
    pub fn new(orbit: usize, index: usize) -> Self {
        Self { orbit, index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PieceKeyEdge {
    pub piece: PieceKey,
    pub edge: usize,
}
impl PieceKeyEdge {
    pub fn new(orbit: usize, index: usize, edge: usize) -> Self {
        Self {
            piece: PieceKey::new(orbit, index),
            edge,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeKey {
    pub orbit: usize,
    pub index: usize,
}
impl EdgeKey {
    pub fn new(orbit: usize, index: usize) -> Self {
        Self { orbit, index }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceOrbitInfo {
    /// how many pieces are in this orbit
    pub len: usize,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeOrbitInfo {
    /// how many edges are in this orbit
    pub len: usize,
}

pub trait Puzzle {
    fn num_piece_orbits(&self) -> usize;
    fn piece_orbit(&self, orbit: usize) -> PieceOrbitInfo;

    fn num_edge_orbits(&self) -> usize;
    fn edge_orbit(&self, orbit: usize) -> EdgeOrbitInfo;

    fn piece_edge(&self, piece: PieceKey, edge: usize) -> EdgeKey;
    fn piece_neighbor(&self, piece: PieceKey, edge: usize) -> PieceKeyEdge;

    fn edge_pieces(&self, edge: EdgeKey) -> [PieceKeyEdge; 2];
}
