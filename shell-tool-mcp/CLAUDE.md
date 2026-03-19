# shell-tool-mcp/

## Purpose

An MCP (Model Context Protocol) server package (`@openai/codex-shell-tool-mcp`) that provides a `shell` tool for running commands inside a sandboxed instance of Bash. The patched Bash intercepts `execve(2)` calls, allowing the MCP server to decide whether to allow, escalate, or block each command based on user-defined `.rules` files.

## What It Does

- Ships patched Bash and Zsh binaries (with `EXEC_WRAPPER` support) for multiple platforms
- The TypeScript launcher (`src/index.ts`) detects the host platform/OS and selects the appropriate patched shell binary
- Supports Linux (Ubuntu 24.04/22.04/20.04, Debian 12/11, CentOS-like 9) and macOS (15/14/13)
- Declares the `codex/sandbox-state` MCP capability, allowing Codex to update sandbox policy at runtime
- Command decisions: `allow` (escalated), `prompt` (MCP elicitation for human approval), `forbidden` (exit 1)

## Key Files

| File | Role |
|------|------|
| `src/index.ts` | Entry point. Resolves platform, selects bash binary, and prints the path. |
| `src/bashSelection.ts` | Logic for selecting the correct Bash binary variant based on OS release info |
| `src/constants.ts` | Defines Linux and Darwin Bash variant configurations |
| `src/osRelease.ts` | Parses `/etc/os-release` to determine Linux distribution and version |
| `src/platform.ts` | Maps `process.platform`/`process.arch` to Rust target triples |
| `src/types.ts` | TypeScript type definitions for Bash variants, OS info, and selection results |
| `tsup.config.ts` | Build config: compiles `src/index.ts` to `bin/mcp-server.js` (CJS, node18 target) |
| `jest.config.cjs` | Jest test configuration |
| `patches/bash-exec-wrapper.patch` | Patch for Bash's `execute_cmd.c` to add `EXEC_WRAPPER` env var support |
| `patches/zsh-exec-wrapper.patch` | Patch for Zsh's `exec.c` to add `EXEC_WRAPPER` env var support |
| `tests/` | Jest tests for bash selection and OS release parsing |

## Build Commands

```bash
pnpm install
pnpm run build          # Outputs to bin/mcp-server.js
pnpm test               # Run Jest tests
```

## Imports From

- Node.js built-ins: `os`, `path`, `fs`
- No runtime npm dependencies (devDependencies only)

## Exports To

- Published as `@openai/codex-shell-tool-mcp` on npm
- Used by Codex CLI as an MCP server: `npx -y @openai/codex-shell-tool-mcp`
- Part of the pnpm workspace defined in the root `pnpm-workspace.yaml`

## Relationship to Other Directories

- `codex-rs/shell-escalation/`: The Rust-side implementation of the shell escalation binary that acts as the `EXEC_WRAPPER`
- `codex-rs/shell-command/`: Shell command parsing and safety analysis
- The patches in `patches/` modify Bash and Zsh source code to support the `EXEC_WRAPPER` environment variable
