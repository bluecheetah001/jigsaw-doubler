use std::collections::HashMap;
use std::fmt::Debug;

mod puzzle;
mod sat;
use itertools::Itertools;
use puzzle::*;
use sat::*;
use varisat::Lit;

fn main() {
    let puzzle = SquarePuzzle::new(5, 5);
    JigsawDoubler::run(puzzle);
    println!("Hello, world!");
}

struct EdgeMatchingVars {
    orbit: EdgeOrbitInfo,
    // lower-triangular matrix (c < r)
    // index(r,c) = r*(r-1)/2 + c
    // var is true if edge c and edge r have the same shape
    matches: Vec<Lit>,
}
impl EdgeMatchingVars {
    fn new(orbit: EdgeOrbitInfo, sat: &mut SatProblem) -> Self {
        let len = orbit.len * (orbit.len - 1) / 2;
        let matches = (0..len).map(|_| sat.var()).collect_vec();
        Self { orbit, matches }
    }
    fn matches(&self, a: usize, b: usize) -> Lit {
        assert!(a < self.orbit.len);
        assert!(b < self.orbit.len);
        assert!(a != b);
        let c = a.min(b);
        let r = a.max(b);
        self.matches[r * (r - 1) / 2 + c]
    }
}

struct PieceDestVars {
    orbit: PieceOrbitInfo,
    // square matrix
    // index(s,d) = s*len+d
    // var is true if piece s ends up at piece d
    dest: Vec<Lit>,
    // rectangular matrix
    // index(s,r) = s*len+r
    // var is true if piece s ends up with rotation r
    rot: Vec<Lit>,
}
impl PieceDestVars {
    fn new(orbit: PieceOrbitInfo, sat: &mut SatProblem) -> Self {
        let dest_len = orbit.len * orbit.len;
        let dest = (0..dest_len).map(|_| sat.var()).collect_vec();
        let rot_len = orbit.len * orbit.rotations;
        let rot = (0..rot_len).map(|_| sat.var()).collect_vec();
        Self { orbit, dest, rot }
    }
    fn dest(&self, s: usize, d: usize) -> Lit {
        assert!(s < self.orbit.len);
        assert!(d < self.orbit.len);
        self.dest[s * self.orbit.len + d]
    }
    fn rot(&self, s: usize, r: usize) -> Lit {
        assert!(s < self.orbit.len);
        assert!(r < self.orbit.rotations);
        self.rot[s * self.orbit.len + r]
    }
}

struct PieceDestAdjacentVars {
    map: HashMap<(PieceKeyEdge, PieceKeyEdge), Lit>,
}
impl PieceDestAdjacentVars {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    fn put(&mut self, a: PieceKeyEdge, b: PieceKeyEdge, var: Lit) {
        assert!(a < b);
        let key = (a, b);
        assert!(!self.map.contains_key(&key));
        self.map.insert(key, var);
    }
    // return isn't Option as I expect that callers will naturally not call with arguments that can't be adjacent
    fn get(&mut self, a: PieceKeyEdge, b: PieceKeyEdge) -> Lit {
        let key = (a.min(b), a.max(b));
        *self.map.get(&key).unwrap()
    }
}

struct JigsawDoubler<P> {
    puzzle: P,
    sat: SatProblem,
    edge_matching_per_orbit: Vec<EdgeMatchingVars>,
    piece_dest_per_orbit: Vec<PieceDestVars>,
    piece_dest_is_adjacent: PieceDestAdjacentVars,
}
impl<P: Puzzle> JigsawDoubler<P> {
    pub fn run(puzzle: P) {
        let mut s = Self::new(puzzle);
        s.add_edge_vars();
        s.add_piece_vars();
        s.add_edge_one_hot_matching();
        s.add_piece_one_hot_dst();
        s.add_piece_one_hot_rot();
        s.add_piece_one_hot_src();
        s.add_piece_dst_adjacent_vars();
        s.add_piece_dst_adjacent_not_same();
        s.add_piece_dst_adjacent_to_matching();

        // TODO run problem

        // TODO extract result
    }
    fn new(puzzle: P) -> Self {
        Self {
            puzzle,
            sat: SatProblem::new(),
            edge_matching_per_orbit: vec![],
            piece_dest_per_orbit: vec![],
            piece_dest_is_adjacent: PieceDestAdjacentVars::new(),
        }
    }
    fn add_edge_vars(&mut self) {
        self.edge_matching_per_orbit = (0..self.puzzle.num_edge_orbits())
            .map(|i| self.puzzle.edge_orbit(i))
            .map(|o| EdgeMatchingVars::new(o, &mut self.sat))
            .collect_vec();
    }
    fn add_piece_vars(&mut self) {
        self.piece_dest_per_orbit = (0..self.puzzle.num_piece_orbits())
            .map(|i| self.puzzle.piece_orbit(i))
            .map(|o| PieceDestVars::new(o, &mut self.sat))
            .collect_vec();
    }

    fn add_edge_one_hot_matching(&mut self) {
        for info in self.edge_matching_per_orbit.iter() {
            for a in 0..info.orbit.len {
                let a_matches = (0..info.orbit.len)
                    .filter(|b| *b != a)
                    .map(|b| info.matches(a, b))
                    .collect_vec();
                self.sat.exact_count_clause(1, &a_matches);
            }
        }
    }
    fn add_piece_one_hot_dst(&mut self) {
        for info in self.piece_dest_per_orbit.iter() {
            for s in 0..info.orbit.len {
                let s_dst = (0..info.orbit.len).map(|d| info.dest(s, d)).collect_vec();
                self.sat.exact_count_clause(1, &s_dst);
            }
        }
    }
    fn add_piece_one_hot_rot(&mut self) {
        for info in self.piece_dest_per_orbit.iter() {
            for s in 0..info.orbit.len {
                let s_rot = (0..info.orbit.rotations)
                    .map(|r| info.rot(s, r))
                    .collect_vec();
                self.sat.exact_count_clause(1, &s_rot);
            }
        }
    }
    fn add_piece_one_hot_src(&mut self) {
        for info in self.piece_dest_per_orbit.iter() {
            for d in 0..info.orbit.len {
                let d_src = (0..info.orbit.len).map(|s| info.dest(s, d)).collect_vec();
                self.sat.exact_count_clause(1, &d_src);
            }
        }
    }

    fn add_piece_dst_adjacent_vars(&mut self) {
        todo!()
        // loop over piece-a-edge, piece-b-edge
        //   dst-adjacent = and(piece-a-edge-dst == dst-edge-parity && piece-b-edge-dst == dst-edge-parity for each dst-edge-parity)
    }

    fn add_piece_dst_adjacent_not_same(&mut self) {
        for (edge_orbit, edge_info) in self.edge_matching_per_orbit.iter().enumerate() {
            for edge_a in 0..edge_info.orbit.len {
                let [piece_a_1, piece_a_2] =
                    self.puzzle.edge_pieces(EdgeKey::new(edge_orbit, edge_a));
                // destination pieces must change neighbors
                self.sat
                    .not_clause(self.piece_dest_is_adjacent.get(piece_a_1, piece_a_2));
            }
        }
    }

    fn add_piece_dst_adjacent_to_matching(&mut self) {
        for (edge_orbit, edge_info) in self.edge_matching_per_orbit.iter().enumerate() {
            for edge_a in 0..edge_info.orbit.len {
                let [piece_a_1, piece_a_2] =
                    self.puzzle.edge_pieces(EdgeKey::new(edge_orbit, edge_a));

                for edge_b in 0..edge_a {
                    let [piece_b_1, piece_b_2] =
                        self.puzzle.edge_pieces(EdgeKey::new(edge_orbit, edge_b));

                    // edges match implies destination pieces trade neighbors
                    let case_1 = self.sat.and_var(&[
                        self.piece_dest_is_adjacent.get(piece_a_1, piece_b_1),
                        self.piece_dest_is_adjacent.get(piece_a_2, piece_b_2),
                    ]);
                    let case_2 = self.sat.and_var(&[
                        self.piece_dest_is_adjacent.get(piece_a_1, piece_b_2),
                        self.piece_dest_is_adjacent.get(piece_a_2, piece_b_1),
                    ]);
                    let dest_matches = self.sat.or_var(&[case_1, case_2]);
                    self.sat
                        .implies_clause(edge_info.matches(edge_a, edge_b), dest_matches);
                }
            }
        }
    }
}
