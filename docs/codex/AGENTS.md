# docs/

This file applies to `docs/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-monorepo` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.
- Documentation changes should track the current code and command surface. Update examples when behavior or CLI flags change.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Project documentation for the Codex CLI, covering installation, configuration, contributing guidelines, internal design documents, and TUI implementation details.

### Key Files

| File | Role |
|------|------|
| `install.md` | Installation instructions for all platforms |
| `getting-started.md` | Quick-start guide for new users |
| `config.md` | Configuration file format and options |
| `example-config.md` | Example configuration snippets |
| `authentication.md` | API key and authentication setup |
| `contributing.md` | Contributor guidelines, PR process, and code standards |
| `CLA.md` | Contributor License Agreement |
| `agents_md.md` | Documentation about the AGENTS.md convention |
| `sandbox.md` | Sandboxing and security model |
| `exec.md` | Command execution subsystem |
| `execpolicy.md` | Execution policy and approval rules |
| `prompts.md` | System prompt configuration |
| `skills.md` | Skills system documentation |
| `slash_commands.md` | Slash command reference |
| `js_repl.md` | JavaScript REPL integration design doc |
| `license.md` | License information |
| `open-source-fund.md` | Open source fund acknowledgments |

#### TUI Design Documents

| File | Role |
|------|------|
| `tui-alternate-screen.md` | Design for alternate screen mode in the terminal UI |
| `tui-chat-composer.md` | Chat composer widget design and implementation |
| `tui-request-user-input.md` | User input request flow in the TUI |
| `tui-stream-chunking-review.md` | Review of streaming chunk handling |
| `tui-stream-chunking-tuning.md` | Performance tuning for stream chunking |
| `tui-stream-chunking-validation.md` | Validation of stream chunking behavior |
| `exit-confirmation-prompt-design.md` | Design for the exit confirmation dialog |

### Relationship to Other Directories

- Referenced by the root `README.md` for user-facing documentation links
- TUI design docs correspond to implementations in `codex-rs/tui/`
- Config documentation corresponds to `codex-rs/config/`
- Exec/sandbox docs correspond to `codex-rs/exec/`, `codex-rs/execpolicy/`, `codex-rs/linux-sandbox/`
