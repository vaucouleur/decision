#![forbid(unsafe_code)]
//! Graphviz exporter for equality sharing events.

use std::fmt::Write;

use hashbrown::{HashMap, HashSet};
use rustc_hash::FxHasher;
use core::hash::BuildHasherDefault;

use smt_core::{Context, TermId};
use crate::atoms::TheoryId;
use crate::reason::ReasonArena;
use crate::eqshare_trace::EqShareEvent;

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Debug, Clone, Copy)]
pub struct EqDotLimits {
    pub max_events: usize,
    pub max_reason_lits: usize,
    pub include_reason_nodes: bool,
}

impl Default for EqDotLimits {
    fn default() -> Self {
        Self { max_events: 300, max_reason_lits: 6, include_reason_nodes: false }
    }
}

fn fmt_term(_ctx: &Context, t: TermId) -> String {
    format!("{t:?}")
}

fn fmt_lit_short(l: smt_sat::Lit) -> String {
    if l.is_pos() { format!("v{}", l.var().0) } else { format!("¬v{}", l.var().0) }
}

pub fn eqshare_to_dot(
    ctx: &Context,
    reasons: &ReasonArena,
    events: &[EqShareEvent],
    theory_names: &dyn Fn(TheoryId) -> &'static str,
    limits: EqDotLimits,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "digraph EqShare {{").ok();
    writeln!(&mut out, "  rankdir=LR;").ok();
    writeln!(&mut out, "  node [fontname=\"Helvetica\"];").ok();

    let mut term_nodes: HashMap<TermId, String, FxBuild> = HashMap::default();
    let mut next_term = 0usize;

    let mut edge_seen: HashSet<(TheoryId, TheoryId, TermId, TermId, u64), FxBuild> = HashSet::default();

    for ev in events.iter().take(limits.max_events) {
        let (p, q) = if ev.a < ev.b { (ev.a, ev.b) } else { (ev.b, ev.a) };
        if !edge_seen.insert((ev.src, ev.dst, p, q, ev.epoch)) {
            continue;
        }

        let na = term_nodes.entry(ev.a).or_insert_with(|| {
            let id = format!("t{next_term}");
            next_term += 1;
            id
        }).clone();

        let nb = term_nodes.entry(ev.b).or_insert_with(|| {
            let id = format!("t{next_term}");
            next_term += 1;
            id
        }).clone();

        let src_name = theory_names(ev.src);
        let dst_name = theory_names(ev.dst);

        let lits = reasons.expand_lits(ev.explain);
        let mut rs = String::new();
        for (i, l) in lits.iter().take(limits.max_reason_lits).enumerate() {
            if i > 0 { rs.push_str(","); }
            rs.push_str(&fmt_lit_short(*l));
        }
        if lits.len() > limits.max_reason_lits { rs.push_str(",..."); }

        let label = if rs.is_empty() {
            format!("{src_name}→{dst_name} @{}", ev.epoch)
        } else {
            format!("{src_name}→{dst_name} @{}\\n{rs}", ev.epoch)
        };

        writeln!(&mut out, "  {na} -> {nb} [label=\"{label}\"];").ok();

        if limits.include_reason_nodes {
            let rn = format!("r{}", ev.explain.0);
            writeln!(&mut out, "  {rn} [shape=box,style=dashed,label=\"ReasonId({})\"];", ev.explain.0).ok();
            writeln!(&mut out, "  {na} -> {rn} [style=dotted,arrowhead=none];").ok();
            writeln!(&mut out, "  {rn} -> {nb} [style=dotted,arrowhead=none];").ok();
        }
    }

    for (t, nid) in &term_nodes {
        let lab = fmt_term(ctx, *t).replace('"', "\\"");
        writeln!(&mut out, "  {nid} [shape=ellipse,label=\"{lab}\"];").ok();
    }

    if events.len() > limits.max_events {
        writeln!(&mut out, "  truncated [shape=note,label=\"TRUNCATED: events={} max_events={}\"];", events.len(), limits.max_events).ok();
    }

    writeln!(&mut out, "}}").ok();
    out
}
