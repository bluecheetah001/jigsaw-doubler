use std::fmt::Debug;

mod sat;
use sat::*;

fn main() {
    let config = Config::new(5, 5, 2);
    JigsawDoubler::run(config);
    println!("Hello, world!");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieceKind {
    Center,
    Edge,
    Corner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PieceLocation {
    UL,
    UC,
    UR,
    CL,
    CC,
    CR,
    DL,
    DC,
    DR,
}

struct Config {
    rows: usize,
    cols: usize,
    edge_dups: usize,

    num_edges: usize,
    num_edge_ids: usize,
    num_pieces: usize,
}
impl Config {
    fn new(rows: usize, cols: usize, edge_dups: usize) -> Self {
        // allowing rows or cols to equal 1 creates edge pieces that are able to rotate.
        // a lot of code is assuming edges can't rotate
        assert!(rows >= 2);
        assert!(cols >= 2);
        // edge dups < 2 trivially can't produce a puzzle with multiple solutions
        assert!(edge_dups >= 2);

        let num_edges = rows * (cols - 1) + (rows - 1) * cols;
        // loosening this to allow some edge ids to be duplicated an extra time should only affect JigsawDoubler::add_edge_duplicate_ids
        // (and make sure that num_edge_ids is computed properly)
        assert!(num_edges % edge_dups == 0);
        let num_edge_ids = num_edges / edge_dups;

        let num_pieces = rows * cols;

        Self {
            rows,
            cols,
            edge_dups,
            num_edges,
            num_edge_ids,
            num_pieces,
        }
    }
    fn piece_kind(&self, p: usize) -> PieceKind {
        let c = p % self.cols;
        let r = p / self.cols;
        let c_edge = c == 0 || c == self.cols - 1;
        let r_edge = r == 0 || r == self.rows - 1;
        if c_edge && r_edge {
            PieceKind::Corner
        } else if c_edge || r_edge {
            PieceKind::Edge
        } else {
            PieceKind::Center
        }
    }
    fn piece_location(&self, p: usize) -> PieceLocation {
        let c = p % self.cols;
        let r = p / self.cols;
        match (c == 0, c == self.cols - 1, r == 0, r == self.rows - 1) {
            (false, false, false, false) => PieceLocation::CC,
            (true, false, false, false) => PieceLocation::CL,
            (false, true, false, false) => PieceLocation::CR,
            (false, false, true, false) => PieceLocation::UC,
            (true, false, true, false) => PieceLocation::UL,
            (false, true, true, false) => PieceLocation::UR,
            (false, false, false, true) => PieceLocation::DC,
            (true, false, false, true) => PieceLocation::DL,
            (false, true, false, true) => PieceLocation::DR,
            _ => unreachable!(),
        }
    }
    // get edge index of side of piece index
    fn piece_edge(&self, mut p: usize, side: usize) -> usize {
        // center piece sides are u, r, d, l
        // edge and corner piece sides are clockwise from puzzle edge
        // piece index is row major
        // edge index is row major for vertical then row major for horizontal
        let d_side = match self.piece_location(p) {
            PieceLocation::UL => 1,
            PieceLocation::UC => 1,
            PieceLocation::UR => 2,
            PieceLocation::CL => 0,
            PieceLocation::CC => 0,
            PieceLocation::CR => 2,
            PieceLocation::DL => 0,
            PieceLocation::DC => 3,
            PieceLocation::DR => 3,
        };
        let side = (side + d_side) % 4;
        if side == 0 {
            p -= self.cols;
        }
        if side == 3 {
            p -= 1
        }
        let c = p % self.cols;
        let r = p / self.cols;

        // vertical
        if side & 1 == 1 {
            r * (self.cols - 1) + c
        } else {
            let vertical_edges = (self.cols - 1) * self.rows;
            vertical_edges + r * self.cols + c
        }
    }
    // get neighbor piece index of side of piece index
    fn piece_neighbor(&self, p: usize, side: usize) -> usize {
        let d_side = match self.piece_location(p) {
            PieceLocation::UL => 1,
            PieceLocation::UC => 1,
            PieceLocation::UR => 2,
            PieceLocation::CL => 0,
            PieceLocation::CC => 0,
            PieceLocation::CR => 2,
            PieceLocation::DL => 0,
            PieceLocation::DC => 3,
            PieceLocation::DR => 3,
        };
        match (side + d_side) % 4 {
            0 => p - self.cols,
            1 => p + 1,
            2 => p + self.cols,
            3 => p - 1,
            _ => unreachable!(),
        }
    }
    // get the implicit rotation from moving src piece to dst piece
    fn implicit_rotation(&self, src: usize, dst: usize) -> usize {
        // TODO cleanup duplication
        //      probably through introduction of Piece, Edge, Rotation, and RotatedPiece structs
        let src_rot = match self.piece_location(src) {
            PieceLocation::UL => 0,
            PieceLocation::UC => 0,
            PieceLocation::UR => 1,
            PieceLocation::CR => 1,
            PieceLocation::DR => 2,
            PieceLocation::DC => 2,
            PieceLocation::DL => 3,
            PieceLocation::CL => 3,
            PieceLocation::CC => 0,
        };
        let dst_rot = match self.piece_location(dst) {
            PieceLocation::UL => 0,
            PieceLocation::UC => 0,
            PieceLocation::UR => 1,
            PieceLocation::CR => 1,
            PieceLocation::DR => 2,
            PieceLocation::DC => 2,
            PieceLocation::DL => 3,
            PieceLocation::CL => 3,
            PieceLocation::CC => 0,
        };
        dst_rot - src_rot
    }
}
struct SingleEdgeVars {
    id: Vec<Lit>,
    dst_id: Vec<Lit>,
}
struct SinglePieceVars {
    dst: Vec<Lit>,
    rot: Option<[Lit; 4]>,
}
struct JigsawDoubler {
    config: Config,
    problem: Problem,
    edge_vars: Vec<SingleEdgeVars>,
    piece_vars: Vec<SinglePieceVars>,
}
impl JigsawDoubler {
    pub fn run(config: Config) {
        let mut s = Self::new(config);
        s.add_edge_vars();
        s.add_piece_vars();
        s.add_edge_one_hot_id();
        s.add_edge_duplicate_ids();
        s.add_piece_one_hot_dst();
        s.add_piece_one_hot_rot();
        s.add_piece_no_duplicate_dst();
        s.add_dst_valid();
        s.add_dst_changed();

        // TODO run problem

        // TODO extract result
    }
    fn new(config: Config) -> Self {
        let problem = Problem::new();
        let edge_vars = vec![];
        let piece_vars = vec![];
        Self {
            config,
            problem,
            edge_vars,
            piece_vars,
        }
    }
    fn add_edge_vars(&mut self) {
        self.edge_vars = (0..self.config.num_edges)
            .into_iter()
            .map(|edge| Self::new_single_edge_vars(&self.config, &mut self.problem, edge))
            .collect();
    }
    fn new_single_edge_vars(
        config: &Config,
        problem: &mut Problem,
        edge_index: usize,
    ) -> SingleEdgeVars {
        let id = (0..config.num_edge_ids)
            .into_iter()
            .map(|edge_id| {
                // to help break symetries, don't allow edge to have an id greater than its index
                if edge_id <= edge_index {
                    if edge_index == 0 {
                        Lit::TRUE
                    } else {
                        problem.var()
                    }
                } else {
                    Lit::FALSE
                }
            })
            .collect();
        let dst_id = (0..config.num_edge_ids)
            .into_iter()
            .map(|_| problem.var())
            .collect();
        SingleEdgeVars { id, dst_id }
    }
    fn add_piece_vars(&mut self) {
        self.piece_vars = (0..self.config.num_pieces)
            .into_iter()
            .map(|piece| Self::new_single_piece_vars(&self.config, &mut self.problem, piece))
            .collect();
    }
    fn new_single_piece_vars(
        config: &Config,
        problem: &mut Problem,
        src: usize,
    ) -> SinglePieceVars {
        let src_k = config.piece_kind(src);
        let dst = (0..config.num_pieces)
            .into_iter()
            .map(|dst| {
                let dst_k = config.piece_kind(dst);

                // force piece 0,0 to not move
                if src == 0 {
                    return if dst == 0 { Lit::TRUE } else { Lit::FALSE };
                }
                if dst == 0 {
                    return Lit::FALSE;
                }

                // piece kind must agree
                if src_k == dst_k {
                    problem.var()
                } else {
                    Lit::FALSE
                }
            })
            .collect();
        let rot = if src_k == PieceKind::Center {
            Some([problem.var(), problem.var(), problem.var(), problem.var()])
        } else {
            None
        };
        SinglePieceVars { dst, rot }
    }

    fn add_edge_one_hot_id(&mut self) {
        for e in self.edge_vars.iter() {
            let count = self.problem.count_up_to(2, &e.id);
            self.problem.clause(vec![count[1]]);
            self.problem.clause(vec![!count[2]]);
        }
    }
    fn add_edge_duplicate_ids(&mut self) {
        for id in 0..self.config.num_edge_ids {
            let vars = self.edge_vars.iter().map(|e| e.id[id]).collect::<Vec<_>>();
            let count = self.problem.count_up_to(self.config.edge_dups + 1, &vars);
            self.problem.clause(vec![count[self.config.edge_dups]]);
            self.problem.clause(vec![!count[self.config.edge_dups + 1]]);
        }
    }
    fn add_piece_one_hot_dst(&mut self) {
        for p in self.piece_vars.iter() {
            let count = self.problem.count_up_to(2, &p.dst);
            self.problem.clause(vec![count[1]]);
            self.problem.clause(vec![!count[2]]);
        }
    }
    fn add_piece_one_hot_rot(&mut self) {
        for p in self.piece_vars.iter() {
            if let Some(rot) = &p.rot {
                let count = self.problem.count_up_to(2, rot);
                self.problem.clause(vec![count[1]]);
                self.problem.clause(vec![!count[2]]);
            }
        }
    }
    fn add_piece_no_duplicate_dst(&mut self) {
        for dst in 0..self.config.num_pieces {
            let vars = self
                .piece_vars
                .iter()
                .map(|p| p.dst[dst])
                .collect::<Vec<_>>();
            let count = self.problem.count_up_to(2, &vars);
            self.problem.clause(vec![count[1]]);
            self.problem.clause(vec![!count[2]]);
        }
    }

    fn add_dst_valid(&mut self) {
        for (src, src_vars) in self.piece_vars.iter().enumerate() {
            for (dst, dst_var) in src_vars.dst.iter().enumerate() {
                if dst_var.is_false() {
                    continue;
                }
                match self.config.piece_kind(src) {
                    PieceKind::Center => {
                        for (j, rot_var) in src_vars.rot.unwrap().iter().enumerate() {
                            for i in 0..4 {
                                Self::add_implies_move_edge(
                                    &mut self.problem,
                                    &self.edge_vars,
                                    *dst_var,
                                    *rot_var,
                                    self.config.piece_edge(src, i),
                                    self.config.piece_edge(dst, (i + j) % 4),
                                )
                            }
                        }
                    }
                    PieceKind::Edge => {
                        for i in 0..3 {
                            Self::add_implies_move_edge(
                                &mut self.problem,
                                &self.edge_vars,
                                *dst_var,
                                Lit::TRUE,
                                self.config.piece_edge(src, i),
                                self.config.piece_edge(dst, i),
                            )
                        }
                    }
                    PieceKind::Corner => {
                        for i in 0..2 {
                            Self::add_implies_move_edge(
                                &mut self.problem,
                                &self.edge_vars,
                                *dst_var,
                                Lit::TRUE,
                                self.config.piece_edge(src, i),
                                self.config.piece_edge(dst, i),
                            )
                        }
                    }
                }
            }
        }
    }
    fn add_implies_move_edge(
        problem: &mut Problem,
        edge_vars: &[SingleEdgeVars],
        piece_dst: Lit,
        piece_rot: Lit,
        src_e: usize,
        dst_e: usize,
    ) {
        let src_e = &*edge_vars[src_e].id;
        let dst_e = &*edge_vars[dst_e].dst_id;
        for (src_e, dst_e) in std::iter::zip(src_e, dst_e) {
            // piece_dst & piece_rot => (src_e == dst_e)
            problem.clause(vec![!piece_dst, !piece_rot, !*src_e, *dst_e]);
            problem.clause(vec![!piece_dst, !piece_rot, *src_e, !*dst_e]);
        }
    }

    fn add_dst_changed(&mut self) {
        for (src, src_vars) in self.piece_vars.iter().enumerate() {
            for (dst, dst_var) in src_vars.dst.iter().enumerate() {
                if dst_var.is_false() {
                    continue;
                }
                match self.config.piece_kind(src) {
                    PieceKind::Center => {
                        for (j, rot_var) in src_vars.rot.unwrap().iter().enumerate() {
                            for i in 0..4 {
                                Self::add_dst_not_adjacent(
                                    &mut self.problem,
                                    &self.piece_vars,
                                    *dst_var,
                                    *rot_var,
                                    self.config.piece_neighbor(src, i),
                                    self.config.piece_neighbor(dst, (i + j) % 4),
                                    j,
                                )
                            }
                        }
                    }
                    PieceKind::Edge => {
                        for i in 0..3 {
                            Self::add_dst_not_adjacent(
                                &mut self.problem,
                                &self.piece_vars,
                                *dst_var,
                                Lit::TRUE,
                                self.config.piece_neighbor(src, i),
                                self.config.piece_neighbor(dst, i),
                                self.config.implicit_rotation(src, dst),
                            )
                        }
                    }
                    PieceKind::Corner => {
                        for i in 0..2 {
                            Self::add_dst_not_adjacent(
                                &mut self.problem,
                                &self.piece_vars,
                                *dst_var,
                                Lit::TRUE,
                                self.config.piece_neighbor(src, i),
                                self.config.piece_neighbor(dst, i),
                                self.config.implicit_rotation(src, dst),
                            )
                        }
                    }
                }
            }
        }
    }
    fn add_dst_not_adjacent(
        problem: &mut Problem,
        piece_vars: &[SinglePieceVars],
        piece_dst: Lit,
        piece_rot: Lit,
        neighbor: usize,
        neighbor_dst: usize,
        neighbor_rot: usize,
    ) {
        let neighbor_vars = &piece_vars[neighbor];
        let neighbor_dst = neighbor_vars.dst[neighbor_dst];
        let neighbor_rot = neighbor_vars
            .rot
            .map(|r| r[neighbor_rot])
            // if a neighbor is implicitly rotated, then it either can't go to the destination or will have the correct rotation
            .unwrap_or(Lit::TRUE);

        // TODO pretty sure this clause is emitted twice due to it's symetry
        // piece_dst & piece_rot => !neighbor_dst | !neighbor_rot
        problem.clause(vec![!piece_dst, !piece_rot, !neighbor_dst, !neighbor_rot]);
    }
}
