# TODO: Arbitrary-Precision Floating-Point Engine

## Goal
- Replace the current `f64` execution engine with a true arbitrary-precision floating-point backend so that expressions such as `1/3` yield exact decimal expansions (subject to configured precision) and advanced functions (`sin`, `log`, etc.) benefit from configurable precision.

## Proposed Direction
1. **Backend Choice**
   - Adopt `rug::Float` (MPFR bindings) to gain configurable precision, round modes, and a complete math function set.
   - Evaluate binary size & dependencies (MPFR/GMP) implications for release artifacts.

2. **Precision Management**
   - Map `scale=` to both output formatting and internal precision:
     - Compute required bits (e.g., `scale * log2(10) + margin`) for `Float::with_val`.
     - Allow overriding via env/CLI if higher precision is desired.

3. **Engine Refactor**
   - Replace `f64` values in `BcExecuter`, histories, variable scopes, and functions with `Float`.
   - Update `StatementOutcome` to carry `Float`.
   - Re-implement built-ins (trig, logs, `length`, `scale`, `rand`, `srand`, etc.) using MPFR equivalents.

4. **Formatting & Bases**
   - Rework `format_result` to produce bc-style strings via `Float`:
     - Handle obase formatting (base 2–36) for both integer and fractional parts.
     - Maintain bc conventions like stripping leading zero before decimal.

5. **Randomness**
   - Decide on random strategy:
     - Use MPFR’s random functions (requires RNG state).
     - Or derive from deterministic integer RNG converted to `Float`.

6. **Testing & Docs**
   - Update test suite expectations for high precision (`scale>=20`) and fractional base outputs.
   - Document new precision semantics, dependency requirements, and performance considerations in README / CLI help.

## Open Questions
- Static vs dynamic linking for MPFR/GMP in release builds.
- Default precision (current `scale=20` vs larger default for better accuracy).
- Performance benchmarks & potential caching of precision contexts.
