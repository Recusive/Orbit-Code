# codex-rs/arg0/src/

Argv[0]-based dispatch and process bootstrapping. Allows a single binary to behave as multiple tools (`apply_patch`, `orbit-code-linux-sandbox`, `codex-execve-wrapper`) via symlinks, and handles PATH setup, dotenv loading, and Tokio runtime construction.

## Build & Test
```bash
cargo build -p orbit-code-arg0
cargo test -p orbit-code-arg0
```

## Key Considerations
- Creates a temp directory under `~/.codex/tmp/arg0/` with symlinks (Unix) or `.bat` wrappers (Windows) and prepends it to PATH so child processes find helper tools.
- Loads `~/.codex/.env` but filters out `CODEX_`-prefixed variables for security.
- Tokio runtime uses 16 MB worker stack size.
- A janitor cleans stale temp directories from previous sessions using file locks.
