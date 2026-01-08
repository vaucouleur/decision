#![forbid(unsafe_code)]
//! High-level session API (scaffold).

use smt_core::{Context, SortId, TermId};
use smt_engine::engine::{SmtEngine, CheckSat};
use smt_sat::DummySat;

/// A label wrapper (kept minimal).
#[derive(Debug, Clone)]
pub struct Label(pub String);

/// Session wraps a context and an engine.
/// In a real solver, `assert` would create atoms and push them into SAT/theories.
pub struct Session {
    ctx: Context,
    eng: SmtEngine<DummySat>,
    asserted: Vec<(TermId, Option<Label>)>,
}

impl Session {
    pub fn new(theories: Vec<Box<dyn smt_engine::theory::Theory>>) -> Self {
        let ctx = Context::new();
        let eng = SmtEngine::new(ctx.clone_for_engine(), DummySat::default(), theories);
        Self { ctx, eng, asserted: Vec::new() }
    }

    /// Access the context.
    pub fn ctx(&self) -> &Context { &self.ctx }

    /// Mutable context (to declare sorts/terms).
    pub fn ctx_mut(&mut self) -> &mut Context { &mut self.ctx }

    /// Declare an uninterpreted sort.
    pub fn declare_uninterpreted_sort(&mut self, name: &str) -> SortId {
        self.ctx.declare_uninterpreted_sort(name)
    }

    /// Declare a constant.
    pub fn declare_const(&mut self, name: &str, sort: SortId) -> TermId {
        self.ctx.const_term(name, sort)
    }

    /// UF application.
    pub fn app_uf(&mut self, name: &str, args: &[TermId], out_sort: SortId) -> TermId {
        self.ctx.uf_app(name, args, out_sort)
    }

    /// Equality term.
    pub fn eq(&mut self, a: TermId, b: TermId) -> TermId {
        self.ctx.eq(a, b)
    }

    /// <= term.
    pub fn le(&mut self, a: TermId, b: TermId) -> TermId {
        self.ctx.le(a, b)
    }

    /// not term.
    pub fn not(&mut self, t: TermId) -> TermId {
        self.ctx.not(t)
    }

    /// Assert a formula (scaffold: store only).
    pub fn assert(&mut self, t: TermId, label: Option<&str>) {
        self.asserted.push((t, label.map(|s| Label(s.to_string()))));
    }

    pub fn check_sat(&mut self) -> CheckSat {
        // In a real solver, we'd translate asserted terms to atoms and feed them.
        // Here: just run the engine stub.
        self.eng.check_sat()
    }

    /// Drain eqsharing events.
    pub fn take_eqshare_events(&mut self) -> Vec<smt_engine::eqshare_trace::EqShareEvent> {
        self.eng.take_eqshare_events()
    }

    /// UNSAT core (scaffold: empty).
    pub fn get_unsat_core(&self) -> Vec<Label> { Vec::new() }
}

// --- small helper to clone context into engine ---
// For the scaffold we want both Session ctx and Engine ctx.
// In a real implementation you'd likely share an Arc<Context>.

trait CloneForEngine {
    fn clone_for_engine(&self) -> Context;
}

impl CloneForEngine for Context {
    fn clone_for_engine(&self) -> Context {
        // Minimal: create a new empty Context; production: share or clone intern tables.
        Context::new()
    }
}
