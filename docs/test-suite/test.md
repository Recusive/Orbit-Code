# Testing Guide — Orbit Code

## Quick Reference

```bash
# Full test suite (from repo root)
just test

# Full test suite with macOS-safe settings (4 threads, 5min timeout)
cd codex-rs && cargo nextest run --no-fail-fast -P slow

# Single crate
cargo test -p <crate-name>

# Single crate, unit tests only (fastest)
cargo test -p <crate-name> --lib

# Single test by name
cargo test -p <crate-name> -- <test_name>
```

---

## macOS: Avoiding `syspolicyd` Crashes

macOS has a security daemon (`syspolicyd`) that scans every new process binary before allowing it to run. This repo has **7000+ tests** across **134 binaries**. Running them all at full parallelism causes `syspolicyd` to spike to 1000%+ CPU, which:

- Starves test processes of CPU time
- Causes mass 30-second timeouts (fake failures)
- Can freeze terminals and exhaust PTY resources
- Gets worse after a reboot (cache is cold)

### The `slow` Profile

A nextest profile at `codex-rs/.config/nextest.toml` is configured for macOS development:

```toml
[profile.slow]
slow-timeout = { period = "300s", terminate-after = 2 }
test-threads = 4
```

This runs 4 tests at a time (instead of all CPUs) and gives each test 5 minutes instead of 30 seconds. Tests still finish in <1 second normally — the timeout only matters when `syspolicyd` is active.

```bash
# Use the slow profile
cd codex-rs && cargo nextest run --no-fail-fast -P slow

# Bump to 8 threads once syspolicyd settles
cd codex-rs && cargo nextest run --no-fail-fast -P slow -j 8
```

### Rules for AI Assistants

1. **Never run multiple test suites simultaneously** — this triggers `syspolicyd` storms
2. **Never use `script` command** — corrupts terminal state system-wide
3. **Never pipe nextest output** — nextest needs a real terminal (TTY) for its UI; piping through `tee`, `tail`, or redirecting to files produces empty output
4. **For programmatic verification**, use `cargo test -p <crate> --lib` — this works without a TTY
5. **For full suite verification**, tell the user to run `just test` or `cargo nextest run --no-fail-fast -P slow` in their terminal and paste results
6. **After full recompilation**, the first test run is always slow (`syspolicyd` scanning). Second run is fast.
7. **If `syspolicyd` spikes above 500%**, reduce test threads with `-j 4` or `-j 2`

---

## Testing by Crate

All commands run from `codex-rs/`.

### Foundation Layer

| Crate | Command | Tests | Notes |
|-------|---------|-------|-------|
| `orbit-code-utils-home-dir` | `cargo test -p orbit-code-utils-home-dir` | ~4 | Home directory resolution |
| `orbit-code-protocol` | `cargo test -p orbit-code-protocol` | ~125 | Core types, sandbox policies, permissions |
| `orbit-code-config` | `cargo test -p orbit-code-config` | ~20 | TOML config parsing |
| `orbit-code-hooks` | `cargo test -p orbit-code-hooks` | ~20 | Lifecycle hook execution |
| `orbit-code-secrets` | `cargo test -p orbit-code-secrets` | ~10 | Encrypted secrets |
| `orbit-code-execpolicy` | `cargo test -p orbit-code-execpolicy` | ~40 | Execution policy rules |

### Engine Layer

| Crate | Command | Tests | Notes |
|-------|---------|-------|-------|
| `orbit-code-core` (unit) | `cargo test -p orbit-code-core --lib` | ~1854 | Agent loop, config, auth, tools, skills |
| `orbit-code-core` (integration) | `cargo test -p orbit-code-core --test all` | ~816 | End-to-end with mock servers (slow) |
| `orbit-code-app-server-protocol` | `cargo test -p orbit-code-app-server-protocol` | ~137 | JSON-RPC types, schema validation |
| `orbit-code-app-server` (integration) | `cargo test -p orbit-code-app-server --test all` | ~80 | App-server integration tests |

### Consumer Layer

| Crate | Command | Tests | Notes |
|-------|---------|-------|-------|
| `orbit-code-tui` (unit) | `cargo test -p orbit-code-tui --lib` | ~1342 | Terminal UI unit tests |
| `orbit-code-tui` (integration) | `cargo test -p orbit-code-tui --test all` | ~5 | TUI integration tests |
| `orbit-code-tui-app-server` | `cargo test -p orbit-code-tui-app-server` | ~3 | TUI app-server variant |
| `orbit-code-exec` | `cargo test -p orbit-code-exec` | ~50 | Headless CLI |
| `orbit-code-mcp-server` | `cargo test -p orbit-code-mcp-server` | ~15 | MCP protocol server |

### Utility Crates

| Crate | Command | Tests | Notes |
|-------|---------|-------|-------|
| `orbit-code-shell-command` | `cargo test -p orbit-code-shell-command` | ~80 | Shell command parsing/safety |
| `orbit-code-shell-escalation` | `cargo test -p orbit-code-shell-escalation` | ~15 | Shell privilege escalation |
| `orbit-code-state` | `cargo test -p orbit-code-state` | ~50 | SQLite state/rollout |
| `orbit-code-skills` | `cargo test -p orbit-code-skills` | ~5 | Skill asset fingerprinting |
| `orbit-code-rmcp-client` | `cargo test -p orbit-code-rmcp-client` | ~30 | MCP client |
| `orbit-code-login` | `cargo test -p orbit-code-login` | ~15 | OAuth/auth login flows |
| `orbit-code-git` | `cargo test -p orbit-code-git` | ~25 | Git operations |
| `orbit-code-otel` | `cargo test -p orbit-code-otel` | ~30 | OpenTelemetry |

### Platform Sandboxes

| Crate | Command | Platform | Notes |
|-------|---------|----------|-------|
| `orbit-code-core` seatbelt | `cargo test -p orbit-code-core -- seatbelt` | macOS only | Seatbelt sandbox tests |
| `orbit-code-linux-sandbox` | `cargo test -p orbit-code-linux-sandbox` | Linux only | Landlock/bwrap tests |
| `orbit-code-windows-sandbox-rs` | `cargo test -p orbit-code-windows-sandbox-rs` | Windows only | ACL tests |

### SDKs

```bash
# TypeScript
cd sdk/typescript && pnpm install && pnpm test

# Python
cd sdk/python && pip install -e . && pytest -q
```

---

## Testing After Changes

### After modifying config types
```bash
just write-config-schema
cargo test -p orbit-code-core -- config
```

### After modifying app-server protocol
```bash
just write-app-server-schema
just write-app-server-schema --experimental
cargo test -p orbit-code-app-server-protocol
```

### After modifying TUI
```bash
cargo test -p orbit-code-tui --lib
cargo insta pending-snapshots -p orbit-code-tui
cargo insta accept -p orbit-code-tui  # if snapshots changed
```

### After modifying sandbox/permissions
```bash
cargo test -p orbit-code-protocol -- permissions
cargo test -p orbit-code-core -- seatbelt  # macOS
```

### Pre-commit checklist
```bash
just fmt
just fix -p <changed-crate>
cargo test -p <changed-crate>
```

### Full verification (before merge)
```bash
just fmt
just fix
just test  # or: cargo nextest run --no-fail-fast -P slow
```

---

## Known Pre-existing Test Failures

| Test | Status | Reason |
|------|--------|--------|
| `export::tests::generated_ts_optional_nullable_fields_only_in_params` | FAIL | `AccountUpdatedNotification.ts` has optional nullable fields outside `*Params` |
| `suite::skills::list_skills_includes_system_cache_entries` | FAIL | Skill scope classification returns `User` instead of `System` |

These failures exist on `main` before any changes.

---

## Snapshot Tests

TUI changes often require snapshot updates:

```bash
# Run tests — failing snapshots create .snap.new files
cargo test -p orbit-code-tui

# Review pending snapshots
cargo insta pending-snapshots -p orbit-code-tui

# Preview a specific snapshot
cargo insta show -p orbit-code-tui <path.snap.new>

# Accept all pending snapshots
cargo insta accept -p orbit-code-tui
```

Install `cargo-insta` if missing: `cargo install cargo-insta`
