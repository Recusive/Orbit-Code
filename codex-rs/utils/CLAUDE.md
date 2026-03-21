# codex-rs/utils/

Collection of small, focused utility crates used across the Codex CLI workspace.

## What this folder does

This directory contains 19 independent Cargo library crates that provide shared functionality consumed by the core Codex crates (`orbit-code-core`, `orbit-code-tui`, `orbit-code-cli`, `orbit-code-exec`, `orbit-code-app-server`, etc.). Each subfolder is a standalone Rust crate published under the `codex-utils-*` naming convention (except `orbit-code-git`).

## Crate inventory

| Crate | Directory | Purpose |
|-------|-----------|---------|
| `orbit-code-utils-absolute-path` | `absolute-path/` | Guaranteed-absolute path type with serde support |
| `orbit-code-utils-approval-presets` | `approval-presets/` | Built-in approval + sandbox policy preset definitions |
| `orbit-code-utils-cache` | `cache/` | Thread-safe LRU cache with SHA-1 content hashing |
| `orbit-code-utils-cargo-bin` | `cargo-bin/` | Locate test binaries across Cargo and Bazel build systems |
| `orbit-code-utils-cli` | `cli/` | Shared CLI argument types for clap (approval mode, sandbox mode, config overrides) |
| `orbit-code-utils-elapsed` | `elapsed/` | Human-readable elapsed time formatting |
| `orbit-code-utils-fuzzy-match` | `fuzzy-match/` | Case-insensitive fuzzy subsequence matching with scoring |
| `orbit-code-git` | `git/` | Git operations: ghost commits, patch apply, branch merge-base, symlinks |
| `orbit-code-utils-home-dir` | `home-dir/` | Locate the Codex config home directory (`~/.codex` or `$CODEX_HOME`) |
| `orbit-code-utils-image` | `image/` | Image loading, resizing, and base64 encoding for LLM prompts |
| `orbit-code-utils-json-to-toml` | `json-to-toml/` | Convert `serde_json::Value` to `toml::Value` |
| `orbit-code-utils-oss` | `oss/` | OSS provider helpers for LM Studio and Ollama |
| `orbit-code-utils-pty` | `pty/` | PTY and pipe process spawning with cross-platform process group management |
| `orbit-code-utils-readiness` | `readiness/` | Async readiness flag with token-based authorization |
| `orbit-code-utils-rustls-provider` | `rustls-provider/` | One-time rustls crypto provider initialization |
| `orbit-code-utils-sandbox-summary` | `sandbox-summary/` | Human-readable summaries of sandbox and config policies |
| `orbit-code-utils-sleep-inhibitor` | `sleep-inhibitor/` | Prevent system sleep during active turns (macOS/Linux/Windows) |
| `orbit-code-utils-stream-parser` | `stream-parser/` | Incremental streaming text parsers for citations, plans, and UTF-8 |
| `orbit-code-utils-string` | `string/` | String utilities: byte-boundary truncation, UUID finding, metric sanitization |

## Imports / exports

- These crates are consumed by `orbit-code-core`, `orbit-code-tui`, `orbit-code-cli`, `orbit-code-exec`, `orbit-code-app-server`, and each other
- Several depend on `orbit-code-protocol` for shared protocol types
- `orbit-code-utils-oss` depends on `orbit-code-core`, `orbit-code-lmstudio`, and `orbit-code-ollama`
- `orbit-code-utils-sandbox-summary` depends on `orbit-code-core` and `orbit-code-protocol`
- External dependencies include `tokio`, `serde`, `clap`, `image`, `portable-pty`, `lru`, and platform-specific system libraries
