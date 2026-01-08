#![forbid(unsafe_code)]
//! Graphviz exporter for a bounded reason DAG.

use std::fmt::Write;

use hashbrown::{HashMap, HashSet};
use rustc_hash::FxHasher;
use core::hash::BuildHasherDefault;

use crate::reason::{Reason, ReasonArena, ReasonId};

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Debug, Clone, Copy)]
pub struct DotLimits {
    pub max_reason_nodes: usize,
    pub max_lit_label_len: usize,
}

impl Default for DotLimits {
    fn default() -> Self {
        Self { max_reason_nodes: 300, max_lit_label_len: 64 }
    }
}

fn fmt_lit(l: smt_sat::Lit) -> String {
    if l.is_pos() { format!("v{}", l.var().0) } else { format!("¬v{}", l.var().0) }
}

pub fn reason_to_dot(arena: &ReasonArena, root: ReasonId, limits: DotLimits) -> String {
    let mut queue = std::collections::VecDeque::new();
    let mut seen: HashSet<ReasonId, FxBuild> = HashSet::default();

    queue.push_back(root);
    seen.insert(root);

    let mut order = Vec::<ReasonId>::new();

    while let Some(r) = queue.pop_front() {
        order.push(r);
        if order.len() >= limits.max_reason_nodes { break; }
        match arena.get(r) {
            Reason::Atom(_) => {}
            Reason::And(kids) => {
                for &k in kids {
                    if seen.insert(k) { queue.push_back(k); }
                }
            }
        }
    }

    let mut name: HashMap<ReasonId, String, FxBuild> = HashMap::default();
    for (i, rid) in order.iter().copied().enumerate() {
        name.insert(rid, format!("r{i}"));
    }

    let mut out = String::new();
    writeln!(&mut out, "digraph Reason {{").ok();
    writeln!(&mut out, "  rankdir=LR;").ok();
    writeln!(&mut out, "  node [fontname=\"Helvetica\"];").ok();

    for rid in &order {
        let nid = &name[rid];
        match arena.get(*rid) {
            Reason::Atom(l) => {
                let mut lab = fmt_lit(*l);
                if lab.len() > limits.max_lit_label_len { lab.truncate(limits.max_lit_label_len); }
                writeln!(&mut out, "  {nid} [shape=ellipse,label=\"{lab}\"];").ok();
            }
            Reason::And(kids) => {
                writeln!(&mut out, "  {nid} [shape=box,label=\"AND ({})\"];", kids.len()).ok();
            }
        }
    }

    for rid in &order {
        let src = &name[rid];
        if let Reason::And(kids) = arena.get(*rid) {
            for &k in kids {
                if let Some(dst) = name.get(&k) {
                    writeln!(&mut out, "  {src} -> {dst};").ok();
                }
            }
        }
    }

    writeln!(&mut out, "}}").ok();
    out
}
