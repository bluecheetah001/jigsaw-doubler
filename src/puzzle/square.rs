use super::{EdgeKey, PieceKey, PointKey, Puzzle};

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
            piece: PieceLoc {
                row: self.row - 1,
                col: self.col,
            },
            side: EdgeSide::Down,
        }
    }
    fn right_edge(&self) -> EdgeLoc {
        EdgeLoc {
            piece: *self,
            side: EdgeSide::Right,
        }
    }
    fn down_edge(&self) -> EdgeLoc {
        EdgeLoc {
            piece: *self,
            side: EdgeSide::Down,
        }
    }
    fn left_edge(&self) -> EdgeLoc {
        EdgeLoc {
            piece: PieceLoc {
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
struct PointLoc {
    piece: PieceLoc,
    side: PointSide,
}
impl PointLoc {
    fn new(piece: PieceLoc, side: PointSide) -> Self {
        Self { piece, side }
    }
    fn new_up(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
            side: PointSide::Up,
        }
    }
    fn new_right(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
            side: PointSide::Right,
        }
    }
    fn new_down(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
            side: PointSide::Down,
        }
    }
    fn new_left(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
            side: PointSide::Left,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointSide {
    Up,
    Right,
    Down,
    Left,
}
enum PieceLocationKind {
    UpLeft,
    Up,
    UpRight,
    Left,
    Center,
    Right,
    DownLeft,
    Down,
    DownRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EdgeLoc {
    piece: PieceLoc,
    side: EdgeSide,
}
impl EdgeLoc {
    fn new_right(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
            side: EdgeSide::Right,
        }
    }
    fn new_down(row: usize, col: usize) -> Self {
        Self {
            piece: PieceLoc::new(row, col),
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
        // rows + cols must be even to have an even number of edges
        assert!((rows + cols) % 2 == 0);
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

    fn piece_location_kind(&self, piece: PieceLoc) -> PieceLocationKind {
        match (
            piece.row == 0,
            piece.row == self.last_row(),
            piece.col == 0,
            piece.col == self.last_col(),
        ) {
            (false, false, false, false) => PieceLocationKind::Center,
            (true, false, false, false) => PieceLocationKind::Up,
            (false, true, false, false) => PieceLocationKind::Down,
            (false, false, true, false) => PieceLocationKind::Left,
            (false, false, false, true) => PieceLocationKind::Right,
            (true, false, true, false) => PieceLocationKind::UpLeft,
            (true, false, false, true) => PieceLocationKind::UpRight,
            (false, true, true, false) => PieceLocationKind::DownLeft,
            (false, true, false, true) => PieceLocationKind::DownRight,
            _ => unreachable!(),
        }
    }
    fn point_orbit(&self, point: PointLoc) -> u32 {
        match (self.piece_location_kind(point.piece), point.side) {
            // up left corner is isolated to break rotational symmetry
            (PieceLocationKind::UpLeft, PointSide::Right) => 0,
            (PieceLocationKind::UpLeft, PointSide::Down) => 1,
            // corner
            (PieceLocationKind::UpRight, PointSide::Down) => 2,
            (PieceLocationKind::UpRight, PointSide::Left) => 3,
            (PieceLocationKind::DownRight, PointSide::Left) => 2,
            (PieceLocationKind::DownRight, PointSide::Up) => 3,
            (PieceLocationKind::DownLeft, PointSide::Up) => 2,
            (PieceLocationKind::DownLeft, PointSide::Right) => 3,
            // edge
            (PieceLocationKind::Up, PointSide::Right) => 4,
            (PieceLocationKind::Up, PointSide::Down) => 5,
            (PieceLocationKind::Up, PointSide::Left) => 6,
            (PieceLocationKind::Right, PointSide::Down) => 4,
            (PieceLocationKind::Right, PointSide::Left) => 5,
            (PieceLocationKind::Right, PointSide::Up) => 6,
            (PieceLocationKind::Down, PointSide::Left) => 4,
            (PieceLocationKind::Down, PointSide::Up) => 5,
            (PieceLocationKind::Down, PointSide::Right) => 6,
            (PieceLocationKind::Left, PointSide::Up) => 4,
            (PieceLocationKind::Left, PointSide::Right) => 5,
            (PieceLocationKind::Left, PointSide::Down) => 6,
            // center
            (PieceLocationKind::Center, _) => 7,

            _ => unreachable!(),
        }
    }

    fn piece_loc(&self, piece: PieceKey) -> PieceLoc {
        assert!(piece.0 < self.num_pieces());
        let row = piece.0 / self.cols;
        let col = piece.0 % self.cols;
        PieceLoc::new(row, col)
    }
    fn piece_key(&self, piece: PieceLoc) -> PieceKey {
        PieceKey(self.cols * piece.row + piece.col)
    }
    fn point_loc(&self, point: PointKey) -> PointLoc {
        let mut index = point.0;
        assert!(index < self.num_points());
        let first: bool = index & 1 == 0;
        index >>= 1;
        if index < self.num_vert_edges() {
            let row = index / (self.cols - 1);
            let col = index % (self.cols - 1);
            if first {
                PointLoc::new_right(row, col)
            } else {
                PointLoc::new_left(row, col + 1)
            }
        } else {
            index -= self.num_vert_edges();
            let row = index / self.cols;
            let col = index % self.cols;
            if first {
                PointLoc::new_down(row, col)
            } else {
                PointLoc::new_up(row + 1, col)
            }
        }
    }
    fn point_key(&self, point: PointLoc) -> PointKey {
        match point.side {
            PointSide::Right => PointKey((point.piece.row * (self.cols - 1) + point.piece.col) * 2),
            PointSide::Left => {
                PointKey((point.piece.row * (self.cols - 1) + (point.piece.col - 1)) * 2 + 1)
            }
            PointSide::Down => PointKey(
                (self.num_vert_edges() + point.piece.row * self.cols + point.piece.col) * 2,
            ),
            PointSide::Up => PointKey(
                (self.num_vert_edges() + (point.piece.row - 1) * self.cols + point.piece.col) * 2
                    + 1,
            ),
        }
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
            EdgeSide::Right => EdgeKey(edge.piece.row * (self.cols - 1) + edge.piece.col),
            EdgeSide::Down => {
                EdgeKey(self.num_vert_edges() + edge.piece.row * self.cols + edge.piece.col)
            }
        }
    }
}
impl Puzzle for SquarePuzzle {
    fn num_pieces(&self) -> usize {
        self.rows * self.cols
    }
    fn num_edges(&self) -> usize {
        self.num_vert_edges() + self.num_horz_edges()
    }

    fn arbitrary_point_on_piece(&self, piece: PieceKey) -> PointKey {
        let piece = self.piece_loc(piece);
        let side = if piece.row == 0 {
            PointSide::Down
        } else {
            PointSide::Up
        };
        self.point_key(PointLoc::new(piece, side))
    }
    fn next_point_on_piece(&self, point: PointKey) -> PointKey {
        let point = self.point_loc(point);
        let next_side = match (self.piece_location_kind(point.piece), point.side) {
            // corner
            (PieceLocationKind::UpLeft, PointSide::Right) => PointSide::Down,
            (PieceLocationKind::UpLeft, PointSide::Down) => PointSide::Right,
            (PieceLocationKind::UpRight, PointSide::Down) => PointSide::Left,
            (PieceLocationKind::UpRight, PointSide::Left) => PointSide::Down,
            (PieceLocationKind::DownRight, PointSide::Left) => PointSide::Up,
            (PieceLocationKind::DownRight, PointSide::Up) => PointSide::Left,
            (PieceLocationKind::DownLeft, PointSide::Up) => PointSide::Right,
            (PieceLocationKind::DownLeft, PointSide::Right) => PointSide::Up,
            // edge
            (PieceLocationKind::Up, PointSide::Right) => PointSide::Down,
            (PieceLocationKind::Up, PointSide::Down) => PointSide::Left,
            (PieceLocationKind::Up, PointSide::Left) => PointSide::Right,
            (PieceLocationKind::Right, PointSide::Down) => PointSide::Left,
            (PieceLocationKind::Right, PointSide::Left) => PointSide::Up,
            (PieceLocationKind::Right, PointSide::Up) => PointSide::Down,
            (PieceLocationKind::Down, PointSide::Left) => PointSide::Up,
            (PieceLocationKind::Down, PointSide::Up) => PointSide::Right,
            (PieceLocationKind::Down, PointSide::Right) => PointSide::Left,
            (PieceLocationKind::Left, PointSide::Up) => PointSide::Right,
            (PieceLocationKind::Left, PointSide::Right) => PointSide::Down,
            (PieceLocationKind::Left, PointSide::Down) => PointSide::Up,
            // center
            (PieceLocationKind::Center, PointSide::Up) => PointSide::Right,
            (PieceLocationKind::Center, PointSide::Right) => PointSide::Down,
            (PieceLocationKind::Center, PointSide::Down) => PointSide::Left,
            (PieceLocationKind::Center, PointSide::Left) => PointSide::Up,

            _ => unreachable!(),
        };
        self.point_key(PointLoc::new(point.piece, next_side))
    }
    fn point_piece(&self, point: PointKey) -> PieceKey {
        self.piece_key(self.point_loc(point).piece)
    }

    fn arbitrary_point_on_edge(&self, edge: EdgeKey) -> PointKey {
        let edge = self.edge_loc(edge);
        let side = match edge.side {
            EdgeSide::Right => PointSide::Right,
            EdgeSide::Down => PointSide::Down,
        };
        self.point_key(PointLoc::new(edge.piece, side))
    }
    fn other_point_on_edge(&self, point: PointKey) -> PointKey {
        let point = self.point_loc(point);
        let other_point = match point.side {
            PointSide::Up => PointLoc::new_down(point.piece.row - 1, point.piece.col),
            PointSide::Right => PointLoc::new_left(point.piece.row, point.piece.col + 1),
            PointSide::Down => PointLoc::new_up(point.piece.row + 1, point.piece.col),
            PointSide::Left => PointLoc::new_right(point.piece.row, point.piece.col - 1),
        };
        self.point_key(other_point)
    }
    fn point_edge(&self, point: PointKey) -> EdgeKey {
        let point = self.point_loc(point);
        let edge = match point.side {
            PointSide::Up => EdgeLoc::new_down(point.piece.row - 1, point.piece.col),
            PointSide::Right => EdgeLoc::new_right(point.piece.row, point.piece.col),
            PointSide::Down => EdgeLoc::new_down(point.piece.row, point.piece.col),
            PointSide::Left => EdgeLoc::new_right(point.piece.row, point.piece.col - 1),
        };
        self.edge_key(edge)
    }

    fn can_exchange(&self, point_a: PointKey, point_b: PointKey) -> bool {
        let point_a = self.point_loc(point_a);
        let point_b = self.point_loc(point_b);
        self.point_orbit(point_a) == self.point_orbit(point_b)
    }

    fn format_edge(&self, edge: EdgeKey) -> String {
        let edge = self.edge_loc(edge);
        let side_char = match edge.side {
            EdgeSide::Right => '|',
            EdgeSide::Down => '_',
        };
        format!(
            "{}{}{}",
            row_char(edge.piece.row),
            col_char(edge.piece.col),
            side_char
        )
    }
    fn format_point(&self, point: PointKey) -> String {
        let point = self.point_loc(point);
        let side_char = match point.side {
            PointSide::Up => '^',
            PointSide::Right => '>',
            PointSide::Down => 'v',
            PointSide::Left => '<',
        };
        format!(
            "{}{}{}",
            row_char(point.piece.row),
            col_char(point.piece.col),
            side_char
        )
    }
}
fn row_char(row: usize) -> char {
    assert!(row < 26);
    ('a' as u8 + row as u8) as char
}
fn col_char(col: usize) -> char {
    assert!(col < 9);
    ('1' as u8 + col as u8) as char
}

// TODO tests!
