#![forbid(unsafe_code)]

use smt_api::Session;
use smt_engine::config::SharingConfig;

/// Build a session with two placeholder theories named UF and DL.
/// Replace these with real implementations.
pub fn make_session(_sharing: SharingConfig) -> Session {
    let theories: Vec<Box<dyn smt_engine::theory::Theory>> = vec![
        Box::new(PlaceholderTheory { nm: "UF" }),
        Box::new(PlaceholderTheory { nm: "DL" }),
    ];
    Session::new(theories)
}

struct PlaceholderTheory {
    nm: &'static str,
}

impl smt_engine::theory::Theory for PlaceholderTheory {
    fn name(&self) -> &'static str { self.nm }

    fn atom_endpoints(&self, _atom_term: smt_core::TermId) -> Vec<smt_core::TermId> {
        // No endpoints in scaffold.
        Vec::new()
    }
}
