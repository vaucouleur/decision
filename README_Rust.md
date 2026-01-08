# SMT Workspace Scaffold (UF + Difference Logic)

This is a **scaffold** Rust workspace that matches the architecture discussed:
- `smt-core`: terms/sorts/context
- `smt-sat`: SAT literals/kernel interface (stub)
- `smt-engine`: atoms + theories + equality sharing plumbing (oracle, epochs, trace, DOT dumps)
- `smt-api`: `Session` convenience wrapper
- `smt-tests`: test helpers/macros + a small smoke test (regressions are provided as templates)

> This is intentionally minimal: it compiles and provides the code design/structure,
> but it is **not** a fully-fledged SMT solver yet.

## Build

```bash
cargo test
```

## Where to extend next

- Implement a real SAT kernel (or integrate an existing one)
- Implement real UF congruence closure with explanation
- Implement real DL solver / propagation
- Turn the golden regression templates into true solver tests once theories are complete
