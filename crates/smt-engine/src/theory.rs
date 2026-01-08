#![forbid(unsafe_code)]
//! Theory traits + equality sharing messages.

use smt_core::TermId;

use crate::reason::ReasonId;
use crate::shared_terms::SharedTermOracle;
use crate::theory_ctx::TheoryCtx;

#[derive(Debug, Clone)]
pub struct SharedEq {
    pub a: TermId,
    pub b: TermId,
    pub explain: ReasonId,
}

pub trait EqualitySharing {
    fn export_equalities(&mut self, oracle: &SharedTermOracle, export_epoch: u64, tcx: &mut TheoryCtx) -> Vec<SharedEq>;
    fn import_equality(&mut self, eq: SharedEq, tcx: &mut TheoryCtx);
}

pub trait Theory {
    fn name(&self) -> &'static str;

    /// Return the endpoint terms used by the engine to compute shared terms.
    fn atom_endpoints(&self, atom_term: TermId) -> Vec<TermId>;

    /// Optional equality sharing hook.
    fn equality_sharing_mut(&mut self) -> Option<&mut dyn EqualitySharing> { None }
}
