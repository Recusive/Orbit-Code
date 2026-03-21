# shell-tool-mcp/patches/

Source patches for Bash and Zsh that add `EXEC_WRAPPER` environment variable support. When `EXEC_WRAPPER` is set, the patched shell prepends the wrapper binary and the original command path to the `execve(2)` argument list, allowing an external program to intercept and decide whether to allow each process execution.
