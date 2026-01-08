#![forbid(unsafe_code)]
//! Bundle-on-failure rerun helpers (template).
//!
//! In a full solver, this will rerun the instance with eq-sharing debug enabled,
//! forcing the engine to write DOT bundles on UNSAT or on detected mismatch.

use smt_api::Session;
use smt_engine::engine::CheckSat;
use smt_engine::test_debug;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    Sat,
    Unsat,
}

pub fn assert_sat_with_bundle_on_mismatch<FMake, FSetup>(
    mut make_sess: FMake,
    setup: FSetup,
    expect: Expect,
) -> smt_core::Result<()>
where
    FMake: FnMut() -> Session,
    FSetup: Fn(&mut Session) -> smt_core::Result<()>,
{
    test_debug::set_eqshare_debug_enabled(false);

    let mut sess = make_sess();
    setup(&mut sess)?;
    let got = sess.check_sat();

    let ok = match (expect, got) {
        (Expect::Sat, CheckSat::Sat) => true,
        (Expect::Unsat, CheckSat::Unsat) => true,
        _ => false,
    };

    if ok {
        return Ok(());
    }

    test_debug::set_eqshare_debug_enabled(true);

    let mut sess2 = make_sess();
    setup(&mut sess2)?;
    let got2 = sess2.check_sat();

    test_debug::set_eqshare_debug_enabled(false);

    panic!(
        "check_sat mismatch: expected {:?}, got {:?}; rerun-with-debug got {:?}.          See target/smt-debug-<pid>-<epoch>/ for eqshare.dot + conflict.dot.",
        expect, got, got2
    );
}
