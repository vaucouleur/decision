#![forbid(unsafe_code)]

use smt_core::TermId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TheoryId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct Atom {
    pub term: TermId,
    pub theory: TheoryId,
}

#[derive(Debug, Default)]
pub struct AtomTable {
    atoms: Vec<Atom>,
}

impl AtomTable {
    pub fn len(&self) -> usize { self.atoms.len() }

    pub fn push(&mut self, atom: Atom) {
        self.atoms.push(atom);
    }

    pub fn iter_atoms(&self) -> impl Iterator<Item = Atom> + '_ {
        self.atoms.iter().copied()
    }
}
