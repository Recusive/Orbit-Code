# codex-rs/utils/approval-presets/src/

Defines built-in approval presets pairing an `AskForApproval` policy with a `SandboxPolicy` -- "Read Only", "Default" (auto), and "Full Access". Used by TUI and app-server for presenting approval mode choices.

## Build & Test
```bash
cargo build -p orbit-code-utils-approval-presets
cargo test -p orbit-code-utils-approval-presets
```
