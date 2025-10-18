# Repository Guidelines

## Project Structure & Module Organization
dntk is a Rust CLI wrapper around GNU bc. `src/main.rs` wires terminal setup and launches the `Dntker` state machine in `src/dntker/`. Terminal adapters live in `src/term/`, and shared exports sit in `src/lib.rs`. Integration tests are under `tests/` (`tests/e2e.rs` exercises CLI flows with `assert_cmd`), while CI scripts and packaging helpers live in `ci/`, `Makefile`, and shell utilities like `release.sh`.

## Build, Test, and Development Commands
- `cargo build --release`: produce an optimized binary; run this before cutting releases.
- `cargo run --release`: run the CLI locally with production flags; use `DNTK_ENV=TEST` for predictable output.
- `cargo test`: execute unit and integration tests; CI runs this for each target.
- `cargo clippy -- -D warnings`: match CI linting; fix warnings before pushing.
- `cargo fmt`: standardize formatting; ensure diffs stay minimal.
- `make run`, `make test`, `make release`: convenience aliases wrapping the commands above.

## Coding Style & Naming Conventions
Rust 2018 edition defaults apply with 4-space indentation. Keep modules and files snake_case (`src/dntker/dntker.rs`) and types in UpperCamelCase. Prefer immutable bindings and `?` error propagation; add targeted comments only where logic differs by platform. Always format with `cargo fmt` and check lints with `cargo clippy -- -D warnings` before review.

## Testing Guidelines
Write integration tests under `tests/` using `assert_cmd` to spawn the compiled binary; mimic existing `test_cmd_with_*` patterns. Name tests with descriptive verbs (`test_cmd_with_show_limits`) and reset state via `DNTK_ENV=TEST` when output should be deterministic. When adding platform-dependent assertions, gate them with `cfg(target_os)` like the current suite. Coverage is not enforced, but new CLI options should include at least one regression test.

## Commit & Pull Request Guidelines
History favors short, lowercase, imperative subjects (`readme update`, `release 2.2.1`). Follow that tone, scope commits narrowly, and reference issues with `#id` when relevant. Pull requests should list the main changes, note any platform-specific impact, and describe how to reproduce verification (commands run, targets exercised). Include screenshots or terminal captures only when behavior changes are user-facing.
