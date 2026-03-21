# codex-rs/skills/src/

Embedded skill installation logic and bundled skill assets.

## Module Layout

- **lib** (`lib.rs`) -- `install_system_skills()` extracts embedded skills to disk with fingerprint-based caching; `system_cache_root_dir()` returns the cache path; `embedded_system_skills_fingerprint()` computes content hash for change detection
- **assets/** -- Embedded skill packages (agent configurations, reference docs, scripts) compiled into the binary via `include_dir`
