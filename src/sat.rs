use varisat::cnf::CnfFormula;
use varisat::lit::LitIdx;

use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lit(LitIdx);
impl Lit {
    const SIGN_BIT: LitIdx = 1;
    const MAX_ID: usize = LitIdx::MAX as usize >> 1;
    pub const TRUE: Self = Self(0);
    pub const FALSE: Self = Self(Self::SIGN_BIT);

    pub fn new(id: usize) -> Self {
        assert!(
            id <= Self::MAX_ID && id >= 1,
            "{} must be within [1,{}]",
            id,
            Self::MAX_ID
        );
        Self((id as LitIdx) << 1)
    }

    pub fn is_const(&self) -> bool {
        (self.0 & !Self::SIGN_BIT) == 0
    }
    pub fn is_true(&self) -> bool {
        *self == Self::TRUE
    }
    pub fn is_false(&self) -> bool {
        *self == Self::FALSE
    }

    // between 1 and Self::MAX_ID
    pub fn id(&self) -> usize {
        assert!(!self.is_const());
        (self.0 >> 1) as usize
    }
    pub fn is_positive(&self) -> bool {
        self.0 & Self::SIGN_BIT == 0
    }
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }
}
impl std::ops::Not for Lit {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0 ^ Self::SIGN_BIT)
    }
}

#[derive(Debug)]
pub struct Problem {
    num_variables: usize,
    clauses: Vec<Box<[Lit]>>,
}
impl Problem {
    pub fn new() -> Self {
        Self {
            num_variables: 0,
            clauses: vec![],
        }
    }
    /// allocate a new var
    pub fn var(&mut self) -> Lit {
        assert!(self.num_variables < Lit::MAX_ID);
        self.num_variables += 1;
        Lit::new(self.num_variables)
    }
    /// add a new CNF clause
    pub fn clause(&mut self, mut vars: Vec<Lit>) {
        if !vars.contains(&Lit::TRUE) {
            vars.retain(|v| *v != Lit::FALSE);
            if vars.is_empty() {
                panic!("added rule that can never be true")
            }
            for v in vars.iter() {
                if v.id() > self.num_variables {
                    panic!("rule contains variable out of bounds")
                }
            }
            self.clauses.push(vars.into_boxed_slice())
        }
    }

    // tseytin transform
    pub fn and_var(&mut self, mut vars: Vec<Lit>) -> Lit {
        if vars.contains(&Lit::FALSE) {
            return Lit::FALSE;
        }
        vars.retain(|v| *v != Lit::TRUE);
        if vars.is_empty() {
            return Lit::TRUE;
        }
        if let [v] = *vars {
            return v;
        }

        let result = self.var();

        // result => v
        for v in vars.iter() {
            self.clause(vec![!result, *v]);
        }
        // and(vars) => result
        self.clause(
            vars.into_iter()
                .map(|v| !v)
                .chain(std::iter::once(result))
                .collect(),
        );

        result
    }
    pub fn or_var(&mut self, mut vars: Vec<Lit>) -> Lit {
        if vars.contains(&Lit::TRUE) {
            return Lit::TRUE;
        }
        vars.retain(|v| *v != Lit::FALSE);
        if vars.is_empty() {
            return Lit::FALSE;
        }
        if let [v] = *vars {
            return v;
        }

        let result = self.var();

        // v => result
        for v in vars.iter() {
            self.clause(vec![!*v, result]);
        }
        // result => or(vars)
        self.clause(std::iter::once(!result).chain(vars).collect());

        result
    }
    pub fn xor_var(&mut self, a: Lit, b: Lit) -> Lit {
        if a == Lit::FALSE {
            return b;
        }
        if a == Lit::TRUE {
            return !b;
        }
        if b == Lit::FALSE {
            return a;
        }
        if b == Lit::TRUE {
            return !a;
        }

        let result = self.var();
        self.clause(vec![!a, !b, !result]);
        self.clause(vec![a, b, !result]);
        self.clause(vec![a, !b, result]);
        self.clause(vec![!a, b, result]);
        result
    }
    pub fn eq_var(&mut self, a: Lit, b: Lit) -> Lit {
        self.xor_var(a, !b)
    }
    pub fn count_up_to(&mut self, up_to: usize, vars: &[Lit]) -> Vec<Lit> {
        let mut prior = vec![Lit::FALSE; up_to + 1];
        prior[0] = Lit::TRUE;
        for v in vars.iter().copied() {
            let mut row = Vec::with_capacity(up_to + 1);
            row.push(Lit::TRUE);
            for c in 1..=up_to {
                row.push(self.count(prior[c - 1], prior[c], v));
            }

            prior = row;
        }
        prior
    }
    fn count(&mut self, prior_minus_one: Lit, prior: Lit, var: Lit) -> Lit {
        if prior_minus_one == Lit::FALSE {
            return Lit::FALSE;
        }
        if prior == Lit::TRUE {
            return Lit::TRUE;
        }
        if var == Lit::FALSE {
            return prior;
        }
        if var == Lit::TRUE {
            return prior_minus_one;
        }

        let result = self.var();
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
        self.clause(vec![!result, prior_minus_one, prior]); // result => prior_minus_one | prior
        self.clause(vec![!result, prior, var]); // result => prior | var
        self.clause(vec![!prior_minus_one, !var, result]); // prior_minus_one & var => result
        self.clause(vec![!prior, result]); // prior => result

        result
    }
}

impl Problem {
    pub fn write_dimacs(&self, mut w: impl Write) -> std::io::Result<()> {
        writeln!(w, "p cnf {} {}", self.num_variables, self.clauses.len())?;
        for clause in self.clauses.iter() {
            for var in clause {
                if var.is_negative() {
                    write!(w, "-{} ", var.id())?;
                } else {
                    write!(w, "{} ", var.id())?;
                }
            }
            writeln!(w, "0")?;
        }
        Ok(())
    }
}
