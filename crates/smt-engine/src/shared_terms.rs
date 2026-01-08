#![forbid(unsafe_code)]
//! Engine-owned shared-term oracle for Nelson–Oppen style combination.

use hashbrown::{HashMap, HashSet};
use rustc_hash::FxHasher;
use core::hash::BuildHasherDefault;

use smt_core::TermId;

use crate::atoms::AtomTable;
use crate::theory::Theory;

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Default)]
pub struct SharedTermOracle {
    shared: HashSet<TermId, FxBuild>,
    epoch: u64,
}

impl SharedTermOracle {
    pub fn epoch(&self) -> u64 { self.epoch }

    pub fn is_shared(&self, t: TermId) -> bool {
        self.shared.contains(&t)
    }

    pub fn shared_set(&self) -> &HashSet<TermId, FxBuild> {
        &self.shared
    }

    /// Recompute shared terms based on current atoms and theory endpoint extraction.
    pub fn recompute(&mut self, atoms: &AtomTable, theories: &mut [Box<dyn Theory>]) {
        let mut owners: HashMap<TermId, u32, FxBuild> = HashMap::default();

        for atom in atoms.iter_atoms() {
            let th_idx = atom.theory.0;
            let bit = 1u32 << (th_idx.min(31) as u32);

            for t in theories[th_idx].atom_endpoints(atom.term) {
                owners.entry(t).and_modify(|m| *m |= bit).or_insert(bit);
            }
        }

        self.shared.clear();
        for (t, mask) in owners {
            if mask.count_ones() >= 2 {
                self.shared.insert(t);
            }
        }

        self.epoch = self.epoch.wrapping_add(1);
    }
}
