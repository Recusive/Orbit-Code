# shell-tool-mcp/patches/

## Purpose

Source patches for Bash and Zsh that add `EXEC_WRAPPER` environment variable support. When `EXEC_WRAPPER` is set, the patched shell prepends the wrapper binary and the original command path to the `execve(2)` argument list, allowing an external program to intercept and decide whether to allow each process execution.

## Key Files

| File | Role |
|------|------|
| `bash-exec-wrapper.patch` | Patches Bash's `execute_cmd.c` (`shell_execve` function). When `EXEC_WRAPPER` is set and non-empty, it rewrites the `args` array to `[exec_wrapper, original_command, ...original_args]` and sets `command` to the wrapper path before calling `execve()`. |
| `zsh-exec-wrapper.patch` | Patches Zsh's `Src/exec.c` (`zexecve` function). Same approach: when `EXEC_WRAPPER` is set and non-empty, it prepends the wrapper and original path to the argv array, then calls `execve()` with the wrapper as the program. |

## How It Works

1. Before every `execve()` call, the patched shell checks `getenv("EXEC_WRAPPER")`
2. If set, the wrapper binary is invoked instead, receiving the original command path as its first argument followed by the original arguments
3. The wrapper (implemented in `codex-rs/shell-escalation/`) can then allow, deny, or escalate the command

## Relationship to Other Directories

- These patches are applied when building the shell binaries that ship in the `vendor/` directory of the npm package
- `codex-rs/shell-escalation/` implements the Rust binary that acts as the `EXEC_WRAPPER`
- The patched binaries are selected at runtime by `../src/bashSelection.ts`
