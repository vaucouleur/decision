#![forbid(unsafe_code)]
//! SMT engine scaffold: atoms, theories, equality sharing, trace and debug bundles.

pub mod atoms;
pub mod config;
pub mod eqshare_trace;
pub mod eqshare_dot;
pub mod reason;
pub mod reason_dot;
pub mod shared_terms;
pub mod theory;
pub mod theory_ctx;
pub mod unsat_bundle;
pub mod engine;

pub mod test_debug;
