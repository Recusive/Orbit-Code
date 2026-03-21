# codex-rs/skills/

Embedded system skills: compiles skill assets (agent configs, reference docs, scripts) into the binary and extracts them to `CODEX_HOME/skills/.system` on startup.

## Build & Test
```bash
cargo build -p orbit-code-skills
cargo test -p orbit-code-skills
```

## Architecture

The crate uses `include_dir` to embed the contents of `src/assets/samples/` at compile time. On startup, `install_system_skills()` computes a fingerprint (hash of all embedded files) and compares it against a marker file in the target directory. If the fingerprint differs, it extracts all embedded files to `CODEX_HOME/skills/.system`, overwriting stale content. If the fingerprint matches, extraction is skipped entirely.

## Key Considerations

- `build.rs` emits `cargo:rerun-if-changed` for all files under `src/assets/samples/` -- adding or modifying skill files triggers a rebuild.
- The `include_dir` dependency means all skill assets are compiled into the binary -- large files increase binary size directly.
- Fingerprint-based caching means skills are only re-extracted when the embedded content changes, not on every startup.
- If using Bazel, embedded asset files must be listed in `BUILD.bazel` `compile_data` or the build will fail even though Cargo passes.
