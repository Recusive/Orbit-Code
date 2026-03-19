# codex-rs/shell-command/src/command_safety/

Command safety classification logic -- determines if commands are safe to auto-approve or dangerous enough to warn about.

## What this folder does

Contains the "known safe" and "known dangerous" command classifiers. The safe classifier maintains an allowlist of read-only commands (ls, cat, grep, git status, etc.) with per-command argument validation. The dangerous classifier flags destructive operations (rm -rf, sudo rm -f). Both classifiers handle `bash -lc "..."` wrappers by parsing the inner script.

## Key files

- `mod.rs` -- re-exports `is_safe_command` and `is_dangerous_command` submodules, plus `windows_safe_commands`.
- `is_safe_command.rs` -- `is_known_safe_command()` checks commands against an allowlist. Handles shell wrappers (`bash -lc`, `zsh -lc`), composite scripts (parsing each subcommand), and per-tool argument validation: `git` (read-only subcommands, config-override rejection), `find` (rejects -exec/-delete), `rg` (rejects --pre/--search-zip), `base64` (rejects -o/--output), `sed -n Np` patterns.
- `is_dangerous_command.rs` -- `command_might_be_dangerous()` flags `rm -f`/`rm -rf` and `sudo` wrappers. Also provides shared helpers: `executable_name_lookup_key()` (normalizes executable names, strips Windows extensions), `find_git_subcommand()` (skips git global options to find the first positional subcommand).
- `windows_safe_commands.rs` -- Windows-specific safe command classification for PowerShell commands (Get-ChildItem, Get-Content, Get-Location, etc.).
- `windows_dangerous_commands.rs` -- Windows-specific dangerous command detection.
- `powershell_parser.ps1` -- PowerShell AST parser script for command classification.

## Imports from

- `crate::bash::parse_shell_lc_plain_commands` for shell script parsing.
- `crate::command_safety::is_dangerous_command::executable_name_lookup_key` and `find_git_subcommand` (shared between safe and dangerous classifiers).

## Exports to

- `is_known_safe_command()` and `command_might_be_dangerous()` are the primary public API, re-exported from `codex-shell-command`.
