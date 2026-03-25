# E2E Capture — Visual Verification for Orbit Code

## What It Does

`scripts/e2e-capture.sh` launches orbit-code in a tmux session, sends a command, waits for the UI to render, captures the full terminal screen to a file, and prints it to stdout. One command, one shot.

This is how we verify features actually work — not by reading test output, but by seeing the real app.

---

## Usage

```bash
./scripts/e2e-capture.sh "<command>" [wait_seconds] [output_file]
```

| Argument | Default | Description |
|----------|---------|-------------|
| `command` | `""` (empty) | What to type into the TUI. Slash commands, text, anything. |
| `wait_seconds` | `5` | How long to wait after sending the command before capturing. |
| `output_file` | `/tmp/orbit-e2e-capture.txt` | Where to save the screen capture. |

---

## Examples

### Check startup screen
```bash
./scripts/e2e-capture.sh "" 5
```
Expected: Header with model, directory, version.

### Check model picker
```bash
./scripts/e2e-capture.sh "/model" 10
```
Expected: List of GPT + Claude models with current model highlighted.

> Use a longer wait (10-15s) for `/model` since it fetches from the API.

### Check auth status
```bash
./scripts/e2e-capture.sh "/auth" 5
```
Expected: Provider list showing OpenAI and Anthropic with auth status.

### Check skills list
```bash
./scripts/e2e-capture.sh "/skills" 5
```
Expected: List of available skills.

### Check permissions
```bash
./scripts/e2e-capture.sh "/permissions" 5
```
Expected: Current permission mode.

### Send a message
```bash
./scripts/e2e-capture.sh "explain what this repo does" 15
```
Expected: Agent response streaming in the TUI.

### Save to a specific file
```bash
./scripts/e2e-capture.sh "/auth" 5 ./docs/test-suite/captures/auth-screen.txt
```

---

## How It Works

1. **Builds** the binary if `codex-rs/target/debug/orbit-code` doesn't exist
2. **Launches** orbit-code in a detached tmux session (120x50 terminal)
3. **Waits 5 seconds** for startup (MCP servers, auth loading, model cache)
4. **Sends** the command as keystrokes + Enter
5. **Waits** the specified seconds for the UI to settle
6. **Captures** the full tmux pane to a file via `tmux capture-pane`
7. **Prints** the capture to stdout
8. **Kills** the tmux session

---

## When to Use

### After building a feature
```bash
# Built a new /skills command? Verify it:
./scripts/e2e-capture.sh "/skills" 5

# Does the screen show the skill list? Yes = done. No = fix it.
```

### After fixing a bug
```bash
# Fixed auth display? Check it:
./scripts/e2e-capture.sh "/auth" 5

# Does it show the correct provider info? Yes = done. No = keep fixing.
```

### After a migration or refactor
```bash
# Changed config paths? Verify startup:
./scripts/e2e-capture.sh "" 5

# Does it show the right model/directory? Yes = done. No = investigate.
```

### Quick smoke test
```bash
# Does the app even start?
./scripts/e2e-capture.sh "" 3
```

---

## Timing Guide

| Command | Recommended Wait | Why |
|---------|-----------------|-----|
| `""` (startup) | 3-5s | Just needs TUI to render |
| `/auth` | 5s | Reads local auth.json, no API call |
| `/model` | 10-15s | Fetches model list from API |
| `/skills` | 5s | Reads local skill files |
| `/permissions` | 3s | Local state only |
| Text message | 15-30s | Needs API round-trip for response |

If the system is under load (e.g. test suite running), double the wait times.

---

## Troubleshooting

### Screen shows "model: loading"
The model cache API call hasn't completed. Increase wait time or ensure no other heavy processes are running.

### tmux session not found
Another tmux session with the same name may be lingering. The script uses PID-based session names to avoid this, but you can manually clean up:
```bash
tmux kill-server
```

### Binary not found
The script auto-builds, but if compilation fails:
```bash
cd codex-rs && cargo build --bin orbit-code
```

### Empty capture
tmux needs a real terminal. If running from a CI/pipe context, this won't work — use unit tests instead.

---

## Requirements

- `tmux` installed (`brew install tmux`)
- Rust toolchain (for building the binary)
- `~/.orbit/auth.json` with valid credentials (for API-dependent commands)
