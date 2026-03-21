# codex-rs/shell-command/

Shell command parsing and safety classification library. Determines whether commands proposed by the AI agent can be auto-approved or need user confirmation.

## Build & Test
```bash
cargo build -p orbit-code-shell-command
cargo test -p orbit-code-shell-command
```

## Architecture

The crate parses shell commands using tree-sitter-bash for robust AST-based analysis, then classifies them as "known safe" (auto-approvable) or "known dangerous" (requiring a warning). The `bash` module handles bash/zsh parsing including `bash -lc "..."` unwrapping and heredoc scripts. The `powershell` module handles PowerShell command extraction and executable discovery. The `command_safety` submodule contains the actual classification logic with platform-specific safe/dangerous command lists.

## Key Considerations
- tree-sitter-bash is used for parsing -- `try_parse_shell()` returns a syntax tree, not regex matching
- `is_safe_command` checks are conservative: a command must be provably read-only to be classified as safe
- `is_dangerous_command` checks are inclusive: any command that *might* cause destructive side effects is flagged
- The `command_safety/` submodule has separate files for Windows vs Unix safe/dangerous command lists
- `shlex` is used for shell word splitting as a fallback when tree-sitter parsing fails
- Shell type detection (`shell_detect.rs`) maps executable paths to `ShellType` enum (Zsh, Bash, Sh, PowerShell, Cmd)
