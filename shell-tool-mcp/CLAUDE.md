# shell-tool-mcp/

MCP server (`@orbit.build/orbit-code-shell-tool-mcp`) that provides a sandboxed `shell` tool using patched Bash/Zsh binaries with `EXEC_WRAPPER` support for command interception.

## Build & Test

```bash
pnpm install          # install dependencies
pnpm run build        # compile to bin/mcp-server.js (CJS, node18)
pnpm test             # run jest tests
```

## Architecture

The server detects the host platform via `process.platform`/`process.arch`, maps it to a Rust target triple, and selects the correct patched Bash binary from bundled vendor directories. On Linux, it parses `/etc/os-release` to match against known distro variants (Ubuntu, Debian, CentOS/RHEL/Rocky/AlmaLinux with version-aware fallback). On macOS, it matches the Darwin kernel major version. The patched shells intercept `execve(2)` calls, allowing the MCP server to allow, prompt (human approval via MCP elicitation), or block each command based on `.rules` files.

## Key Considerations

- No runtime npm dependencies -- devDependencies only.
- Output is CJS (not ESM) targeting Node 18+. Build config in `tsup.config.ts`.
- The `patches/` directory contains the Bash and Zsh source patches that add `EXEC_WRAPPER` env var support.
- Related Rust crates: `codex-rs/shell-escalation/` (the escalation binary) and `codex-rs/shell-command/` (command parsing/safety analysis).
- Supports Linux (Ubuntu 24.04/22.04/20.04, Debian 12/11, CentOS-like 9) and macOS (15/14/13).
