# SMT Solver (UF + Difference Logic) — Equality Sharing Architecture & Regression Suite (Recap)

This document recaps the end-to-end design we built so far: a modular Rust SMT architecture with UF + Difference Logic (DL), a Nelson–Oppen-style equality sharing mechanism, plus a “golden” regression suite and debug tooling (logs + Graphviz bundles) that make combination bugs easy to diagnose.

---

## 1. Goal and scope

We implemented a **two-theory SMT core**:

- **UF (Uninterpreted Functions)**: congruence closure with reasons.
- **DL (Difference Logic)** over integers: constraints of the form `x - y ≤ c` (including equality when both directions hold).

We then added **theory combination** via **equality sharing**, including:
- directional control (UF→DL, DL→UF),
- shared-term detection,
- stable fixpoint execution,
- throttling/deduplication to avoid repeated exports,
- deterministic regression tests that fail fast when sharing breaks.

The final deliverable includes:
- **3 regressions**: UF→DL only, DL→UF only, and a **ping-pong** that requires **both directions** in the same run.
- A **SAT/UNSAT matrix** over the sharing config.
- **Trace-based assertions** for exact “hop” events.
- Automatic **UNSAT debug bundles** (DOT files) on failure.

---

## 2. High-level architecture

### 2.1 Workspace layout (conceptual)

- `smt-core`
  - Terms, sorts, operations, context, term constructors
  - Canonical IDs: `TermId`, `SortId`, `OpId`
- `smt-sat`
  - CDCL SAT kernel trait + implementation (toy or real)
  - Literals, variables, propagation interface
- `smt-engine`
  - Glue layer: atoms, theory registry, fixpoint loop
  - Equality sharing infrastructure (oracle + rounds)
  - Debug + trace + DOT exporters
- `smt-api`
  - Friendly `Session`: declare symbols, assert formulas, check_sat, get_unsat_core
- `smt-tests`
  - Golden regression suite
  - Helpers/macros for core and trace assertions

---

## 3. Engine execution model

### 3.1 Atoms and theory ownership

The engine maintains an `AtomTable` of propositional atoms (e.g. `t1 = t2`, `t1 ≤ t2`, …).

A **classifier** assigns each atom to exactly one theory:
- Any atom containing UF applications goes to **UF**.
- Difference logic constraints over integers go to **DL**.

Each theory receives:
- the atom being asserted / propagated,
- a `ReasonId` proving why it must hold (for explanation and UNSAT core).

### 3.2 Fixpoint loop

At a high level:

1. SAT propagates literals.
2. Engine notifies theories of newly true atoms.
3. Theories propagate derived atoms (to SAT) or report conflict.
4. Engine runs an **equality-sharing round**:
   - theories export implied equalities among shared terms
   - engine imports them into other theories
5. Engine runs a second theory fixpoint (imports may unlock new propagations).
6. If no progress, SAT decides / terminates.

This is the standard “CDCL(T)” style, extended with explicit equality exchange.

---

## 4. Equality sharing (Nelson–Oppen style)

### 4.1 Directional sharing configuration

We introduced a runtime config:

```rust
pub struct SharingConfig {
    pub uf_to_dl: bool,
    pub dl_to_uf: bool,
}
````

The engine uses it to allow/disallow sharing for specific directed edges:

* UF → DL
* DL → UF

This enables precise regression matrices and helps debug half-broken sharing.

### 4.2 Shared term detection: SharedTermOracle

We centralized shared-term computation in an engine-owned oracle:

* Each theory provides `atom_endpoints(atom_term) -> Vec<TermId>`.
* Engine maps each endpoint term to an “owner bitmask”.
* A term is **shared** if it appears in endpoints for ≥ 2 theories.

The oracle is recomputed when new atoms appear (or conservatively each outer loop).

**Why engine-owned?**

* A single source of truth.
* No theory needs to guess sharedness.
* Deterministic and stable across fixes.

### 4.3 Critical UF endpoint fix: include UF app arguments

To make DL→UF and ping-pong work reliably, UF endpoint extraction must include not only the atom endpoints (lhs/rhs) but also the **arguments of UF applications** inside them.

Example: in `g(x) ≠ g(y)` the shared integer terms are `x` and `y`, even though they appear only as UF arguments.

We added:

* `collect_uf_app_args(term, out)` which finds UF `App` nodes and pushes their args.
* `atom_endpoints` returns `{lhs, rhs} + uf_app_args(lhs) + uf_app_args(rhs)`.

This is required so DL can export `x=y` and UF can receive it.

### 4.4 Exporting equalities from UF: bucket shared UF-app terms by representative

UF must export not just asserted equalities, but **implied congruence equalities** among shared UF-app terms. Key example:

* asserted: `a = b`
* implied by UF congruence: `f(a) = f(b)`

This implication is not necessarily present as an explicit atom.

We implemented UF export as:

* Consider all shared terms from oracle.
* Filter to UF app terms (e.g., `f(a)`, `f(b)`).
* Compute UF representative for each term.
* Bucket by representative.
* Export a **spanning tree** of equalities within each bucket (head = others) to avoid quadratic output.

This produces minimal, deterministic equality exports.

### 4.5 Exporting equalities from DL

DL exports equalities for shared integer terms when it can prove:

* `a ≤ b` and `b ≤ a` (distance both ways ≤ 0)

Implementation:

* compute all-pairs shortest paths (or incremental) with predecessor tracking,
* if both directions distance ≤ 0, produce `a=b` with a combined reason (two path reasons AND-ed).

### 4.6 Importing equalities

* UF imports `a=b` by union in congruence closure with reason.
* DL imports `a=b` by adding edges `a→b (0)` and `b→a (0)` with reason.

---

## 5. Throttling & deduplication (production-grade behavior)

Equality sharing can easily spam the same equalities each round. We added two layers:

### 5.1 Epoch throttling

Engine maintains `export_epoch: u64` and increments it on significant state changes (at least per outer loop; optionally on imports).

Each theory has `last_export_epoch`:

* If `last_export_epoch == export_epoch`, export nothing.

This prevents repeated export calls within a stable fixpoint state.

### 5.2 Pair dedup with rollback support

Each theory maintains:

* `exported_pairs: HashSet<(TermId, TermId)>`
* `exported_pairs_trail: Vec<(TermId, TermId)>`
* `exported_pairs_cp: Vec<usize>` checkpoints

On export, before pushing an equality, normalize pair `(min,max)` and insert:

* if it was already exported in this branch/level, skip.
* if new, push the key to the trail.

On checkpoint:

* push current trail length.

On rollback:

* pop checkpoint length, pop trail down to target and remove those keys from set.

This ensures:

* no duplicates within a search branch,
* correctness under SAT backtracking and theory push/pop.

---

## 6. Three deterministic regressions

We built three minimal examples to validate the whole pipeline.

### 6.1 UF → DL regression (requires UF→DL)

Symbols:

* sort `U`
* `a,b : U`
* `f : U → Int`

Constraints:

* `a = b` (UF)
* `f(a) ≤ 0` (DL)
* `1 ≤ f(b)` (DL)

Reasoning:

* UF congruence implies `f(a)=f(b)`
* DL then conflicts with `f(a) ≤ 0` and `f(b) ≥ 1`

Matrix:

* no sharing: SAT
* UF→DL only: UNSAT
* DL→UF only: SAT
* both: UNSAT (still)

Trace requirement:

* Must see UF→DL event exporting/importing `f(a)=f(b)`.

### 6.2 DL → UF regression (requires DL→UF)

Symbols:

* `x,y : Int`
* sort `U`
* `g : Int → U`

Constraints:

* `x ≤ y` (DL)
* `y ≤ x` (DL) ⇒ DL proves `x=y`
* `g(x) ≠ g(y)` (UF)

Reasoning:

* DL exports `x=y`
* UF imports `x=y` ⇒ congruence gives `g(x)=g(y)` contradicting diseq.

Matrix:

* no sharing: SAT
* DL→UF only: UNSAT
* UF→DL only: SAT
* both: UNSAT

Trace requirement:

* Must see DL→UF event exporting/importing `x=y`.

### 6.3 Ping-pong regression (requires both directions)

Symbols:

* sort `U`
* `a,b : U`
* `x,y : Int`
* `f : U → Int`
* `g : Int → U`

Constraints:

* UF: `a = b`
* DL: `x = f(a)` (encoded as `x ≤ f(a)` and `f(a) ≤ x`)
* DL: `y = f(b)` (encoded similarly)
* UF: `g(x) ≠ g(y)`

Reasoning:

1. UF ⇒ `a=b` implies `f(a)=f(b)` (**UF→DL**)
2. DL with `x=f(a)` and `y=f(b)` and `f(a)=f(b)` ⇒ `x=y` (**DL proves equality**)
3. DL exports `x=y` back (**DL→UF**)
4. UF imports `x=y` ⇒ `g(x)=g(y)` contradicting `g(x) ≠ g(y)`.

Matrix:

* none: SAT
* UF→DL only: SAT
* DL→UF only: SAT
* both: **UNSAT**

Trace requirement:

* Must see both directions:

  * UF→DL includes `f(a)=f(b)`
  * DL→UF includes `x=y`

---

## 7. Regression suite design

### 7.1 Matrix tests

For each regression, we run a small matrix over `(uf_to_dl, dl_to_uf)` and assert:

* SAT/UNSAT outcomes,
* optional UNSAT core shape,
* trace hop presence/absence.

### 7.2 Core assertions that aren’t brittle

UNSAT cores vary. We used “requirements groups”:

* Each group is `any_of: &[label]`, and the core must contain at least one from each group.
* This avoids overfitting to a specific proof path.

### 7.3 Equality-sharing trace assertions

We record `EqShareEvent` on actual imports:

```rust
pub struct EqShareEvent {
    pub epoch: u64,
    pub src: TheoryId,
    pub dst: TheoryId,
    pub a: TermId,
    pub b: TermId,
    pub explain: ReasonId,
}
```

We added `Session::take_eqshare_events()` (drains trace).

### 7.4 Ergonomic macros

To keep tests readable:

* `assert_eqshare_events_empty!(&events)`
* `assert_eqshare_dir!(&events, UF => DL)`
* `assert_eqshare_dir_none!(&events, DL => UF)`
* `assert_eqshare_hop!(&events, UF => DL, a, b)`
* `assert_eqshare_hop_any!(&events, UF => DL, [(a,b), (c,d)])`
* `assert_eqshare_hop_none!(&events, UF => DL, [(a,b), ...])`

We also added a `prelude` module for tests:

```rust
use crate::prelude::*;
```

---

## 8. Debugging toolkit (logs + Graphviz)

When combination breaks, you want fast diagnosis.

### 8.1 Equality-sharing debug logs

We added a debug config:

* logs shared-term count per epoch
* logs first-time-seen exports/imports with:

  * direction label (`UF→DL`, `DL→UF`)
  * epoch
  * terms `a=b`
  * a bounded reason summary (literals)

To prevent spam we keep an engine-level “seen key” set.

### 8.2 Reason DAG Graphviz dump

We can dump any `ReasonId` root to DOT:

* AND nodes are boxes
* Atom/literal leaves are ellipses labeled `vNN` / `¬vNN`
* bounded traversal (`max_reason_nodes`)

Output: `conflict.dot`

### 8.3 Equality-sharing Graphviz dump

We dump the trace into a DOT graph of term nodes + equality edges:

* edges labeled with `src→dst @epoch` + short reason summary
* optionally include `ReasonId` nodes

Output: `eqshare.dot`

### 8.4 UNSAT debug bundle writer

On UNSAT (or unexpected mismatch), when debug is enabled we write:

* `target/smt-debug-<pid>-<epoch>/eqshare.dot`
* `target/smt-debug-<pid>-<epoch>/conflict.dot`
* `target/smt-debug-<pid>-<epoch>/README.txt` with rendering commands

Rendering:

```bash
dot -Tsvg eqshare.dot  > eqshare.svg
dot -Tsvg conflict.dot > conflict.svg
```

### 8.5 Bundle-on-failure testing

To avoid overhead on passing tests:

1. run with debug OFF
2. if result/core assertions fail, enable debug
3. re-run the same instance (fresh session) to produce the bundle
4. panic with a message pointing to the bundle folder

---

## 9. Test-only debug enablement across crates

We added a `smt-engine` feature `test-debug`:

* exposes global toggle:

  * `set_eqshare_debug_enabled(bool)`
  * `eqshare_debug_enabled() -> bool`
* `SmtEngine::new()` checks it and turns on debug config automatically
* `smt-tests` uses an RAII guard (or explicit enabling in helpers)

This makes it possible to:

* enable debug in `smt-engine` unit tests,
* enable debug in `smt-tests`,
* keep production builds clean.

---

## 10. Key design decisions

### 10.1 Why the oracle is in the engine

Shared-term detection affects correctness. If each theory computed “sharedness” independently, you’d get:

* inconsistencies,
* hard-to-debug missing exports,
* nondeterminism.

Centralizing it yields stable combination behavior.

### 10.2 Why UF exports implied app equalities

Combination requires exchanging equalities that are true in a theory, not only those asserted.
The `a=b ⇒ f(a)=f(b)` step is essential to make UF→DL work.

### 10.3 Why include UF app args in endpoints

Otherwise, DL would never consider `x` and `y` shared if they appear only as `g(x)` in UF atoms.
This is a classic subtlety in Nelson–Oppen integration.

### 10.4 Why spanning-tree exports

Exporting all pairs inside an equivalence class is quadratic.
A spanning tree is sufficient: other theories can reconstruct equality closure.

### 10.5 Why export dedup needs rollback

Because CDCL backtracks. If you “remember forever” that a pair was exported, you might skip necessary exports in a different branch. Trails + checkpoints solve this.

---

## 11. Current state checklist

**Correctness**

* ✅ UF→DL regression passes with UF→DL enabled
* ✅ DL→UF regression passes with DL→UF enabled
* ✅ Ping-pong passes only with both directions enabled
* ✅ Shared terms include UF app args, enabling DL to see `x,y`

**Performance / stability**

* ✅ Epoch throttling
* ✅ Export pair dedup with rollback
* ✅ Spanning-tree exports per UF class
* ✅ Trace draining per `check_sat`

**Debuggability**

* ✅ Bundle-on-failure rerun
* ✅ eqshare.dot and conflict.dot generation
* ✅ log filtering / first-time seen suppression

---

## 12. Next natural extensions (optional roadmap)

* Add more theories (EUF + LIA, arrays, bitvectors) using the same oracle/round design.
* Improve DL solver incrementality (avoid APSP recomputation) while keeping proof reconstruction.
* Strengthen theory classifier (handle mixed atoms more carefully).
* Integrate proper pretty-printer for terms (makes DOT graphs much nicer).
* Add proof-producing SAT kernel or core-minimization if you want smaller cores.

