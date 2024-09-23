use super::{EdgeKey, EdgeOrbitKey, PieceKey, PieceKeyEdge, PieceOrbitInfo, PieceOrbitKey, Puzzle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PieceLoc {
    row: usize,
    col: usize,
}
impl PieceLoc {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
    fn up_edge(&self) -> EdgeLoc {
        EdgeLoc {
            loc: PieceLoc {
                row: self.row - 1,
                col: self.col,
            },
            side: EdgeSide::Down,
        }
    }
    fn right_edge(&self) -> EdgeLoc {
        EdgeLoc {
            loc: *self,
            side: EdgeSide::Right,
        }
    }
    fn down_edge(&self) -> EdgeLoc {
        EdgeLoc {
            loc: *self,
            side: EdgeSide::Down,
        }
    }
    fn left_edge(&self) -> EdgeLoc {
        EdgeLoc {
            loc: PieceLoc {
                row: self.row,
                col: self.col - 1,
            },
            side: EdgeSide::Right,
        }
    }

    fn up_piece(&self) -> PieceLoc {
        PieceLoc {
            row: self.row - 1,
            col: self.col,
        }
    }
    fn right_piece(&self) -> PieceLoc {
        PieceLoc {
            row: self.row,
            col: self.col + 1,
        }
    }
    fn down_piece(&self) -> PieceLoc {
        PieceLoc {
            row: self.row + 1,
            col: self.col,
        }
    }
    fn left_piece(&self) -> PieceLoc {
        PieceLoc {
            row: self.row,
            col: self.col - 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EdgeLoc {
    loc: PieceLoc,
    side: EdgeSide,
}
impl EdgeLoc {
    fn new_right(row: usize, col: usize) -> Self {
        Self {
            loc: PieceLoc::new(row, col),
            side: EdgeSide::Right,
        }
    }
    fn new_down(row: usize, col: usize) -> Self {
        Self {
            loc: PieceLoc::new(row, col),
            side: EdgeSide::Down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeSide {
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SquarePuzzle {
    rows: usize,
    cols: usize,
}
impl SquarePuzzle {
    pub fn new(rows: usize, cols: usize) -> Self {
        // rows or cols == 1 changes orbits, so is disallowed for simplicity
        assert!(rows >= 2);
        assert!(cols >= 2);
        Self { rows, cols }
    }
    fn num_vert_edges(&self) -> usize {
        self.rows * (self.cols - 1)
    }
    fn num_horz_edges(&self) -> usize {
        (self.rows - 1) * self.cols
    }

    fn last_col(&self) -> usize {
        self.cols - 1
    }
    fn last_row(&self) -> usize {
        self.rows - 1
    }

    fn implicit_rotation(&self, piece: PieceLoc) -> usize {
        match (
            piece.row == 0,
            piece.row == self.last_row(),
            piece.col == 0,
            piece.col == self.last_col(),
        ) {
            (false, false, false, false) => 0, // center
            (true, false, false, false) => 1,  // up
            (false, true, false, false) => 3,  // down
            (false, false, true, false) => 0,  // left
            (false, false, false, true) => 2,  // right
            (true, false, true, false) => 1,   // up left
            (true, false, false, true) => 2,   // up right
            (false, true, true, false) => 0,   // down left
            (false, true, false, true) => 3,   // down right
            _ => unreachable!(),
        }
    }

    fn piece_loc(&self, piece: PieceKey) -> PieceLoc {
        assert!(piece.0 < self.num_pieces());
        let row = piece.0 / self.cols;
        let col = piece.0 % self.cols;
        PieceLoc::new(row, col)
    }
    fn piece_key_edge(&self, piece: PieceLoc, edge: usize) -> PieceKeyEdge {
        let edge = (edge + 4 - self.implicit_rotation(piece)) % 4;
        let piece = PieceKey(self.cols * piece.row + piece.col);
        PieceKeyEdge::new(piece, edge)
    }
    fn edge_loc(&self, edge: EdgeKey) -> EdgeLoc {
        assert!(edge.0 < self.num_edges());
        if edge.0 < self.num_vert_edges() {
            let row = edge.0 / (self.cols - 1);
            let col = edge.0 % (self.cols - 1);
            EdgeLoc::new_right(row, col)
        } else {
            let row = (edge.0 - self.num_vert_edges()) / self.cols;
            let col = (edge.0 - self.num_vert_edges()) % self.cols;
            EdgeLoc::new_down(row, col)
        }
    }
    fn edge_key(&self, edge: EdgeLoc) -> EdgeKey {
        match edge.side {
            EdgeSide::Right => EdgeKey(edge.loc.row * (self.cols - 1) + edge.loc.col),
            EdgeSide::Down => {
                EdgeKey(self.num_vert_edges() + edge.loc.row * self.cols + edge.loc.col)
            }
        }
    }
}
impl Puzzle for SquarePuzzle {
    fn num_pieces(&self) -> usize {
        self.rows * self.cols
    }

    fn piece_orbit(&self, piece: PieceKey) -> PieceOrbitKey {
        if piece == PieceKey(0) {
            // ul corner is given a dedicated orbit to break rotational symmetry of the puzzle
            return PieceOrbitKey(0);
        }
        let piece = self.piece_loc(piece);
        let inner_row = piece.row > 0 && piece.row < self.last_row();
        let inner_col = piece.col > 0 && piece.col < self.last_col();
        PieceOrbitKey(1 + inner_row as usize + inner_col as usize)
    }

    fn piece_orbit_info(&self, orbit: PieceOrbitKey) -> PieceOrbitInfo {
        match orbit.0 {
            // corners
            0 | 1 => PieceOrbitInfo {
                rotations: 1,
                edges: 2,
                edge_increment_per_rotation: 2,
            },
            // edges
            2 => PieceOrbitInfo {
                rotations: 1,
                edges: 3,
                edge_increment_per_rotation: 3,
            },
            // centers
            3 => PieceOrbitInfo {
                rotations: 4,
                edges: 4,
                edge_increment_per_rotation: 1,
            },
            _ => panic!("invalid piece orbit"),
        }
    }

    fn num_edges(&self) -> usize {
        self.num_vert_edges() + self.num_horz_edges()
    }

    fn edge_orbit(&self, edge: EdgeKey) -> EdgeOrbitKey {
        let edge = self.edge_loc(edge);
        let inner = match edge.side {
            EdgeSide::Right => edge.loc.row > 0 && edge.loc.row < self.last_row(),
            EdgeSide::Down => edge.loc.col > 0 && edge.loc.col < self.last_col(),
        };
        EdgeOrbitKey(inner as usize)
    }

    fn piece_edge(&self, piece_edge: PieceKeyEdge) -> EdgeKey {
        let loc = self.piece_loc(piece_edge.piece);
        let edge = (piece_edge.edge + self.implicit_rotation(loc)) % 4;
        let edge_loc = match edge {
            0 => loc.up_edge(),
            1 => loc.right_edge(),
            2 => loc.down_edge(),
            3 => loc.left_edge(),
            _ => unreachable!(),
        };
        self.edge_key(edge_loc)
    }

    fn piece_neighbor(&self, piece_edge: PieceKeyEdge) -> PieceKeyEdge {
        let loc = self.piece_loc(piece_edge.piece);
        let edge = (piece_edge.edge + self.implicit_rotation(loc)) % 4;
        match edge {
            0 => self.piece_key_edge(loc.up_piece(), 2),
            1 => self.piece_key_edge(loc.right_piece(), 3),
            2 => self.piece_key_edge(loc.down_piece(), 0),
            3 => self.piece_key_edge(loc.left_piece(), 1),
            _ => unreachable!(),
        }
    }

    fn edge_pieces(&self, edge: EdgeKey) -> [PieceKeyEdge; 2] {
        let edge_loc = self.edge_loc(edge);
        match edge_loc.side {
            EdgeSide::Right => [
                self.piece_key_edge(edge_loc.loc, 1),
                self.piece_key_edge(edge_loc.loc.right_piece(), 3),
            ],
            EdgeSide::Down => [
                self.piece_key_edge(edge_loc.loc, 2),
                self.piece_key_edge(edge_loc.loc.down_piece(), 0),
            ],
        }
    }
}

// TODO tests!
