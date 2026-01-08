#![forbid(unsafe_code)]
//! Core term model: `Context`, `TermId`, `SortId`, and basic term construction.
//!
//! This is deliberately small but keeps the shape you want for a modular SMT engine.

use rustc_hash::FxHashMap;

/// Error type for fallible APIs in this crate family.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Result alias for fallible APIs in this crate family.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SortId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SortKind {
    Int,
    Uninterpreted(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OpKind {
    /// Uninterpreted function symbol.
    Uf(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Op {
    pub kind: OpKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermKind {
    /// An n-ary application (including UF).
    App { op: Op, args: Vec<TermId> },
    /// Integer constant.
    IntConst(i64),
    /// A named constant/variable.
    Const(String),
    /// Equality (as a term).
    Eq(TermId, TermId),
    /// Integer <= (as a term).
    Le(TermId, TermId),
    /// Negation (as a term).
    Not(TermId),
}

#[derive(Debug, Clone)]
pub struct TermNode {
    pub kind: TermKind,
    pub sort: SortId,
}

#[derive(Debug, Default)]
pub struct Context {
    sorts: Vec<SortKind>,
    terms: Vec<TermNode>,
    sort_cache: FxHashMap<SortKind, SortId>,
}

impl Context {
    /// Create an empty context.
    pub fn new() -> Self {
        let mut ctx = Self::default();
        // Intern Int sort at SortId(0) for convenience.
        ctx.sorts.push(SortKind::Int);
        ctx.sort_cache.insert(SortKind::Int, SortId(0));
        ctx
    }

    /// Built-in Int sort.
    pub fn int_sort(&self) -> SortId {
        SortId(0)
    }

    /// Declare an uninterpreted sort.
    pub fn declare_uninterpreted_sort(&mut self, name: impl Into<String>) -> SortId {
        let k = SortKind::Uninterpreted(name.into());
        if let Some(&sid) = self.sort_cache.get(&k) {
            return sid;
        }
        let sid = SortId(self.sorts.len() as u32);
        self.sorts.push(k.clone());
        self.sort_cache.insert(k, sid);
        sid
    }

    /// Add a term and return its id.
    pub fn intern(&mut self, kind: TermKind, sort: SortId) -> TermId {
        let id = TermId(self.terms.len() as u32);
        self.terms.push(TermNode { kind, sort });
        id
    }

    /// Read a term node.
    pub fn term_node(&self, t: TermId) -> (&TermKind, SortId) {
        let n = &self.terms[t.0 as usize];
        (&n.kind, n.sort)
    }

    /// Construct an int constant term.
    pub fn int_const(&mut self, v: i64) -> TermId {
        self.intern(TermKind::IntConst(v), self.int_sort())
    }

    /// Construct a named constant/variable term.
    pub fn const_term(&mut self, name: impl Into<String>, sort: SortId) -> TermId {
        self.intern(TermKind::Const(name.into()), sort)
    }

    /// Construct a UF application term.
    pub fn uf_app(&mut self, name: impl Into<String>, args: &[TermId], out_sort: SortId) -> TermId {
        self.intern(
            TermKind::App {
                op: Op { kind: OpKind::Uf(name.into()) },
                args: args.to_vec(),
            },
            out_sort,
        )
    }

    /// Construct equality term.
    pub fn eq(&mut self, a: TermId, b: TermId) -> TermId {
        // Sort is Int-like boolean; we don't model Bool sort in this scaffold.
        self.intern(TermKind::Eq(a, b), self.int_sort())
    }

    /// Construct <= term.
    pub fn le(&mut self, a: TermId, b: TermId) -> TermId {
        self.intern(TermKind::Le(a, b), self.int_sort())
    }

    /// Construct not term.
    pub fn not(&mut self, t: TermId) -> TermId {
        self.intern(TermKind::Not(t), self.int_sort())
    }
}
