use super::{EdgeKey, EdgeOrbitInfo, PieceKey, PieceKeyEdge, PieceOrbitInfo, Puzzle};

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum PieceOrbit{
//     Corner,
//     Edge,
//     Center
// }
// impl PieceOrbit {
//     fn from_index(index:usize)->Self{
//         match index {
//             0 =>Self::Corner,
//             1=>Self::Edge,
//             2=>Self::Center
//         }
//     }
//     fn to_index(&self)->usize{

//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum EdgeOrbit{
//     Outer,
//     Inner,
// }

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
struct PieceLocRot {
    loc: PieceLoc,
    implicit_rotation: usize,
}
impl PieceLocRot {
    fn new(row: usize, col: usize, rot: usize) -> Self {
        Self {
            loc: PieceLoc::new(row, col),
            implicit_rotation: rot,
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
    fn up_edge_pieces(&self) -> usize {
        self.cols - 2
    }
    fn left_edge_pieces(&self) -> usize {
        self.rows - 2
    }
    fn up_outer_edges(&self) -> usize {
        self.cols - 1
    }
    fn left_outer_edges(&self) -> usize {
        self.rows - 1
    }
    fn vert_inner_edges(&self) -> usize {
        self.left_edge_pieces() * self.up_outer_edges()
    }
    fn horz_inner_edges(&self) -> usize {
        self.up_edge_pieces() * self.left_outer_edges()
    }

    fn last_col(&self) -> usize {
        self.cols - 1
    }
    fn last_row(&self) -> usize {
        self.rows - 1
    }

    fn piece_loc_rot(&self, piece: PieceKey) -> PieceLocRot {
        match piece.orbit {
            // corners
            0 => match piece.index {
                // ul
                0 => PieceLocRot::new(0, 0, 1),
                // ur
                1 => PieceLocRot::new(0, self.last_col(), 2),
                // dl
                2 => PieceLocRot::new(self.last_row(), 0, 0),
                // dr
                3 => PieceLocRot::new(self.last_row(), self.last_col(), 3),
                _ => panic!("invalid corner piece index"),
            },
            // edges
            1 => {
                let mut index = piece.index;
                // up
                if index < self.up_edge_pieces() {
                    return PieceLocRot::new(0, 1 + index, 1);
                }
                index -= self.up_edge_pieces();
                // down
                if index < self.up_edge_pieces() {
                    return PieceLocRot::new(self.last_row(), 1 + index, 3);
                }
                index -= self.up_edge_pieces();

                // left
                if index < self.left_edge_pieces() {
                    return PieceLocRot::new(1 + index, 0, 0);
                }
                index -= self.left_edge_pieces();
                //right
                if index < self.left_edge_pieces() {
                    return PieceLocRot::new(1 + index, self.last_col(), 2);
                }
                // don't need to `index -= self.left_edge_pieces();` since this is last
                panic!("invalid edge piece index");
            }
            // centers
            2 => {
                if piece.index > self.up_edge_pieces() * self.left_edge_pieces() {
                    panic!("invalid center piece index")
                }
                PieceLocRot::new(
                    1 + piece.index / self.up_edge_pieces(),
                    1 + piece.index % self.up_edge_pieces(),
                    0,
                )
            }
            _ => panic!("invalid piece orbit"),
        }
    }
    fn piece_key_edge(&self, piece: PieceLoc, edge: usize) -> PieceKeyEdge {
        if piece.row == 0 {
            if piece.col == 0 {
                PieceKeyEdge::new(0, 0, (edge + 3) % 4)
            } else if piece.col == self.last_col() {
                PieceKeyEdge::new(0, 1, (edge + 2) % 4)
            } else {
                PieceKeyEdge::new(1, piece.col - 1, (edge + 3) % 4)
            }
        } else if piece.row == self.last_row() {
            if piece.col == 0 {
                PieceKeyEdge::new(0, 2, edge)
            } else if piece.col == self.last_col() {
                PieceKeyEdge::new(0, 3, (edge + 1) % 4)
            } else {
                PieceKeyEdge::new(1, self.up_edge_pieces() + piece.col - 1, (edge + 1) % 4)
            }
        } else {
            if piece.col == 0 {
                PieceKeyEdge::new(1, self.up_edge_pieces() * 2 + piece.row - 1, edge)
            } else if piece.col == self.last_col() {
                PieceKeyEdge::new(
                    1,
                    self.up_edge_pieces() * 2 + self.left_edge_pieces() + piece.row - 1,
                    (edge + 2) % 4,
                )
            } else {
                PieceKeyEdge::new(
                    2,
                    (piece.row - 1) * self.up_edge_pieces() + piece.col - 1,
                    edge,
                )
            }
        }
    }
    fn edge_loc(&self, edge: EdgeKey) -> EdgeLoc {
        match edge.orbit {
            // outer
            0 => {
                let mut index = edge.index;
                // up
                if index < self.up_outer_edges() {
                    return EdgeLoc::new_right(0, index);
                }
                index -= self.up_outer_edges();
                // down
                if index < self.up_outer_edges() {
                    return EdgeLoc::new_right(self.last_row(), index);
                }
                index -= self.up_outer_edges();

                // left
                if index < self.left_outer_edges() {
                    return EdgeLoc::new_down(index, 0);
                }
                index -= self.left_outer_edges();
                //right
                if index < self.left_outer_edges() {
                    return EdgeLoc::new_down(index, self.last_col());
                }
                // don't need to `index -= self.left_outer_edges();` since this is last
                panic!("invalid outer edge index");
            }
            // inner
            1 => {
                let mut index = edge.index;
                // vert
                if index < self.vert_inner_edges() {
                    return EdgeLoc::new_right(
                        1 + index / self.up_outer_edges(),
                        index % self.up_outer_edges(),
                    );
                }
                index -= self.vert_inner_edges();

                // horz
                if index < self.horz_inner_edges() {
                    return EdgeLoc::new_right(
                        index / self.up_edge_pieces(),
                        1 + index % self.up_edge_pieces(),
                    );
                }
                // don't need to `index -= self.horz_inner_edges();` since this is last
                panic!("invalid outer edge index");
            }
            _ => panic!("invalid edge orbit"),
        }
    }
    fn edge_key(&self, edge: EdgeLoc) -> EdgeKey {
        match edge.side {
            EdgeSide::Right => {
                if edge.loc.row == 0 {
                    EdgeKey::new(0, edge.loc.col)
                } else if edge.loc.row == self.last_row() {
                    EdgeKey::new(0, self.up_outer_edges() + edge.loc.col)
                } else {
                    EdgeKey::new(1, (edge.loc.row - 1) * self.up_outer_edges() + edge.loc.col)
                }
            }
            EdgeSide::Down => {
                if edge.loc.col == 0 {
                    EdgeKey::new(0, self.up_outer_edges() * 2 + edge.loc.row)
                } else if edge.loc.col == self.last_col() {
                    EdgeKey::new(
                        0,
                        self.up_outer_edges() * 2 + self.left_outer_edges() + edge.loc.row,
                    )
                } else {
                    EdgeKey::new(
                        1,
                        self.vert_inner_edges()
                            + edge.loc.row * self.up_edge_pieces()
                            + (edge.loc.col - 1),
                    )
                }
            }
        }
    }
}
impl Puzzle for SquarePuzzle {
    fn num_piece_orbits(&self) -> usize {
        3
    }

    fn piece_orbit(&self, orbit: usize) -> PieceOrbitInfo {
        match orbit {
            // corners
            0 => PieceOrbitInfo {
                len: 4,
                rotations: 1,
                edges: 2,
                edge_increment_per_rotation: 2,
            },
            // edges
            1 => PieceOrbitInfo {
                len: (self.up_edge_pieces() + self.left_edge_pieces()) * 2,
                rotations: 1,
                edges: 3,
                edge_increment_per_rotation: 3,
            },
            // centers
            2 => PieceOrbitInfo {
                len: self.up_edge_pieces() * self.left_edge_pieces(),
                rotations: 4,
                edges: 4,
                edge_increment_per_rotation: 1,
            },
            _ => panic!("invalid piece orbit"),
        }
    }

    fn num_edge_orbits(&self) -> usize {
        2
    }

    fn edge_orbit(&self, orbit: usize) -> EdgeOrbitInfo {
        match orbit {
            // outer
            0 => EdgeOrbitInfo {
                len: (self.up_outer_edges() + self.left_outer_edges()) * 2,
            },
            // inner
            1 => EdgeOrbitInfo {
                len: self.vert_inner_edges() * self.horz_inner_edges(),
            },
            _ => panic!("invalid edge orbit"),
        }
    }

    fn piece_edge(&self, piece: PieceKey, edge: usize) -> EdgeKey {
        let loc_rot = self.piece_loc_rot(piece);
        let edge = (edge + loc_rot.implicit_rotation) % 4;
        let edge_loc = match edge {
            0 => loc_rot.loc.up_edge(),
            1 => loc_rot.loc.right_edge(),
            2 => loc_rot.loc.down_edge(),
            3 => loc_rot.loc.left_edge(),
            _ => unreachable!(),
        };
        self.edge_key(edge_loc)
    }

    fn piece_neighbor(&self, piece: PieceKey, edge: usize) -> PieceKeyEdge {
        let loc_rot = self.piece_loc_rot(piece);
        let edge = (edge + loc_rot.implicit_rotation) % 4;
        match edge {
            0 => self.piece_key_edge(loc_rot.loc.up_piece(), 2),
            1 => self.piece_key_edge(loc_rot.loc.right_piece(), 3),
            2 => self.piece_key_edge(loc_rot.loc.down_piece(), 0),
            3 => self.piece_key_edge(loc_rot.loc.left_piece(), 1),
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
