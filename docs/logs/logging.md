# Logging & Debugging

## Home Directory

Logs live under the Orbit Code home directory. Resolution order:

1. `ORBIT_HOME` env var
2. `~/.orbit`

Check which one you're using:

```bash
ls -d ~/.orbit 2>/dev/null
```

## TUI Logs

The TUI writes tracing logs to `<home>/log/codex-tui.log` automatically.

**Tail in a second terminal while running:**

```bash
# Terminal 1 — watch logs
tail -f ~/.orbit/log/codex-tui.log

# Terminal 2 — run the CLI
cd codex-rs
cargo run -p orbit-code
```

**Increase verbosity with `RUST_LOG`:**

```bash
# Everything at debug level
RUST_LOG=debug cargo run -p orbit-code

# Target specific crates
RUST_LOG=orbit_code_core=trace,orbit_code_tui=debug cargo run -p orbit-code

# Only warnings and errors (quieter)
RUST_LOG=warn cargo run -p orbit-code
```

## Session Recording

Full event-level recording dumps every `Op` and `AppEvent` as JSONL — useful for debugging agent interactions, tool calls, and streaming issues.

```bash
# Terminal 2 — run with recording enabled
ORBIT_TUI_RECORD_SESSION=1 cargo run -p orbit-code

# Terminal 1 — tail the session log (pipe through jq for readability)
tail -f ~/.orbit/log/session-*.jsonl | jq .
```

Custom output path:

```bash
ORBIT_TUI_SESSION_LOG_PATH=/tmp/debug.jsonl ORBIT_TUI_RECORD_SESSION=1 cargo run -p orbit-code
```

The session log header includes `cwd`, `model`, `model_provider_id`, and `model_provider_name` for context.

## SSE Fixture Replay

Replay a saved SSE response file without hitting the API — useful for testing streaming and event parsing:

```bash
ORBIT_RS_SSE_FIXTURE=/path/to/saved-sse.txt cargo run -p orbit-code
```

## Login Logs

The login flow writes to a separate log file at `<home>/log/codex-login.log`:

```bash
tail -f ~/.orbit/log/codex-login.log
```

## Debug Logs Feature Flag

The TUI crate has a `debug-logs` cargo feature that gates additional verbose logging:

```bash
cargo run -p orbit-code --features orbit-code-tui/debug-logs
```

## Quick Reference

| What | Command |
|------|---------|
| Tail TUI logs | `tail -f ~/.orbit/log/codex-tui.log` |
| Tail login logs | `tail -f ~/.orbit/log/codex-login.log` |
| Record session events | `ORBIT_TUI_RECORD_SESSION=1 cargo run -p orbit-code` |
| Verbose tracing | `RUST_LOG=debug cargo run -p orbit-code` |
| Replay SSE fixture | `ORBIT_RS_SSE_FIXTURE=file.txt cargo run -p orbit-code` |
| Find log directory | `ls ~/.orbit/log/` |
