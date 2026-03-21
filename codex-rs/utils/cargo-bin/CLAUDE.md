# codex-rs/utils/cargo-bin/

Locates compiled test binaries and test resources at runtime, transparently supporting both Cargo (`CARGO_BIN_EXE_*` env vars) and Bazel (rlocationpaths via runfiles).

## Build & Test
```bash
cargo build -p orbit-code-utils-cargo-bin
cargo test -p orbit-code-utils-cargo-bin
```

## Key Considerations
- Use `cargo_bin("name")` instead of hardcoding binary paths in tests.
- Use `find_resource!` macro instead of `env!("CARGO_MANIFEST_DIR")` for test fixtures -- it works under both Cargo and Bazel.
