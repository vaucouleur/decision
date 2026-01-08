#![forbid(unsafe_code)]
//! UNSAT core assertion helpers (template).
//!
//! This scaffold does not compute real cores yet, but keeps the API shape.

use smt_api::Session;
use smt_engine::test_debug;

#[derive(Debug, Clone)]
pub struct CoreReq {
    pub any_of: &'static [&'static str],
    pub name: &'static str,
}

pub fn assert_unsat_core_contains_any<FMake, FSetup>(
    mut make_sess: FMake,
    setup: FSetup,
    _reqs: &[CoreReq],
) -> smt_core::Result<()>
where
    FMake: FnMut() -> Session,
    FSetup: Fn(&mut Session) -> smt_core::Result<()>,
{
    // Placeholder: rerun with debug on mismatch would go here once the core exists.
    test_debug::set_eqshare_debug_enabled(false);
    let mut sess = make_sess();
    setup(&mut sess)?;
    let _ = sess.check_sat();
    let _core = sess.get_unsat_core();
    Ok(())
}
