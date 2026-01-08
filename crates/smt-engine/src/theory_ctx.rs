#![forbid(unsafe_code)]
//! TheoryCtx: convenience builder for reason composition.

use crate::reason::{Reason, ReasonArena, ReasonId};

pub struct TheoryCtx<'a> {
    arena: &'a mut ReasonArena,
}

impl<'a> TheoryCtx<'a> {
    pub fn new(arena: &'a mut ReasonArena) -> Self { Self { arena } }

    /// Create an AND reason from component reasons.
    pub fn r_and(&mut self, kids: Vec<ReasonId>) -> ReasonId {
        if kids.len() == 1 { return kids[0]; }
        self.arena.push(Reason::And(kids))
    }
}
