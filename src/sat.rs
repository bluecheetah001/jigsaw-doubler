use std::collections::HashSet;

use itertools::Itertools;
use varisat::cnf::CnfFormula;
use varisat::{ExtendFormula, Lit, Solver, Var};

pub struct SatProblem {
    cnf: Solver<'static>,
}
impl SatProblem {
    pub fn new() -> Self {
        Self { cnf: Solver::new() }
    }
    pub fn solve(&mut self) -> Option<SatSolution> {
        // default configuration of Solver should never fail
        let satisfiable = self.cnf.solve().unwrap();
        if satisfiable {
            Some(SatSolution::new(&self.cnf.model().unwrap()))
        } else {
            None
        }
    }

    /// allocate a new var
    pub fn var(&mut self) -> Lit {
        self.cnf.new_var().positive()
    }
    // pub fn printed_var(&mut self, name: &str) -> Lit {
    //     let var = self.var();
    //     println!("{} := {}", var, name);
    //     var
    // }
    /// add a new CNF clause
    pub fn or_clause(&mut self, vars: &[Lit]) {
        if vars.is_empty() {
            panic!("trivially not satisfiable")
        }
        self.cnf.add_clause(vars);
    }
    pub fn nor_clause(&mut self, vars: &[Lit]) {
        let not_vars = vars.iter().map(|v| !*v).collect_vec();
        self.and_clause(&not_vars);
    }
    pub fn and_clause(&mut self, vars: &[Lit]) {
        for v in vars {
            self.or_clause(&[*v])
        }
    }
    pub fn nand_clause(&mut self, vars: &[Lit]) {
        let not_vars = vars.iter().map(|v| !*v).collect_vec();
        self.or_clause(&not_vars);
    }
    pub fn not_clause(&mut self, var: Lit) {
        self.or_clause(&[!var])
    }
    pub fn implies_clause(&mut self, a: Lit, b: Lit) {
        self.or_clause(&[!a, b])
    }

    pub fn exact_count_clause(&mut self, count: usize, vars: &[Lit]) {
        // short-circuit / optimize a few obvious edge cases
        if count == 0 {
            self.nor_clause(vars);
        } else if count == vars.len() {
            self.and_clause(vars);
        } else if count > vars.len() {
            panic!("trivially not satisfiable")
        } else {
            let count_greater_than = self.count_up_to_vars(count + 1, vars);
            self.and_clause(&[count_greater_than[count - 1], !count_greater_than[count]]);
        }
    }

    // tseytin transform
    pub fn and_var(&mut self, vars: &[Lit]) -> Lit {
        let result = self.var();
        // println!("{} := and({})", result, vars.iter().format(","));

        // result => v
        for v in vars.iter() {
            self.implies_clause(result, *v);
        }
        // and(vars) => result
        let clause = vars
            .iter()
            .map(|v| !*v)
            .chain(std::iter::once(result))
            .collect_vec();
        self.or_clause(&clause);

        result
    }
    pub fn or_var(&mut self, vars: &[Lit]) -> Lit {
        let result = self.var();
        // println!("{} := or({})", result, vars.iter().format(","));

        // v => result
        for v in vars.iter() {
            self.implies_clause(*v, result);
        }
        // result => or(vars)
        let clause = std::iter::once(!result)
            .chain(vars.iter().copied())
            .collect_vec();
        self.or_clause(&clause);

        result
    }
    pub fn xor_var(&mut self, a: Lit, b: Lit) -> Lit {
        let result = self.var();
        // println!("{} := xor({},{})", result, a, b);
        self.or_clause(&[!a, !b, !result]);
        self.or_clause(&[a, b, !result]);
        self.or_clause(&[a, !b, result]);
        self.or_clause(&[!a, b, result]);
        result
    }
    pub fn eq_var(&mut self, a: Lit, b: Lit) -> Lit {
        self.xor_var(a, !b)
    }
    fn count_up_to_vars(&mut self, up_to: usize, vars: &[Lit]) -> Vec<Lit> {
        let mut prior = vec![];
        if up_to == 0 {
            return prior;
        }
        for v in vars {
            let row_len = up_to.min(prior.len() + 1);
            prior = (0..row_len)
                .into_iter()
                .map(|c| self.count_var(&prior, *v, c))
                .collect_vec();
        }
        prior
    }
    fn count_var(
        &mut self,
        prior_greater_than: &[Lit],
        var: Lit,
        count_greater_than: usize,
    ) -> Lit {
        debug_assert!(count_greater_than <= prior_greater_than.len());

        if count_greater_than == 0 {
            if let Some(prior) = prior_greater_than.get(0) {
                // prior | var
                return self.or_var(&[*prior, var]);
            } else {
                // no prior, so must be first iteration
                return var;
            }
        }
        if count_greater_than == prior_greater_than.len() {
            // prior_minus_one & var
            return self.and_var(&[prior_greater_than[count_greater_than - 1], var]);
        }

        let result = self.var();
        let prior_minus_one = prior_greater_than[count_greater_than - 1];
        let prior = prior_greater_than[count_greater_than];
        // println!(
        //     "{} := [{}] + {} > {}",
        //     result,
        //     prior_greater_than.iter().format(","),
        //     var,
        //     count_greater_than
        // );

        // r == (m&v) | p
        // r => (m&v) | p & (m&v) | p => r
        // !r | (m&v) | p & !((m&v) | p) | r
        // !r | (m&v) | p & (!(m&v) & !p) | r
        // !r | (m&v) | p & ((!m | !v) & !p) | r
        // !r | (m&v) | p & (!m&!p) | (!v&!p) | r
        // !r | (m&v) | p & (!m&!p) | (!v&!p) | r
        // !r|m|p & !r|v|p & !m|!v|r & !m|!p|r & !p|!v|r & !p|!p|r
        // !r|m|p & !r|v|p & r|!m|!v & r|!m|!p & r|!v|!p & r|!p
        // remove r|!m|!p , r|!v|!p because they imply r|!p
        self.or_clause(&[!result, prior_minus_one, prior]);
        self.or_clause(&[!result, prior, var]);
        self.or_clause(&[!prior_minus_one, !var, result]);
        self.or_clause(&[!prior, result]);

        result
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SatSolution {
    true_vars: HashSet<Lit>,
}
impl SatSolution {
    pub fn new(vars: &[Lit]) -> Self {
        let true_vars = vars
            .into_iter()
            .copied()
            .filter(|v| v.is_positive())
            .collect();
        Self { true_vars }
    }
    pub fn get(&self, var: Lit) -> bool {
        if var.is_positive() {
            self.true_vars.contains(&var)
        } else {
            !self.true_vars.contains(&!var)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_all_solutions(mut p: SatProblem, vars: Vec<Lit>, mut solutions: Vec<SatSolution>) {
        // while expecting more solutions
        while !solutions.is_empty() {
            let solution = p.solve();
            if let Some(solution) = solution {
                let i = solutions.iter().position(|s| s == &solution);
                // remove solutions that were expected
                if let Some(i) = i {
                    solutions.swap_remove(i);

                    exclude_solution(&mut p, &vars, &solution);
                } else {
                    panic!("unexpected solution {solution:?} given vars {vars:?}");
                }
            } else {
                panic!("missing solutions {solutions:?} given vars {vars:?}");
            }
        }

        // verify no other solutions
        let solution = p.solve();
        if let Some(solution) = solution {
            panic!("unexpected solution {solution:?} given vars {vars:?}");
        }
    }
    fn assert_all_solutions_ignore_hidden(
        mut p: SatProblem,
        vars: Vec<Lit>,
        mut solutions: Vec<SatSolution>,
    ) {
        // while expecting more solutions
        while !solutions.is_empty() {
            let solution = p.solve();
            if let Some(solution) = solution {
                let solution = sub_solution(&solution, &vars);
                let i = solutions.iter().position(|s| s == &solution);
                // remove solutions that were expected
                if let Some(i) = i {
                    solutions.swap_remove(i);

                    exclude_solution(&mut p, &vars, &solution);
                } else {
                    panic!("unexpected solution {solution:?} given vars {vars:?}");
                }
            } else {
                panic!("missing solutions {solutions:?} given vars {vars:?}");
            }
        }

        // verify no other solutions
        let solution = p.solve();
        if let Some(solution) = solution {
            panic!("unexpected solution {solution:?} given vars {vars:?}");
        }
    }
    fn exclude_solution(p: &mut SatProblem, vars: &[Lit], solution: &SatSolution) {
        let clause = vars
            .iter()
            .copied()
            .map(|v| if solution.get(v) { !v } else { v })
            .collect_vec();
        p.or_clause(&clause)
    }
    fn sub_solution(solution: &SatSolution, vars: &[Lit]) -> SatSolution {
        let true_vars = vars
            .iter()
            .copied()
            .filter(|v| solution.get(*v))
            .collect_vec();
        SatSolution::new(&true_vars)
    }

    fn print_all_solutions(mut p: SatProblem) {
        while print_and_exclude_next_solution(&mut p) {}
    }
    fn print_and_exclude_next_solution(p: &mut SatProblem) -> bool {
        let satisfiable = p.cnf.solve().unwrap();
        if satisfiable {
            let vars = p.cnf.model().unwrap();
            println!(
                "solution [{}]",
                vars.iter().filter(|v| v.is_positive()).format(",")
            );
            p.nand_clause(&vars);
        }
        satisfiable
    }

    #[test]
    #[should_panic(expected = "trivially not satisfiable")]
    fn or_clause_0() {
        let mut p = SatProblem::new();
        p.or_clause(&[]);

        assert_all_solutions(p, vec![], vec![]);
    }

    #[test]
    fn or_clause_1() {
        let mut p = SatProblem::new();
        let a = p.var();
        p.or_clause(&[a]);

        assert_all_solutions(p, vec![a], vec![SatSolution::new(&[a])]);
    }

    #[test]
    fn or_clause_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.or_clause(&[a, b]);

        assert_all_solutions(
            p,
            vec![a, b],
            vec![
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[a, b]),
            ],
        );
    }

    #[test]
    fn or_clause_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.or_clause(&[a, b, c]);

        assert_all_solutions(
            p,
            vec![a, b, c],
            vec![
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[c]),
                SatSolution::new(&[a, b]),
                SatSolution::new(&[a, c]),
                SatSolution::new(&[b, c]),
                SatSolution::new(&[a, b, c]),
            ],
        );
    }

    #[test]
    fn and_clause_0() {
        let mut p = SatProblem::new();
        p.and_clause(&[]);

        // can't use assert_all_solutions because that triggers "trivially not satisfiable"
        let expected = Some(SatSolution::new(&[]));
        let solution = p.solve();
        assert_eq!(expected, solution);
    }

    #[test]
    fn and_clause_1() {
        let mut p = SatProblem::new();
        let a = p.var();
        p.and_clause(&[a]);

        assert_all_solutions(p, vec![a], vec![SatSolution::new(&[a])]);
    }

    #[test]
    fn and_clause_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.and_clause(&[a, b]);

        assert_all_solutions(p, vec![a, b], vec![SatSolution::new(&[a, b])]);
    }

    #[test]
    fn and_clause_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.and_clause(&[a, b, c]);

        assert_all_solutions(p, vec![a, b, c], vec![SatSolution::new(&[a, b, c])]);
    }

    #[test]
    fn nor_clause() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.nor_clause(&[a, b]);

        assert_all_solutions(p, vec![a, b], vec![SatSolution::new(&[])]);
    }

    #[test]
    fn nand_clause() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.nand_clause(&[a, b]);

        assert_all_solutions(
            p,
            vec![a, b],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
            ],
        );
    }

    #[test]
    fn not_clause() {
        let mut p = SatProblem::new();
        let a = p.var();
        p.not_clause(a);

        assert_all_solutions(p, vec![a], vec![SatSolution::new(&[])]);
    }

    #[test]
    fn implies_clause() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.implies_clause(a, b);

        assert_all_solutions(
            p,
            vec![a, b],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[b]),
                SatSolution::new(&[a, b]),
            ],
        );
    }

    #[test]
    fn exact_count_clause_0_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.exact_count_clause(0, &[a, b]);

        assert_all_solutions_ignore_hidden(p, vec![a, b], vec![SatSolution::new(&[])]);
    }

    #[test]
    fn exact_count_clause_1_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.exact_count_clause(1, &[a, b]);

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b],
            vec![SatSolution::new(&[a]), SatSolution::new(&[b])],
        );
    }

    #[test]
    fn exact_count_clause_2_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.exact_count_clause(2, &[a, b]);

        assert_all_solutions_ignore_hidden(p, vec![a, b], vec![SatSolution::new(&[a, b])]);
    }

    #[test]
    #[should_panic(expected = "trivially not satisfiable")]
    fn exact_count_clause_3_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        p.exact_count_clause(3, &[a, b]);
    }

    #[test]
    fn exact_count_clause_0_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.exact_count_clause(0, &[a, b, c]);

        assert_all_solutions_ignore_hidden(p, vec![a, b, c], vec![SatSolution::new(&[])]);
    }

    #[test]
    fn exact_count_clause_1_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.exact_count_clause(1, &[a, b, c]);

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b, c],
            vec![
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[c]),
            ],
        );
    }

    #[test]
    fn exact_count_clause_2_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.exact_count_clause(2, &[a, b, c]);

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b, c],
            vec![
                SatSolution::new(&[a, b]),
                SatSolution::new(&[a, c]),
                SatSolution::new(&[b, c]),
            ],
        );
    }

    #[test]
    fn exact_count_clause_3_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        p.exact_count_clause(3, &[a, b, c]);

        assert_all_solutions_ignore_hidden(p, vec![a, b, c], vec![SatSolution::new(&[a, b, c])]);
    }

    #[test]
    fn and_var_0() {
        let mut p = SatProblem::new();
        let r = p.and_var(&[]);

        assert_all_solutions(p, vec![r], vec![SatSolution::new(&[r])]);
    }

    #[test]
    fn and_var_1() {
        let mut p = SatProblem::new();
        let a = p.var();
        let r = p.and_var(&[a]);

        assert_all_solutions(
            p,
            vec![a, r],
            vec![SatSolution::new(&[]), SatSolution::new(&[a, r])],
        );
    }

    #[test]
    fn and_var_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let r = p.and_var(&[a, b]);

        assert_all_solutions(
            p,
            vec![a, b, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[a, b, r]),
            ],
        );
    }

    #[test]
    fn and_var_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        let r = p.and_var(&[a, b, c]);

        assert_all_solutions(
            p,
            vec![a, b, c, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[c]),
                SatSolution::new(&[a, b]),
                SatSolution::new(&[a, c]),
                SatSolution::new(&[b, c]),
                SatSolution::new(&[a, b, c, r]),
            ],
        );
    }

    #[test]
    fn or_var_0() {
        let mut p = SatProblem::new();
        let r = p.or_var(&[]);

        assert_all_solutions(p, vec![r], vec![SatSolution::new(&[])]);
    }

    #[test]
    fn or_var_1() {
        let mut p = SatProblem::new();
        let a = p.var();
        let r = p.or_var(&[a]);

        assert_all_solutions(
            p,
            vec![a, r],
            vec![SatSolution::new(&[]), SatSolution::new(&[a, r])],
        );
    }

    #[test]
    fn or_var_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let r = p.or_var(&[a, b]);

        assert_all_solutions(
            p,
            vec![a, b, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, r]),
                SatSolution::new(&[b, r]),
                SatSolution::new(&[a, b, r]),
            ],
        );
    }

    #[test]
    fn or_var_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        let r = p.or_var(&[a, b, c]);

        assert_all_solutions(
            p,
            vec![a, b, c, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, r]),
                SatSolution::new(&[b, r]),
                SatSolution::new(&[c, r]),
                SatSolution::new(&[a, b, r]),
                SatSolution::new(&[a, c, r]),
                SatSolution::new(&[b, c, r]),
                SatSolution::new(&[a, b, c, r]),
            ],
        );
    }

    #[test]
    fn xor_var() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let r = p.xor_var(a, b);

        assert_all_solutions(
            p,
            vec![a, b, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, r]),
                SatSolution::new(&[b, r]),
                SatSolution::new(&[a, b]),
            ],
        );
    }

    #[test]
    fn eq_var() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let r = p.eq_var(a, b);

        assert_all_solutions(
            p,
            vec![a, b, r],
            vec![
                SatSolution::new(&[r]),
                SatSolution::new(&[a]),
                SatSolution::new(&[b]),
                SatSolution::new(&[a, b, r]),
            ],
        );
    }

    #[test]
    fn count_var_0_0() {
        let mut p = SatProblem::new();
        let v = p.var();
        let prior = [];
        let r = p.count_var(&prior, v, 0);

        assert_all_solutions(
            p,
            vec![v, r],
            vec![SatSolution::new(&[]), SatSolution::new(&[v, r])],
        );
    }

    #[test]
    fn count_var_1_0() {
        let mut p = SatProblem::new();
        let g0 = p.var();
        let v = p.var();
        let prior = [g0];
        let r = p.count_var(&prior, v, 0);

        assert_all_solutions(
            p,
            vec![g0, v, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[g0, r]),
                SatSolution::new(&[v, r]),
                SatSolution::new(&[g0, v, r]),
            ],
        );
    }

    #[test]
    fn count_var_1_1() {
        let mut p = SatProblem::new();
        let g0 = p.var();
        let v = p.var();
        let prior = [g0];
        let r = p.count_var(&prior, v, 1);

        assert_all_solutions(
            p,
            vec![g0, v, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[g0]),
                SatSolution::new(&[v]),
                SatSolution::new(&[g0, v, r]),
            ],
        );
    }

    #[test]
    fn count_var_2_0() {
        let mut p = SatProblem::new();
        let g0 = p.var();
        let g1 = p.var();
        let v = p.var();
        // implications are an assumption of prior
        p.implies_clause(g1, g0);
        let prior = [g0, g1];
        let r = p.count_var(&prior, v, 0);

        assert_all_solutions(
            p,
            vec![g0, g1, v, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[g0, r]),
                SatSolution::new(&[g0, g1, r]),
                SatSolution::new(&[v, r]),
                SatSolution::new(&[g0, v, r]),
                SatSolution::new(&[g0, g1, v, r]),
            ],
        );
    }

    #[test]
    fn count_var_2_1() {
        let mut p = SatProblem::new();
        let g0 = p.var();
        let g1 = p.var();
        let v = p.var();
        // implications are an assumption of prior
        p.implies_clause(g1, g0);
        let prior = [g0, g1];
        let r = p.count_var(&prior, v, 1);

        assert_all_solutions(
            p,
            vec![g0, g1, v, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[g0]),
                SatSolution::new(&[g0, g1, r]),
                SatSolution::new(&[v]),
                SatSolution::new(&[g0, v, r]),
                SatSolution::new(&[g0, g1, v, r]),
            ],
        );
    }

    #[test]
    fn count_var_2_2() {
        let mut p = SatProblem::new();
        let g0 = p.var();
        let g1 = p.var();
        let v = p.var();
        // implications are an assumption of prior
        p.implies_clause(g1, g0);
        let prior = [g0, g1];
        let r = p.count_var(&prior, v, 2);

        assert_all_solutions(
            p,
            vec![g0, g1, v, r],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[g0]),
                SatSolution::new(&[g0, g1]),
                SatSolution::new(&[v]),
                SatSolution::new(&[g0, v]),
                SatSolution::new(&[g0, g1, v, r]),
            ],
        );
    }

    #[test]
    fn count_up_to_vars_2_1() {
        let mut p = SatProblem::new();
        let a = p.var();
        let [g0] = p.count_up_to_vars(2, &[a]).try_into().unwrap();

        assert_all_solutions(
            p,
            vec![a, g0],
            vec![SatSolution::new(&[]), SatSolution::new(&[a, g0])],
        );
    }

    #[test]
    fn count_up_to_vars_2_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let [g0, g1] = p.count_up_to_vars(2, &[a, b]).try_into().unwrap();

        assert_all_solutions(
            p,
            vec![a, b, g0, g1],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, g0]),
                SatSolution::new(&[b, g0]),
                SatSolution::new(&[a, b, g0, g1]),
            ],
        );
    }

    #[test]
    fn count_up_to_vars_2_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        let [g0, g1] = p.count_up_to_vars(2, &[a, b, c]).try_into().unwrap();

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b, c, g0, g1],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, g0]),
                SatSolution::new(&[b, g0]),
                SatSolution::new(&[c, g0]),
                SatSolution::new(&[a, b, g0, g1]),
                SatSolution::new(&[a, c, g0, g1]),
                SatSolution::new(&[b, c, g0, g1]),
                SatSolution::new(&[a, b, c, g0, g1]),
            ],
        );
    }

    #[test]
    fn count_up_to_vars_3_2() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let [g0, g1] = p.count_up_to_vars(3, &[a, b]).try_into().unwrap();

        assert_all_solutions(
            p,
            vec![a, b, g0, g1],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, g0]),
                SatSolution::new(&[b, g0]),
                SatSolution::new(&[a, b, g0, g1]),
            ],
        );
    }

    #[test]
    fn count_up_to_vars_3_3() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        let [g0, g1, g2] = p.count_up_to_vars(3, &[a, b, c]).try_into().unwrap();

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b, c, g0, g1, g2],
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, g0]),
                SatSolution::new(&[b, g0]),
                SatSolution::new(&[c, g0]),
                SatSolution::new(&[a, b, g0, g1]),
                SatSolution::new(&[a, c, g0, g1]),
                SatSolution::new(&[b, c, g0, g1]),
                SatSolution::new(&[a, b, c, g0, g1, g2]),
            ],
        );
    }

    #[test]
    fn count_up_to_vars_3_4() {
        let mut p = SatProblem::new();
        let a = p.var();
        let b = p.var();
        let c = p.var();
        let d = p.var();
        let [g0, g1, g2] = p.count_up_to_vars(3, &[a, b, c, d]).try_into().unwrap();

        // print_all_solutions(p);

        assert_all_solutions_ignore_hidden(
            p,
            vec![a, b, c, d, g0, g1, g2],
            // g1 d g0 b c
            vec![
                SatSolution::new(&[]),
                SatSolution::new(&[a, g0]),
                SatSolution::new(&[b, g0]),
                SatSolution::new(&[c, g0]),
                SatSolution::new(&[d, g0]),
                SatSolution::new(&[a, b, g0, g1]),
                SatSolution::new(&[a, c, g0, g1]),
                SatSolution::new(&[a, d, g0, g1]),
                SatSolution::new(&[b, c, g0, g1]),
                SatSolution::new(&[b, d, g0, g1]),
                SatSolution::new(&[c, d, g0, g1]),
                SatSolution::new(&[a, b, c, g0, g1, g2]),
                SatSolution::new(&[a, b, d, g0, g1, g2]),
                SatSolution::new(&[a, c, d, g0, g1, g2]),
                SatSolution::new(&[b, c, d, g0, g1, g2]),
                SatSolution::new(&[a, b, c, d, g0, g1, g2]),
            ],
        );
    }
}
