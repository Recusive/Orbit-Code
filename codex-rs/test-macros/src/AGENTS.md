# codex-rs/test-macros/src/

This file applies to `codex-rs/test-macros/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-test-macros` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-test-macros`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-test-macros` proc-macro crate.

### What this folder does

Contains the single-file implementation of the `#[large_stack_test]` procedural macro attribute.

### Key files

- `lib.rs` -- Complete implementation:
  - **Constants**: `LARGE_STACK_TEST_STACK_SIZE_BYTES` (16 MB)
  - **Macro entry point**: `large_stack_test(attr, item)` -- Parses the attributed function and delegates to `expand_large_stack_test`
  - **Expansion**: `expand_large_stack_test(item)`:
    - Strips `asyncness` from the function signature
    - For async functions: generates code that builds a Tokio `Builder::new_multi_thread()` runtime with 2 workers, then `block_on()` the async body
    - For sync functions: wraps the body directly
    - Spawns a named thread with 16 MB stack via `std::thread::Builder`
    - Joins the thread and re-raises panics via `std::panic::resume_unwind`
  - **Attribute filtering**: `filtered_attributes(attrs)`:
    - Removes `#[tokio::test]` attributes (the macro manages its own runtime)
    - Adds `#[test]` if neither `#[test]` nor `#[test_case]` is present
    - Preserves all other attributes
  - **Helper predicates**: `is_test_attr`, `is_test_case_attr`, `is_tokio_test_attr`
  - **Tests**: Verify attribute manipulation (adding `#[test]`, removing `#[tokio::test]`, preserving `#[test_case]`)

### Imports from / exports to

**Imports:**
- `proc_macro::TokenStream` -- Compiler interface
- `proc_macro2::TokenStream` -- Token manipulation
- `quote::quote` -- Code generation
- `syn::{ItemFn, Attribute, parse_macro_input, parse_quote}` -- Rust syntax parsing

**Exports:**
- `large_stack_test` -- The proc-macro attribute (via `#[proc_macro_attribute]`)
