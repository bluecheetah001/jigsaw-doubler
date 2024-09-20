use std::collections::HashSet;

use itertools::Itertools;
use varisat::cnf::CnfFormula;
use varisat::{ExtendFormula, Lit, Solver, Var};

#[derive(Debug)]
pub struct SatProblem {
    cnf: CnfFormula,
}
impl SatProblem {
    pub fn new() -> Self {
        Self {
            cnf: CnfFormula::new(),
        }
    }
    pub fn solve(&self) -> SatSolution {
        let mut solver = Solver::new();
        solver.add_formula(&self.cnf);
        let satisfiable = solver.solve().unwrap();
        if satisfiable {
            SatSolution::new(solver.model().unwrap())
        } else {
            panic!("not satisfiable")
        }
    }

    /// allocate a new var
    pub fn var(&mut self) -> Lit {
        self.cnf.new_var().positive()
    }
    /// add a new CNF clause
    pub fn or_clause(&mut self, vars: &[Lit]) {
        self.cnf.add_clause(vars);
    }
    pub fn nor_clause(&mut self, vars: &[Lit]) {
        let vars = vars.iter().map(|v| !*v).collect_vec();
        self.and_clause(&vars);
    }
    pub fn and_clause(&mut self, vars: &[Lit]) {
        for v in vars {
            self.or_clause(&[*v])
        }
    }
    pub fn nand_clause(&mut self, vars: &[Lit]) {
        let vars = vars.iter().map(|v| !*v).collect_vec();
        self.or_clause(&vars);
    }
    pub fn not_clause(&mut self, var: Lit) {
        self.or_clause(&[!var])
    }
    pub fn implies_clause(&mut self, a: Lit, b: Lit) {
        self.or_clause(&[!a, b])
    }
    pub fn and_implies_clause(&mut self, a: &[Lit], b: Lit) {
        let vars = a
            .iter()
            .map(|v| !*v)
            .chain(std::iter::once(b))
            .collect_vec();
        self.or_clause(&vars)
    }
    pub fn implies_or_clause(&mut self, a: Lit, b: &[Lit]) {
        let vars = std::iter::once(a).chain(b.iter().copied()).collect_vec();
        self.or_clause(&vars)
    }
    pub fn exact_count_clause(&mut self, count: usize, vars: &[Lit]) {
        // short-circuit / optimize a few obvious edge cases
        if count == 0 {
            self.nor_clause(vars);
        } else if count == vars.len() {
            self.and_clause(vars);
        } else if count > vars.len() {
            panic!("not satisfiable")
        } else {
            let count_greater_than = self.count_up_to_vars(count + 1, vars);
            self.and_clause(&[count_greater_than[count - 1], !count_greater_than[count]]);
        }
    }

    // tseytin transform
    pub fn and_var(&mut self, vars: &[Lit]) -> Lit {
        let result = self.var();

        // result => v
        for v in vars.iter() {
            self.implies_clause(result, *v);
        }
        // and(vars) => result
        self.and_implies_clause(vars, result);

        result
    }
    pub fn or_var(&mut self, vars: &[Lit]) -> Lit {
        let result = self.var();

        // v => result
        for v in vars.iter() {
            self.implies_clause(*v, result);
        }
        // result => or(vars)
        self.implies_or_clause(result, vars);

        result
    }
    pub fn xor_var(&mut self, a: Lit, b: Lit) -> Lit {
        let result = self.var();
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
                .map(|c| self.count_var(&prior, c, *v))
                .collect_vec();
        }
        prior
    }
    fn count_var(&mut self, all_prior: &[Lit], c: usize, var: Lit) -> Lit {
        if c == 0 {
            if let Some(prior) = all_prior.first() {
                // prior | var
                return self.or_var(&[*prior, var]);
            } else {
                // no prior, so must be first iteration
                return var;
            }
        }
        if c == all_prior.len() {
            // prior_minus_one & var
            return self.and_var(&[all_prior[c - 1], var]);
        }

        let result = self.var();
        let prior_minus_one = all_prior[c - 1];
        let prior = all_prior[c];

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

#[derive(Debug)]
pub struct SatSolution {
    true_vars: HashSet<Lit>,
}
impl SatSolution {
    pub fn new(vars: Vec<Lit>) -> Self {
        let true_vars = vars.into_iter().filter(|v| v.is_positive()).collect();
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

// TODO tests! (specifically exact_count_clause)
