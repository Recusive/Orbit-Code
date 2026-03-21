# codex-rs/apply-patch/src/

Parses and applies a custom patch format (`*** Begin Patch ... *** End Patch`) to the filesystem. Serves as both a library (consumed by `orbit-code-core` for verified patch application and diff computation) and a standalone CLI binary (`apply_patch`).

## Build & Test
```bash
cargo build -p orbit-code-apply-patch
cargo test -p orbit-code-apply-patch
```

## Key Considerations
- The patch format supports Add File, Delete File, and Update File (with optional move/rename) -- distinct from standard unified diff.
- Uses tree-sitter with a Bash grammar to detect `apply_patch` heredoc invocations from shell commands.
- `apply_patch_tool_instructions.md` is embedded at compile time via `include_str!` -- if you modify it, update `BUILD.bazel` `compile_data`.
- Fuzzy line matching in `seek_sequence.rs` uses four levels of decreasing strictness (exact, rstrip, trim, Unicode-normalized).
