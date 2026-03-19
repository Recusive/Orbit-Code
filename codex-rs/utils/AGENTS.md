# codex-rs/utils/

This file applies to `codex-rs/utils/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Collection of small, focused utility crates used across the Codex CLI workspace.

### What this folder does

This directory contains 19 independent Cargo library crates that provide shared functionality consumed by the core Codex crates (`codex-core`, `codex-tui`, `codex-cli`, `codex-exec`, `codex-app-server`, etc.). Each subfolder is a standalone Rust crate published under the `codex-utils-*` naming convention (except `codex-git`).

### Crate inventory

| Crate | Directory | Purpose |
|-------|-----------|---------|
| `codex-utils-absolute-path` | `absolute-path/` | Guaranteed-absolute path type with serde support |
| `codex-utils-approval-presets` | `approval-presets/` | Built-in approval + sandbox policy preset definitions |
| `codex-utils-cache` | `cache/` | Thread-safe LRU cache with SHA-1 content hashing |
| `codex-utils-cargo-bin` | `cargo-bin/` | Locate test binaries across Cargo and Bazel build systems |
| `codex-utils-cli` | `cli/` | Shared CLI argument types for clap (approval mode, sandbox mode, config overrides) |
| `codex-utils-elapsed` | `elapsed/` | Human-readable elapsed time formatting |
| `codex-utils-fuzzy-match` | `fuzzy-match/` | Case-insensitive fuzzy subsequence matching with scoring |
| `codex-git` | `git/` | Git operations: ghost commits, patch apply, branch merge-base, symlinks |
| `codex-utils-home-dir` | `home-dir/` | Locate the Codex config home directory (`~/.codex` or `$CODEX_HOME`) |
| `codex-utils-image` | `image/` | Image loading, resizing, and base64 encoding for LLM prompts |
| `codex-utils-json-to-toml` | `json-to-toml/` | Convert `serde_json::Value` to `toml::Value` |
| `codex-utils-oss` | `oss/` | OSS provider helpers for LM Studio and Ollama |
| `codex-utils-pty` | `pty/` | PTY and pipe process spawning with cross-platform process group management |
| `codex-utils-readiness` | `readiness/` | Async readiness flag with token-based authorization |
| `codex-utils-rustls-provider` | `rustls-provider/` | One-time rustls crypto provider initialization |
| `codex-utils-sandbox-summary` | `sandbox-summary/` | Human-readable summaries of sandbox and config policies |
| `codex-utils-sleep-inhibitor` | `sleep-inhibitor/` | Prevent system sleep during active turns (macOS/Linux/Windows) |
| `codex-utils-stream-parser` | `stream-parser/` | Incremental streaming text parsers for citations, plans, and UTF-8 |
| `codex-utils-string` | `string/` | String utilities: byte-boundary truncation, UUID finding, metric sanitization |

### Imports / exports

- These crates are consumed by `codex-core`, `codex-tui`, `codex-cli`, `codex-exec`, `codex-app-server`, and each other
- Several depend on `codex-protocol` for shared protocol types
- `codex-utils-oss` depends on `codex-core`, `codex-lmstudio`, and `codex-ollama`
- `codex-utils-sandbox-summary` depends on `codex-core` and `codex-protocol`
- External dependencies include `tokio`, `serde`, `clap`, `image`, `portable-pty`, `lru`, and platform-specific system libraries
