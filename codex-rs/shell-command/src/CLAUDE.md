# codex-rs/shell-command/src/

Source for the `orbit-code-shell-command` crate -- tree-sitter-based shell parsing with safety classification.

## Module Layout
- **Bash parsing** (`bash.rs`): tree-sitter-bash parsing with `try_parse_shell()`, `try_parse_word_only_commands_sequence()`, `extract_bash_command()`, heredoc handling
- **PowerShell** (`powershell.rs`): PowerShell command extraction, UTF-8 output prefix injection, `pwsh`/`powershell` executable discovery
- **Safety classification** (`command_safety/`): `mod.rs` with `is_safe_command` and `is_dangerous_command`; platform-specific safe/dangerous command lists (`is_safe_command.rs`, `is_dangerous_command.rs`, `windows_safe_commands.rs`, `windows_dangerous_commands.rs`)
- **Utilities** (`shell_detect.rs`, `parse_command.rs`): Shell type detection from executable paths; additional command parsing helpers
