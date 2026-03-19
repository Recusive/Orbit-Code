# codex-rs/.cargo/

Cargo configuration directory for the codex-rs workspace.

## What this folder does

Contains Cargo build configuration files that apply to the entire workspace. These control linker flags and security advisory suppressions.

## Key files

- `config.toml` -- Platform-specific linker flags:
  - **Windows MSVC** (`cfg(all(windows, target_env = "msvc"))`): Sets 8 MB stack size via `/STACK:8388608`
  - **Windows MSVC aarch64**: Adds `/arm64hazardfree` to suppress Cortex-A53 bug #843419 warnings
  - **Windows GNU** (`cfg(all(windows, target_env = "gnu"))`): Sets 8 MB stack size via `-Wl,--stack,8388608`
- `audit.toml` -- `cargo-audit` advisory suppressions for transitive dependencies that cannot be updated yet:
  - `RUSTSEC-2024-0388` (derivative via starlark)
  - `RUSTSEC-2025-0057` (fxhash via starlark_map)
  - `RUSTSEC-2024-0436` (paste via starlark/ratatui)

## What it plugs into

- Cargo reads `config.toml` automatically when building any crate in the workspace
- `cargo audit` reads `audit.toml` to filter known advisory false positives
- The `.github/workflows/cargo-audit.yml` CI workflow runs `cargo audit --deny warnings` against this configuration

## Imports from / exports to

- No code imports; these are Cargo build-system configuration files
- Consumed by Cargo at build time and by `cargo-audit` at audit time
