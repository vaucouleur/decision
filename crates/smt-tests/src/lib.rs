#![forbid(unsafe_code)]
//! Test crate with helpers/macros and a smoke test.

pub mod eqshare_macros;
pub mod prelude;

// Templates for the richer harness described in the recap.
pub mod bundle_on_failure;
pub mod core_assert;

pub mod common;

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::common::make_session;

    #[test]
    fn smoke_builds_session_and_runs_check_sat() {
        // With no real theories, the engine returns Unknown, but we exercise the plumbing.
        let mut sess = make_session(SharingConfig { uf_to_dl: true, dl_to_uf: true });
        let _ = sess.check_sat();
        let _events = sess.take_eqshare_events();
    }
}
