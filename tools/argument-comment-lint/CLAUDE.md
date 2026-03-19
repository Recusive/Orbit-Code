# tools/argument-comment-lint/

## Purpose

A [Dylint](https://github.com/trailofbits/dylint) lint library that enforces Rust `/*param*/` argument comment conventions in the `codex-rs/` workspace. It catches mismatched comments and optionally requires comments on anonymous literal arguments (e.g., `None`, `true`, `false`, numeric literals).

## What It Does

Provides two lints:

- **`argument_comment_mismatch`** (warn by default): Validates that a `/*param*/` comment matches the resolved callee parameter name.
- **`uncommented_anonymous_literal_argument`** (allow by default, promoted to error by `run.sh`): Flags anonymous literal-like arguments without a preceding `/*param*/` comment. String and char literals are exempt.

## Key Files

| File | Role |
|------|------|
| `src/lib.rs` | Main lint implementation using `clippy_utils` and `dylint_linting` |
| `src/comment_parser.rs` | Parser for extracting `/*param*/` comments from source spans |
| `run.sh` | Wrapper script that invokes `cargo dylint` with correct flags, defaults to `--workspace --no-deps` against `codex-rs/Cargo.toml`, and promotes `uncommented_anonymous_literal_argument` to error via `DYLINT_RUSTFLAGS` |
| `Cargo.toml` | Declares the cdylib crate with dependencies on `clippy_utils` and `dylint_linting` |
| `rust-toolchain` | Pins the nightly Rust version required by the lint |
| `.cargo/config.toml` | Configures `dylint-link` as the linker for the lint crate |
| `ui/` | Test fixtures (`.rs` files and `.stderr` expected output) for `dylint_testing` |

## Running

```bash
# From repo root:
./tools/argument-comment-lint/run.sh -p codex-core
just argument-comment-lint -p codex-core

# Run tests:
cd tools/argument-comment-lint && cargo test
```

## Dependencies

- Requires `cargo-dylint` and `dylint-link` installed
- Requires the specific nightly Rust toolchain pinned in `rust-toolchain`
- Uses `clippy_utils` from the `rust-clippy` repository (pinned to a specific rev)

## Relationship to Other Directories

- Targets the `codex-rs/Cargo.toml` workspace by default
- Referenced by the root `justfile` as the `argument-comment-lint` task
