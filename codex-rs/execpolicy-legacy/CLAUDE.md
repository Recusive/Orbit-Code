# codex-rs/execpolicy-legacy/

Legacy argument-aware exec policy engine that understands per-program argument semantics (flags, options, positional args, sed commands) to determine whether a shell command is safe.

## Build & Test
```bash
cargo build -p orbit-code-execpolicy-legacy
cargo test -p orbit-code-execpolicy-legacy
```

## Architecture

Unlike the newer `execpolicy` (prefix-only matching), this engine parses and validates individual program arguments. Policies are written in a Starlark DSL that defines `ProgramSpec` objects describing a program's accepted flags, options, and positional arguments with type annotations (`ArgType`). The `ExecvChecker` matches an `ExecCall` (program + args) against a `ProgramSpec` and produces a `ValidExec` result that tracks which arguments were matched and whether the command `might_write_files()`.

A built-in `default.policy` (embedded via `include_str!`) defines safe commands like `ls`, `cat`, `grep`, `find`, etc. with detailed argument specs.

## Key Considerations
- The `default.policy` file is embedded at compile time via `include_str!` -- if you modify it, update `BUILD.bazel` `compile_data` too
- `might_write_files()` on `ValidExec` drives the safety classification -- any matched arg with a write-capable `ArgType` returns true
- The CLI binary uses exit codes: 0 (ok), 12 (matched but writes files), 13 (might be safe), 14 (forbidden)
- This crate depends on `allocative` for Starlark value memory tracking -- unusual compared to most workspace crates
- `regex-lite` is used instead of `regex` for argument pattern matching (lighter dependency)
