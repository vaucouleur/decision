#![forbid(unsafe_code)]
//! Equality-sharing trace (test/debug aid).

use smt_core::TermId;
use crate::atoms::TheoryId;
use crate::reason::ReasonId;

#[derive(Debug, Clone)]
pub struct EqShareEvent {
    pub epoch: u64,
    pub src: TheoryId,
    pub dst: TheoryId,
    pub a: TermId,
    pub b: TermId,
    pub explain: ReasonId,
}

#[derive(Default)]
pub struct EqShareTrace {
    events: Vec<EqShareEvent>,
}

impl EqShareTrace {
    pub fn push(&mut self, ev: EqShareEvent) {
        self.events.push(ev);
    }

    pub fn clear(&mut self) { self.events.clear(); }

    pub fn events(&self) -> &[EqShareEvent] { &self.events }

    /// Drain all events (useful in tests after a single `check_sat()`).
    pub fn take(&mut self) -> Vec<EqShareEvent> {
        core::mem::take(&mut self.events)
    }
}
