# Debugging Logs

## Log File Location

The TUI writes logs to `~/.orbit/log/codex-tui.log`.

The file is **truncated on each CLI launch** — only the current session's logs are preserved.

## Viewing Logs

Terminal 2 (start first):
```bash
tail -f ~/.orbit/log/codex-tui.log
```

Terminal 1 (start after):
```bash
RUST_LOG=debug cargo run --bin orbit-code
```

## Log Levels

Without `RUST_LOG`, the default filter is:
```
orbit_code_core=info,orbit_code_tui=info,orbit_code_rmcp_client=info
```

Override with `RUST_LOG`:

```bash
# Debug for all orbit-code crates (recommended)
RUST_LOG=orbit_code_core=debug cargo run --bin orbit-code

# Debug for specific modules
RUST_LOG=orbit_code_core::models_manager=debug,orbit_code_core::anthropic_bridge=debug cargo run --bin orbit-code

# Everything (very verbose — includes HTTP, TLS, tokio)
RUST_LOG=debug cargo run --bin orbit-code

# Trace level (maximum verbosity)
RUST_LOG=trace cargo run --bin orbit-code
```

## Filtering Logs

```bash
# Model/context window activity
tail -f ~/.orbit/log/codex-tui.log | grep -i "model\|context_window\|catalog"

# Auth and provider activity
tail -f ~/.orbit/log/codex-tui.log | grep -i "auth\|provider\|anthropic"

# Turn lifecycle
tail -f ~/.orbit/log/codex-tui.log | grep "turn\|submission_dispatch"
```

## Log Format

Logs use `tracing` with targets enabled:
```
TIMESTAMP LEVEL SPAN_CONTEXT: TARGET: message key=value
```

Example:
```
2026-03-23T22:46:33.652Z INFO list_models{refresh_strategy=offline}: orbit_code_core::models_manager::manager: models cache: cache hit
```

## Other Log Tools

- `just log` — tails the state SQLite database (OpenTelemetry metrics only, not app logs)
- `~/.orbit/log/codex-tui.log` — the actual app log file (use this)

## Implementation

Log setup: `tui_app_server/src/lib.rs` (lines 781-812) and `tui/src/lib.rs` (mirrored).
Home dir resolution: `utils/home-dir/src/lib.rs` — resolves to `~/.orbit`.
