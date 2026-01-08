#![forbid(unsafe_code)]
//! Core engine loop (scaffold).
//!
//! This file contains the **equality sharing round** and the debug dump helpers.
//! A real solver would also implement: SAT trail syncing, theory propagation, conflicts, etc.

use hashbrown::HashSet;
use rustc_hash::FxHasher;
use core::hash::BuildHasherDefault;

use smt_core::Context;

use crate::atoms::{AtomTable, TheoryId};
use crate::config::EngineConfig;
use crate::eqshare_dot::{eqshare_to_dot, EqDotLimits};
use crate::eqshare_trace::{EqShareEvent, EqShareTrace};
use crate::reason::{ReasonArena, ReasonId};
use crate::reason_dot::{reason_to_dot, DotLimits};
use crate::shared_terms::SharedTermOracle;
use crate::theory::{EqualitySharing, SharedEq, Theory};
use crate::theory_ctx::TheoryCtx;

type FxBuild = BuildHasherDefault<FxHasher>;

pub enum CheckSat {
    Sat,
    Unsat,
    Unknown,
}

pub struct SmtEngine<K: smt_sat::SatKernel> {
    pub ctx: Context,
    pub sat: K,

    pub theories: Vec<Box<dyn Theory>>,
    pub uf_id: TheoryId,
    pub dl_id: TheoryId,

    pub atoms: AtomTable,

    pub reasons: ReasonArena,

    pub shared_terms: SharedTermOracle,
    pub export_epoch: u64,

    pub config: EngineConfig,

    #[cfg(feature = "test-debug")]
    pub eqshare_trace: EqShareTrace,

    eq_log_seen: HashSet<(TheoryId, TheoryId, smt_core::TermId, smt_core::TermId, u64), FxBuild>,
    last_atom_count: usize,
}

impl<K: smt_sat::SatKernel> SmtEngine<K> {
    pub fn new(ctx: Context, sat: K, theories: Vec<Box<dyn Theory>>) -> Self {
        let mut eng = Self {
            ctx,
            sat,
            uf_id: TheoryId(0),
            dl_id: TheoryId(1),
            theories,
            atoms: AtomTable::default(),
            reasons: ReasonArena::default(),
            shared_terms: SharedTermOracle::default(),
            export_epoch: 0,
            config: EngineConfig::default(),
            #[cfg(feature = "test-debug")]
            eqshare_trace: EqShareTrace::default(),
            eq_log_seen: HashSet::default(),
            last_atom_count: 0,
        };

        // Cross-crate test toggle.
        if crate::test_debug::eqshare_debug_enabled() {
            eng.config.debug_eq.enabled = true;
            eng.config.debug_eq.max_reason_lits = 16;
        }

        eng
    }

    /// (Scaffold) recompute shared terms when atom table changed.
    pub fn maybe_recompute_shared_terms(&mut self) {
        let n = self.atoms.len();
        if n != self.last_atom_count {
            self.shared_terms.recompute(&self.atoms, &mut self.theories);
            self.last_atom_count = n;
        }
    }

    /// Perform one equality-sharing round: export from each theory, import into others.
    pub fn equality_sharing_round(&mut self) {
        self.maybe_recompute_shared_terms();

        let dbg = self.config.debug_eq;
        let enabled = dbg.enabled;

        let mut tcx = TheoryCtx::new(&mut self.reasons);

        if enabled && dbg.log_shared_stats {
            eprintln!(
                "[eqshare][epoch={}]: shared_terms={}",
                self.export_epoch,
                self.shared_terms.shared_set().len()
            );
        }

        let mut exported: Vec<(TheoryId, SharedEq)> = Vec::new();
        for (i, th) in self.theories.iter_mut().enumerate() {
            let src = TheoryId(i);
            if let Some(sh) = th.equality_sharing_mut() {
                for eq in sh.export_equalities(&self.shared_terms, self.export_epoch, &mut tcx) {
                    exported.push((src, eq));
                }
            }
        }

        for (src, eq) in exported {
            for (j, th) in self.theories.iter_mut().enumerate() {
                let dst = TheoryId(j);
                if dst == src { continue; }

                let allow = match (src, dst) {
                    (s, d) if s == self.uf_id && d == self.dl_id => self.config.sharing.uf_to_dl,
                    (s, d) if s == self.dl_id && d == self.uf_id => self.config.sharing.dl_to_uf,
                    _ => true,
                };
                if !allow { continue; }

                // record trace (test/debug only)
                #[cfg(feature = "test-debug")]
                {
                    self.eqshare_trace.push(EqShareEvent {
                        epoch: self.export_epoch,
                        src,
                        dst,
                        a: eq.a,
                        b: eq.b,
                        explain: eq.explain,
                    });
                }

                // (Optional) log first time seen
                if enabled && (dbg.log_exports || dbg.log_imports) {
                    let (p, q) = if eq.a < eq.b { (eq.a, eq.b) } else { (eq.b, eq.a) };
                    if self.eq_log_seen.insert((src, dst, p, q, self.export_epoch)) {
                        let a_str = format!("{:?}", eq.a);
                        let b_str = format!("{:?}", eq.b);
                        let lits = self.reasons.expand_lits(eq.explain);
                        let mut rs = String::new();
                        for (idx, l) in lits.iter().take(dbg.max_reason_lits).enumerate() {
                            if idx > 0 { rs.push_str(", "); }
                            rs.push_str(if l.is_pos() { &format!("v{}", l.var().0) } else { &format!("¬v{}", l.var().0) });
                        }
                        if lits.len() > dbg.max_reason_lits { rs.push_str(", ..."); }

                        eprintln!(
                            "[eqshare][epoch={}]: {} -> {} {} = {}{}",
                            self.export_epoch,
                            self.theories[src.0].name(),
                            self.theories[dst.0].name(),
                            a_str,
                            b_str,
                            if rs.is_empty() { "".to_string() } else { format!("  because {rs}") },
                        );
                    }
                }

                if let Some(sh) = th.equality_sharing_mut() {
                    sh.import_equality(eq.clone(), &mut tcx);
                }
            }
        }

        self.export_epoch = self.export_epoch.wrapping_add(1);
    }

    /// Dump equality sharing trace to DOT (only meaningful if trace is enabled).
    pub fn dump_eqshare_dot(&self) -> String {
        #[cfg(feature = "test-debug")]
        {
            eqshare_to_dot(
                &self.ctx,
                &self.reasons,
                self.eqshare_trace.events(),
                &|tid| self.theories[tid.0].name(),
                EqDotLimits::default(),
            )
        }
        #[cfg(not(feature = "test-debug"))]
        {
            "digraph EqShare { /* test-debug feature disabled */ }".to_string()
        }
    }

    /// Dump conflict reason DAG to DOT.
    pub fn dump_conflict_reason_dot(&self, root: ReasonId) -> String {
        reason_to_dot(&self.reasons, root, DotLimits::default())
    }

    /// (Scaffold) Solve.
    pub fn check_sat(&mut self) -> CheckSat {
        // In a real solver, this would run CDCL(T) + theory propagation + conflicts.
        // Here we just run one sharing round so that trace/dot infrastructure can be exercised.
        self.equality_sharing_round();
        CheckSat::Unknown
    }

    /// (Test/debug) Drain trace events.
    #[cfg(feature = "test-debug")]
    pub fn take_eqshare_events(&mut self) -> Vec<EqShareEvent> {
        self.eqshare_trace.take()
    }

    #[cfg(not(feature = "test-debug"))]
    pub fn take_eqshare_events(&mut self) -> Vec<EqShareEvent> {
        Vec::new()
    }
}
