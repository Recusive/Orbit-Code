# codex-rs/utils/pty/

PTY and pipe-based process spawning with cross-platform process group management. Provides unified `ProcessHandle`/`SpawnedProcess` abstractions with stdin/stdout/stderr multiplexing via Tokio channels.

## Build & Test
```bash
cargo build -p orbit-code-utils-pty
cargo test -p orbit-code-utils-pty
```

## Key Considerations
- Uses `edition = "2021"` (not the workspace default 2024) due to platform-specific compatibility constraints.
- Windows ConPTY implementation is vendored from WezTerm in `src/win/`.
- Process group management (`kill_process_group`, `set_parent_death_signal`) is Unix-only; all functions are no-ops on Windows.
