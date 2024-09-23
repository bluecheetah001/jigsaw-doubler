use std::collections::HashMap;
use std::hash::Hash;

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

// 0: base
// .--.--.--.  .--.--.--.
// |  0  1  |  | 0| 1| 2|
// .-6.-7.-8.  .--.--.--.
// |  2  3  |  | 3| 4| 5|
// .-9.10.11.  .--.--.--.
// |  4  5  |  | 6| 7| 8|
// .--.--.--.  .--.--.--.
//
// 1: rotate edges 180
// .--.--.--.  .--.--.--.
// |  0  1  |  | 0| 7| 2|
// .-6.-7.-8.  .--.--.--.
// |  2  2  |  | 5| 4| 3|
// .-8.-7.-6.  .--.--.--.
// |  1  0  |  | 6| 1| 8|
// .--.--.--.  .--.--.--.
//
// 2: rotates edges 90
// .--.--.--.  .--.--.--.
// |  0  1  |  | 0| 3| 6|
// .-4.-7.-5.  .--.--.--.
// |  2  2  |  | 7| 4| 1|
// .-0.-7.-1.  .--.--.--.
// |  4  5  |  | 2| 5| 8|
// .--.--.--.  .--.--.--.
//
// 3: reflect edges on \
// .--.--.--.  .--.--.--.
// |  0  1  |  | 0| 3| 2|
// .-1.-3.-5.  .--.--.--.
// |  2  3  |  | 1| 4| 7|
// .-0.-2.-4.  .--.--.--.
// |  4  5  |  | 6| 5| 8|
// .--.--.--.  .--.--.--.
//
// 4: rotate edges 270 and reflect corners on \
// .--.--.--.  .--.--.--.
// |  0  1  |  | 0| 5| 6|
// .-1.-7.-0.  .--.--.--.
// |  2  2  |  | 1| 4| 7|
// .-5.-7.-4.  .--.--.--.
// |  4  5  |  | 2| 3| 8|
// .--.--.--.  .--.--.--.
//
// 5, 6, 7  - probably more like above

struct MatchingVars<T>(HashMap<(T, T), Lit>);
impl<T: Ord + Hash> MatchingVars<T> {
    fn new() -> Self {
        Self(HashMap::new())
    }
    fn key(a: T, b: T) -> (T, T) {
        assert!(a != b);
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }
    fn put(&mut self, a: T, b: T, var: Lit) {
        let key = Self::key(a, b);
        assert!(!self.0.contains_key(&key));
        self.0.insert(key, var);
    }
    fn get(&self, a: T, b: T) -> Option<Lit> {
        let key = Self::key(a, b);
        self.0.get(&key).copied()
    }
}
struct TableVars<T, U>(HashMap<(T, U), Lit>);
impl<T: Eq + Hash, U: Eq + Hash> TableVars<T, U> {
    fn new() -> Self {
        Self(HashMap::new())
    }
    fn put(&mut self, a: T, b: U, var: Lit) {
        let key = (a, b);
        assert!(!self.0.contains_key(&key));
        self.0.insert(key, var);
    }
    fn get(&self, a: T, b: U) -> Option<Lit> {
        let key = (a, b);
        self.0.get(&key).copied()
    }
}

struct JigsawDoubler<P> {
    puzzle: P,
    sat: SatProblem,
    edge_matching_vars: MatchingVars<EdgeKey>,
    piece_dest_vars: TableVars<PieceKey, PieceKey>,
    piece_dest_rot_vars: TableVars<PieceKey, usize>,
    piece_dest_adjacent_vars: MatchingVars<PieceKeyEdge>,
}
impl<P: Puzzle> JigsawDoubler<P> {
    pub fn run(puzzle: P) {
        let mut s = Self::new(puzzle);

        s.add_edge_matching_vars();
        s.add_piece_dest_vars();
        s.add_piece_dest_rot_vars();
        s.add_edge_one_hot_matching();
        s.add_piece_one_hot_dest();
        s.add_piece_one_hot_rot();
        s.add_piece_one_hot_src();
        s.add_piece_dest_adjacent_vars();
        s.add_piece_dest_adjacent_not_same();
        s.add_piece_dest_adjacent_to_matching();

        let mut count = 0;
        while let Some(solution) = s.sat.solve() {
            count += 1;
            println!("found solution {}", count);
            s.print_edge_matching(&solution);
            s.print_piece_dest_rot(&solution);
            println!();
            s.add_prior_solution(&solution)
        }
        println!("no more solutions, {} in total", count)

        // TODO extract result
    }
    fn new(puzzle: P) -> Self {
        Self {
            puzzle,
            sat: SatProblem::new(),
            edge_matching_vars: MatchingVars::new(),
            piece_dest_vars: TableVars::new(),
            piece_dest_rot_vars: TableVars::new(),
            piece_dest_adjacent_vars: MatchingVars::new(),
        }
    }

    fn add_edge_matching_vars(&mut self) {
        for a in EdgeKey::iter(self.puzzle.num_edges()) {
            let a_orbit: EdgeOrbitKey = self.puzzle.edge_orbit(a);
            for b in EdgeKey::iter(a.0) {
                let b_orbit = self.puzzle.edge_orbit(b);
                if a_orbit == b_orbit {
                    self.edge_matching_vars.put(a, b, self.sat.var());
                }
            }
        }
    }
    fn print_edge_matching(&self, solution: &SatSolution) {
        for a in EdgeKey::iter(self.puzzle.num_edges()) {
            println!(
                "{} matches [{}]",
                a.0,
                EdgeKey::iter(self.puzzle.num_edges())
                    .filter(|b| b != &a)
                    .filter(|b| self
                        .edge_matching_vars
                        .get(a, *b)
                        .map(|v| solution.get(v))
                        .unwrap_or(false))
                    .map(|b| b.0)
                    .format(",")
            );
        }
    }

    fn add_piece_dest_vars(&mut self) {
        for src in PieceKey::iter(self.puzzle.num_pieces()) {
            let src_orbit = self.puzzle.piece_orbit(src);
            for dest in PieceKey::iter(self.puzzle.num_pieces()) {
                let dest_orbit = self.puzzle.piece_orbit(dest);
                if src_orbit == dest_orbit {
                    self.piece_dest_vars.put(src, dest, self.sat.var());
                }
            }
        }
    }
    fn add_piece_dest_rot_vars(&mut self) {
        for src in PieceKey::iter(self.puzzle.num_pieces()) {
            let src_orbit = self.puzzle.piece_orbit(src);
            let src_info = self.puzzle.piece_orbit_info(src_orbit);
            for rot in 0..src_info.rotations {
                self.piece_dest_rot_vars.put(src, rot, self.sat.var());
            }
        }
    }
    fn print_piece_dest_rot(&self, solution: &SatSolution) {
        for src in PieceKey::iter(self.puzzle.num_pieces()) {
            let src_orbit = self.puzzle.piece_orbit(src);
            let src_info = self.puzzle.piece_orbit_info(src_orbit);

            println!(
                "{} goes to [{}] rotate [{}]",
                src.0,
                PieceKey::iter(self.puzzle.num_pieces())
                    .filter(|dest| self
                        .piece_dest_vars
                        .get(src, *dest)
                        .map(|v| solution.get(v))
                        .unwrap_or(false))
                    .map(|dest| dest.0)
                    .format(","),
                (0..src_info.rotations)
                    .filter(|rot| self
                        .piece_dest_rot_vars
                        .get(src, *rot)
                        .map(|v| solution.get(v))
                        .unwrap_or(false))
                    .format(",")
            );
        }
    }

    fn add_edge_one_hot_matching(&mut self) {
        for a in EdgeKey::iter(self.puzzle.num_edges()) {
            let a_matches = EdgeKey::iter(self.puzzle.num_edges())
                .filter(|b| *b != a)
                .filter_map(|b| self.edge_matching_vars.get(a, b))
                .collect_vec();
            self.sat.exact_count_clause(1, &a_matches);
        }
    }
    fn add_piece_one_hot_dest(&mut self) {
        for src in PieceKey::iter(self.puzzle.num_pieces()) {
            let src_dest = PieceKey::iter(self.puzzle.num_pieces())
                .filter_map(|dest| self.piece_dest_vars.get(src, dest))
                .collect_vec();
            self.sat.exact_count_clause(1, &src_dest);
        }
    }
    fn add_piece_one_hot_rot(&mut self) {
        for src in PieceKey::iter(self.puzzle.num_pieces()) {
            let src_orbit = self.puzzle.piece_orbit(src);
            let src_info = self.puzzle.piece_orbit_info(src_orbit);
            let src_dest_rot = (0..src_info.rotations)
                .map(|rot| self.piece_dest_rot_vars.get(src, rot).unwrap())
                .collect_vec();
            self.sat.exact_count_clause(1, &src_dest_rot);
        }
    }
    fn add_piece_one_hot_src(&mut self) {
        for dest in PieceKey::iter(self.puzzle.num_pieces()) {
            let dest_src = PieceKey::iter(self.puzzle.num_pieces())
                .filter_map(|src| self.piece_dest_vars.get(src, dest))
                .collect_vec();
            self.sat.exact_count_clause(1, &dest_src);
        }
    }

    fn add_piece_dest_adjacent_vars(&mut self) {
        fn rotation_from_edge(
            info: PieceOrbitInfo,
            src_edge: usize,
            dest_edge: usize,
        ) -> Option<usize> {
            // TODO don't do this the dumb way
            for rot in 0..info.rotations {
                if (src_edge + rot * info.edge_increment_per_rotation) % info.edges == dest_edge {
                    return Some(rot);
                }
            }
            // edge pieces can't rotate but their edges are in the same orbit
            None
        }

        // loop over all PieceKeyEdge pairs who's edge share an orbit
        for a_piece in PieceKey::iter(self.puzzle.num_pieces()) {
            let a_orbit = self.puzzle.piece_orbit(a_piece);
            let a_info = self.puzzle.piece_orbit_info(a_orbit);
            for a_piece_edge in a_piece.edges(a_info.edges) {
                let a_edge = self.puzzle.piece_edge(a_piece_edge);
                let a_edge_orbit = self.puzzle.edge_orbit(a_edge);
                // iter over b_pieces that are <= a_pieces
                for b_piece in PieceKey::iter(a_piece.0 + 1) {
                    let b_orbit = self.puzzle.piece_orbit(b_piece);
                    let b_info = self.puzzle.piece_orbit_info(b_orbit);
                    for b_piece_edge in b_piece.edges(b_info.edges) {
                        let b_edge = self.puzzle.piece_edge(b_piece_edge);
                        let b_edge_orbit = self.puzzle.edge_orbit(b_edge);
                        if a_piece_edge > b_piece_edge && a_edge_orbit == b_edge_orbit {
                            // loop over all possible destination edges including parity for which side a_piece_dest is on
                            let dest_adj = EdgeKey::iter(self.puzzle.num_edges())
                                .filter(|dest_edge| {
                                    self.puzzle.edge_orbit(*dest_edge) == a_edge_orbit
                                })
                                .map(|dest_edge| self.puzzle.edge_pieces(dest_edge))
                                .flat_map(|[a_dest, b_dest]| [[a_dest, b_dest], [b_dest, a_dest]])
                                .filter(|[a_dest, b_dest]| {
                                    self.puzzle.piece_orbit(a_dest.piece) == a_orbit
                                        && self.puzzle.piece_orbit(b_dest.piece) == b_orbit
                                })
                                // `and` the required vars for a_piece_edge and b_piece_edge to be adjacent at dest_edge
                                .filter_map(|[a_dest, b_dest]| {
                                    Some([
                                        self.piece_dest_vars.get(a_piece, a_dest.piece).unwrap(),
                                        self.piece_dest_rot_vars
                                            .get(
                                                a_piece,
                                                rotation_from_edge(
                                                    a_info,
                                                    a_piece_edge.edge,
                                                    a_dest.edge,
                                                )?,
                                            )
                                            .unwrap(),
                                        self.piece_dest_vars.get(b_piece, b_dest.piece).unwrap(),
                                        self.piece_dest_rot_vars
                                            .get(
                                                b_piece,
                                                rotation_from_edge(
                                                    b_info,
                                                    b_piece_edge.edge,
                                                    b_dest.edge,
                                                )?,
                                            )
                                            .unwrap(),
                                    ])
                                })
                                .map(|vars| self.sat.and_var(&vars))
                                .collect_vec();

                            // `or` over all possibilities for a_piece_edge and b_piece_edge to be adjacent
                            self.piece_dest_adjacent_vars.put(
                                a_piece_edge,
                                b_piece_edge,
                                self.sat.or_var(&dest_adj),
                            );
                        }
                    }
                }
            }
        }
    }

    fn add_piece_dest_adjacent_not_same(&mut self) {
        for edge in EdgeKey::iter(self.puzzle.num_edges()) {
            let [a, b] = self.puzzle.edge_pieces(edge);
            self.sat
                .not_clause(self.piece_dest_adjacent_vars.get(a, b).unwrap());
        }
    }

    fn add_piece_dest_adjacent_to_matching(&mut self) {
        for edge_a in EdgeKey::iter(self.puzzle.num_edges()) {
            let [piece_a1, piece_a2] = self.puzzle.edge_pieces(edge_a);

            for edge_b in EdgeKey::iter(edge_a.0) {
                if let Some(edge_matches) = self.edge_matching_vars.get(edge_a, edge_b) {
                    let [piece_b1, piece_b2] = self.puzzle.edge_pieces(edge_b);

                    // edges match implies destination pieces trade neighbors
                    let case_1 = self.sat.and_var(&[
                        self.piece_dest_adjacent_vars
                            .get(piece_a1, piece_b1)
                            .unwrap(),
                        self.piece_dest_adjacent_vars
                            .get(piece_a2, piece_b2)
                            .unwrap(),
                    ]);
                    let case_2 = self.sat.and_var(&[
                        self.piece_dest_adjacent_vars
                            .get(piece_a1, piece_b2)
                            .unwrap(),
                        self.piece_dest_adjacent_vars
                            .get(piece_a2, piece_b1)
                            .unwrap(),
                    ]);
                    let dest_matches = self.sat.or_var(&[case_1, case_2]);
                    self.sat.implies_clause(edge_matches, dest_matches);
                }
            }
        }
    }

    fn add_prior_solution(&mut self, solution: &SatSolution) {
        let differ_vars = self
            .edge_matching_vars
            .0
            .values()
            .map(|&v| if solution.get(v) { !v } else { v })
            .collect_vec();
        self.sat.or_clause(&differ_vars);
        // TODO add clause for each global rotation and mirror
    }
}
