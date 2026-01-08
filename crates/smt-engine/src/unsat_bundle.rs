#![forbid(unsafe_code)]
//! Writes an UNSAT debug bundle (eqshare.dot + conflict.dot + README.txt).

use std::io::Write;
use std::path::Path;

use crate::reason::ReasonId;
use crate::engine::SmtEngine;

pub fn write_unsat_debug_bundle<K: smt_sat::SatKernel>(
    engine: &SmtEngine<K>,
    conflict_reason: ReasonId,
) {
    let pid = std::process::id();
    let dir = Path::new("target").join(format!("smt-debug-{}-{}", pid, engine.export_epoch));

    if std::fs::create_dir_all(&dir).is_err() { return; }

    let eqshare_dot = engine.dump_eqshare_dot();
    let _ = std::fs::write(dir.join("eqshare.dot"), eqshare_dot.as_bytes());

    let conflict_dot = engine.dump_conflict_reason_dot(conflict_reason);
    let _ = std::fs::write(dir.join("conflict.dot"), conflict_dot.as_bytes());

    let mut f = match std::fs::File::create(dir.join("README.txt")) {
        Ok(f) => f,
        Err(_) => return,
    };

    let _ = writeln!(f, "SMT UNSAT DEBUG BUNDLE");
    let _ = writeln!(f, "");
    let _ = writeln!(f, "Files:");
    let _ = writeln!(f, "  - eqshare.dot   : equality-sharing exchanges (terms + edges with direction/epoch)");
    let _ = writeln!(f, "  - conflict.dot  : reason DAG for the reported conflict reason");
    let _ = writeln!(f, "");
    let _ = writeln!(f, "Render to SVG:");
    let _ = writeln!(f, "  dot -Tsvg eqshare.dot  > eqshare.svg");
    let _ = writeln!(f, "  dot -Tsvg conflict.dot > conflict.svg");
    let _ = writeln!(f, "");
    let _ = writeln!(f, "Conflict root ReasonId: {}", conflict_reason.0);
}
