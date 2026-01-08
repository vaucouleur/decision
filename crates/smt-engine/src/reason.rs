#![forbid(unsafe_code)]
//! Minimal reason arena with AND-composition and SAT literal leaves.

use smt_sat::Lit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReasonId(pub u32);

#[derive(Debug, Clone)]
pub enum Reason {
    Atom(Lit),
    And(Vec<ReasonId>),
}

#[derive(Debug, Default)]
pub struct ReasonArena {
    reasons: Vec<Reason>,
}

impl ReasonArena {
    pub fn push(&mut self, r: Reason) -> ReasonId {
        let id = ReasonId(self.reasons.len() as u32);
        self.reasons.push(r);
        id
    }

    pub fn get(&self, rid: ReasonId) -> &Reason {
        &self.reasons[rid.0 as usize]
    }

    /// Expand a reason into a flat list of literals (bounded by traversal).
    pub fn expand_lits(&self, root: ReasonId) -> Vec<Lit> {
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(r) = stack.pop() {
            match self.get(r) {
                Reason::Atom(l) => out.push(*l),
                Reason::And(kids) => stack.extend(kids.iter().copied()),
            }
        }
        out
    }
}
