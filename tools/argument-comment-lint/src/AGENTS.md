# tools/argument-comment-lint/src/

This file applies to `tools/argument-comment-lint/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Rust source code for the Dylint argument comment lint library.

### Key Files

| File | Role |
|------|------|
| `lib.rs` | Main lint implementation. Registers two lints (`argument_comment_mismatch` and `uncommented_anonymous_literal_argument`) as late passes with the Rust compiler. Inspects function/method call expressions, resolves callee parameter names, extracts `/*param*/` comments from source spans, and emits diagnostics for mismatches or missing comments on anonymous literals. |
| `comment_parser.rs` | Extracts `/*...*/` block comments that immediately precede an argument expression. Parses the raw source text to find comments in the exact `/*param_name*/` format. |

### How It Works

1. For each function/method call in the AST, the lint resolves the callee's `DefId` and retrieves parameter names from the function signature
2. For each argument expression, `comment_parser.rs` extracts any preceding `/*...*/` comment
3. If a comment is present but does not match the parameter name, `argument_comment_mismatch` fires
4. If no comment is present and the argument is an "anonymous literal" (None, true, false, numeric literal), `uncommented_anonymous_literal_argument` fires

### Imports From

- `clippy_utils` (from rust-clippy): utility functions for working with the Rust compiler internals
- `dylint_linting`: framework for defining custom Dylint lints
- Standard `rustc_*` compiler crates (available because the crate uses `rustc_private`)

### Exports To

- Compiled as a `cdylib` that Dylint loads at lint time
- Test fixtures in `../ui/` validate the lint output
