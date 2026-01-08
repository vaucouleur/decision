#![forbid(unsafe_code)]
//! SAT kernel interface + basic literal type.
//!
//! This is a stub to keep the workspace compiling. Replace with a real CDCL
//! kernel (or wrap an existing solver).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lit {
    var: VarId,
    sign: bool, // true = positive
}

impl Lit {
    pub fn pos(v: VarId) -> Self { Self { var: v, sign: true } }
    pub fn neg(v: VarId) -> Self { Self { var: v, sign: false } }

    pub fn var(self) -> VarId { self.var }
    pub fn is_pos(self) -> bool { self.sign }
}

/// SAT kernel interface expected by the SMT engine.
///
/// A production solver would expose:
/// - enqueue / propagate
/// - conflict analysis
/// - decision heuristic
pub trait SatKernel {
    fn propagate(&mut self) -> Result<(), ()>;
}

/// Trivial kernel: never propagates, never conflicts.
#[derive(Default)]
pub struct DummySat;

impl SatKernel for DummySat {
    fn propagate(&mut self) -> Result<(), ()> { Ok(()) }
}
