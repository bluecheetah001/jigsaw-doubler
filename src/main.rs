use std::collections::HashMap;
use std::hash::Hash;
use std::time::Instant;

mod puzzle;
mod sat;
use itertools::Itertools;
use puzzle::*;
use sat::*;
use varisat::Lit;

fn main() {
    let start_time = Instant::now();
    let puzzle = SquarePuzzle::new(5, 5);
    JigsawDoubler::run(puzzle, start_time);
}

struct MatchingVars<T>(HashMap<(T, T), Lit>);
impl<T: Ord + Hash> MatchingVars<T> {
    fn new() -> Self {
        Self(HashMap::new())
    }
    fn key(a: T, b: T) -> (T, T) {
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
    point_dest_vars: TableVars<PointKey, PointKey>,
    point_dest_adjacent_vars: MatchingVars<PointKey>,
    edge_matching_vars: MatchingVars<EdgeKey>,
}
impl<P: Puzzle> JigsawDoubler<P> {
    pub fn run(puzzle: P, start_time: Instant) {
        let mut s = Self::new(puzzle);

        s.add_point_dest_vars();
        s.add_one_hot_point_dest();
        s.add_one_hot_point_src();
        s.add_point_dest_adjacent_vars();
        s.add_point_dest_adjacent_not_same();
        s.add_edge_matching_vars();
        s.add_one_hot_edge_matching();

        println!("constraints configured, starting solve");
        let mut last = start_time;
        let mut count = 0;
        while let Some(solution) = s.sat.solve() {
            let now = Instant::now();
            count += 1;
            println!(
                "found solution {} in {} ({} total)",
                count,
                humantime::format_duration(now - last),
                humantime::format_duration(now - start_time)
            );
            s.print_point_dest(&solution);
            s.print_edge_matching(&solution);
            println!();
            s.add_prior_solution(&solution);
            last = now;
        }

        let now = Instant::now();
        println!(
            "no more solutions. found {} solutions in {}",
            count,
            humantime::format_duration(now - start_time)
        )
    }
    fn new(puzzle: P) -> Self {
        Self {
            puzzle,
            sat: SatProblem::new(),
            point_dest_vars: TableVars::new(),
            point_dest_adjacent_vars: MatchingVars::new(),
            edge_matching_vars: MatchingVars::new(),
        }
    }

    fn add_point_dest_vars(&mut self) {
        // create var for each src_piece dest_point pair
        for src_piece in puzzle_pieces(&self.puzzle) {
            let src_point = self.puzzle.arbitrary_point_on_piece(src_piece);
            for dest_point in puzzle_exchange_points(&self.puzzle, src_point) {
                let var = self.sat.var();

                // and write to all implied src_point dest_point pairs
                let mut src_point_other = src_point;
                let mut dest_point_other = dest_point;
                loop {
                    self.point_dest_vars
                        .put(src_point_other, dest_point_other, var);
                    src_point_other = self.puzzle.next_point_on_piece(src_point_other);
                    dest_point_other = self.puzzle.next_point_on_piece(dest_point_other);
                    if src_point_other == src_point {
                        debug_assert_eq!(dest_point_other, dest_point);
                        break;
                    }
                }
            }
        }
    }
    fn add_one_hot_point_dest(&mut self) {
        for src_piece in puzzle_pieces(&self.puzzle) {
            let src_point = self.puzzle.arbitrary_point_on_piece(src_piece);
            let dest_vars = puzzle_points(&self.puzzle)
                .filter_map(|dest_point| self.point_dest_vars.get(src_point, dest_point))
                .collect_vec();
            self.sat.exact_count_clause(1, &dest_vars);
        }
    }
    fn add_one_hot_point_src(&mut self) {
        for dest_piece in puzzle_pieces(&self.puzzle) {
            let dest_point = self.puzzle.arbitrary_point_on_piece(dest_piece);
            let src_vars = puzzle_points(&self.puzzle)
                .filter_map(|src_point| self.point_dest_vars.get(src_point, dest_point))
                .collect_vec();
            self.sat.exact_count_clause(1, &src_vars);
        }
    }

    fn add_point_dest_adjacent_vars(&mut self) {
        for (src_point_a, src_point_b) in puzzle_point_pairs(&self.puzzle) {
            let dest_adj_vars = puzzle_points(&self.puzzle)
                .filter_map(|dest_point_a| {
                    let var_a = self.point_dest_vars.get(src_point_a, dest_point_a)?;
                    let dest_point_b = self.puzzle.other_point_on_edge(dest_point_a);
                    let var_b = self.point_dest_vars.get(src_point_b, dest_point_b)?;
                    Some(self.sat.and_var(&[var_a, var_b]))
                })
                .collect_vec();
            let dest_adj_var = self.sat.or_var(&dest_adj_vars);
            self.point_dest_adjacent_vars
                .put(src_point_a, src_point_b, dest_adj_var);
        }
    }
    fn add_point_dest_adjacent_not_same(&mut self) {
        for edge in puzzle_edges(&self.puzzle) {
            let point_1 = self.puzzle.arbitrary_point_on_edge(edge);
            let point_2 = self.puzzle.other_point_on_edge(point_1);
            let stays_adjacent_var = self.point_dest_adjacent_vars.get(point_1, point_2).unwrap();
            self.sat.not_clause(stays_adjacent_var);
        }
    }

    fn add_edge_matching_vars(&mut self) {
        for (edge_a, edge_b) in puzzle_edge_pairs(&self.puzzle) {
            let point_a1 = self.puzzle.arbitrary_point_on_edge(edge_a);
            let point_a2 = self.puzzle.other_point_on_edge(point_a1);
            let point_b1 = self.puzzle.arbitrary_point_on_edge(edge_b);
            let point_b2 = self.puzzle.other_point_on_edge(point_b1);
            let matching_vars = [
                self.point_dest_adjacent_vars.get(point_a1, point_b1),
                self.point_dest_adjacent_vars.get(point_a1, point_b2),
                self.point_dest_adjacent_vars.get(point_a2, point_b1),
                self.point_dest_adjacent_vars.get(point_a2, point_b2),
            ]
            .into_iter()
            .filter_map(|v| v)
            .collect_vec();
            if !matching_vars.is_empty() {
                let matching_var = self.sat.or_var(&matching_vars);
                self.edge_matching_vars.put(edge_a, edge_b, matching_var);
            }
        }
    }
    fn add_one_hot_edge_matching(&mut self) {
        for edge_a in puzzle_edges(&self.puzzle) {
            let matching_vars = puzzle_edges(&self.puzzle)
                .filter_map(|edge_b| self.edge_matching_vars.get(edge_a, edge_b))
                .collect_vec();
            self.sat.exact_count_clause(1, &matching_vars);
        }
    }

    fn print_point_dest(&self, solution: &SatSolution) {
        println!(
            "piece dest: {}",
            puzzle_pieces(&self.puzzle)
                .map(|src_piece| self.puzzle.arbitrary_point_on_piece(src_piece))
                .flat_map(|src_point| puzzle_points(&self.puzzle)
                    .map(move |dest_point| (src_point, dest_point)))
                .filter(|&(src_point, dest_point)| {
                    self.point_dest_vars
                        .get(src_point, dest_point)
                        .map(|var| solution.get(var))
                        .unwrap_or(false)
                })
                .map(|(edge_a, edge_b)| format!(
                    "{}=>{}",
                    self.puzzle.format_point(edge_a),
                    self.puzzle.format_point(edge_b)
                ))
                .format(" ")
        )
    }
    fn print_edge_matching(&self, solution: &SatSolution) {
        println!(
            "edge matching: {}",
            puzzle_edge_pairs(&self.puzzle)
                .filter(|&(edge_a, edge_b)| {
                    self.edge_matching_vars
                        .get(edge_a, edge_b)
                        .map(|var| solution.get(var))
                        .unwrap_or(false)
                })
                .map(|(edge_a, edge_b)| (edge_b, edge_a)) // flip order to have smaller edge first
                .sorted()
                .map(|(edge_a, edge_b)| format!(
                    "{}={}",
                    self.puzzle.format_edge(edge_a),
                    self.puzzle.format_edge(edge_b)
                ))
                .format(" ")
        );
    }

    fn add_prior_solution(&mut self, solution: &SatSolution) {
        let point_dest_vars = &self.point_dest_vars;
        let differ_vars = puzzle_pieces(&self.puzzle)
            .map(|src_piece| self.puzzle.arbitrary_point_on_piece(src_piece))
            .flat_map(|src_point| {
                puzzle_points(&self.puzzle)
                    .filter_map(move |dest_point| point_dest_vars.get(src_point, dest_point))
            })
            .map(|var| if solution.get(var) { !var } else { var })
            .collect_vec();
        self.sat.or_clause(&differ_vars);
    }
}
