#![forbid(unsafe_code)]

pub use smt_engine::config::SharingConfig;
pub use smt_engine::engine::CheckSat;

// helpers
pub use crate::bundle_on_failure::{assert_sat_with_bundle_on_mismatch, Expect};
pub use crate::core_assert::{assert_unsat_core_contains_any, CoreReq};

// macros
pub use crate::{
    assert_eqshare_dir,
    assert_eqshare_dir_none,
    assert_eqshare_events_empty,
    assert_eqshare_hop,
    assert_eqshare_hop_any,
    assert_eqshare_hop_none,
};
