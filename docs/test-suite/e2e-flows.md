# E2E Flow Testing — Scripted TUI Verification

## What It Does

`scripts/e2e-flow.sh` runs a scripted sequence of TUI interactions in tmux — typing commands, pressing keys, capturing screens, and asserting what should appear. Write a `.flow` file once, run it forever.

This is how we verify multi-step TUI flows: open a popup → navigate → select → verify → move on.

---

## Usage

```bash
./scripts/e2e-flow.sh <flow-file>
```

```bash
# Check model picker shows Claude models
./scripts/e2e-flow.sh scripts/flows/check-models.flow

# Check auth status
./scripts/e2e-flow.sh scripts/flows/check-auth.flow

# Send a prompt and verify response
./scripts/e2e-flow.sh scripts/flows/send-prompt.flow

# Full flow: select model → send prompt → check response
./scripts/e2e-flow.sh scripts/flows/select-claude-send-prompt.flow
```

Output shows PASS/FAIL for each assertion and captures every screen to `/tmp/e2e-flow/`.

---

## Flow Script Format

Flow files live in `scripts/flows/` with the `.flow` extension. One instruction per line:

| Command | Description | Example |
|---------|-------------|---------|
| `WAIT <seconds>` | Wait for UI to settle | `WAIT 10` |
| `TYPE <text>` | Type text into the TUI | `TYPE /model` |
| `ENTER` | Press Enter | `ENTER` |
| `ESCAPE` | Press Escape | `ESCAPE` |
| `KEY <key>` | Send a tmux key | `KEY Down`, `KEY C-c`, `KEY Tab` |
| `SELECT <n>` | Press Down n times then Enter | `SELECT 3` |
| `CAPTURE <label>` | Save screen to `<label>.txt` | `CAPTURE model-picker` |
| `EXPECT <text>` | Assert screen contains text (PASS/FAIL) | `EXPECT claude-opus` |
| `#` | Comment (ignored) | `# Check startup` |
| `---` | Visual separator (ignored) | `---` |

---

## Writing a Flow

### 1. Start with WAIT for startup

The TUI needs time to boot, load auth, fetch models. Always start with:

```
WAIT 8
CAPTURE startup
EXPECT Orbit Code
```

### 2. Open a popup via slash command

Type the command, wait for the popup to appear:

```
TYPE /model
WAIT 1
ENTER
WAIT 10
CAPTURE model-picker
```

> The first `WAIT 1` is for the autocomplete popup. `ENTER` selects the command. The second `WAIT 10` is for the model list to load from the API.

### 3. Assert what should be on screen

```
EXPECT claude-opus
EXPECT gpt-5
```

Each `EXPECT` captures the screen and checks if the text appears anywhere. PASS if found, FAIL if not.

### 4. Navigate and interact

```
KEY Down
KEY Down
ENTER
WAIT 2
CAPTURE after-selection
```

### 5. Clean up

```
ESCAPE
WAIT 1
```

The script automatically sends Ctrl+C and kills the tmux session when the flow finishes.

---

## Example Flows

### Verify startup
```
# scripts/flows/smoke-test.flow
WAIT 5
CAPTURE startup
EXPECT Orbit Code
EXPECT model
EXPECT directory
```

### Verify model picker
```
# scripts/flows/check-models.flow
WAIT 10
TYPE /model
ENTER
WAIT 10
CAPTURE model-picker
EXPECT claude-opus
EXPECT claude-sonnet
EXPECT gpt-5
ESCAPE
```

### Verify prompt → response
```
# scripts/flows/send-prompt.flow
WAIT 8
TYPE say hello in exactly 3 words
ENTER
WAIT 15
CAPTURE response
EXPECT hello
```

### Verify permissions popup
```
# scripts/flows/check-permissions.flow
WAIT 8
TYPE /permissions
WAIT 1
ENTER
WAIT 3
CAPTURE permissions
EXPECT permission
ESCAPE
```

---

## Timing Guide

| Action | Recommended Wait |
|--------|-----------------|
| App startup | 5-8s |
| Slash command popup | 1s |
| Model list (API fetch) | 10-15s |
| Auth/permissions (local) | 3s |
| Prompt response (API) | 15-30s |
| Navigation (Down/Up/Enter) | 0.5s (built into SELECT) |

If the system is under load, double the wait times. If a flow fails intermittently, increase the WAIT before the failing EXPECT.

---

## Output

All screen captures are saved to `/tmp/e2e-flow/`:

```
/tmp/e2e-flow/startup.txt
/tmp/e2e-flow/model-picker.txt
/tmp/e2e-flow/response.txt
/tmp/e2e-flow/expect-step-5.txt   # auto-saved on EXPECT
```

Final summary:

```
================================
  PASS: 3  |  FAIL: 0
  Captures saved to: /tmp/e2e-flow/
================================
```

Exit code 0 = all passed. Exit code 1 = at least one failure.

---

## When to Use

| Scenario | Tool |
|----------|------|
| Static UI check (single screen) | `scripts/e2e-capture.sh` |
| Multi-step TUI flow | `scripts/e2e-flow.sh` |
| Prompt → response (structured) | `orbit-code exec --json` |
| Backend function check | Unit tests / probe binary |
| Full regression | `just test` |

### After building a feature

1. Write a `.flow` file that exercises the feature
2. Run `./scripts/e2e-flow.sh scripts/flows/my-feature.flow`
3. All PASS? Done. Any FAIL? Fix and rerun.

### Available slash commands

```
/model         — model and reasoning picker
/fast          — toggle fast mode
/permissions   — permission mode selector
/experimental  — experimental features toggle
/skills        — skills list
/review        — code review
/rename        — rename thread
/new           — new chat
```

---

## Requirements

- `tmux` installed (`brew install tmux`)
- Built binary at `codex-rs/target/debug/orbit-code`
- Valid credentials in `~/.orbit/auth.json` (for API-dependent flows)
