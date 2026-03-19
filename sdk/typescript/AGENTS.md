# sdk/typescript/

This file applies to `sdk/typescript/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `@openai/codex-sdk` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm build`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm test`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm lint`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

TypeScript SDK for the Codex agent. Published as `@openai/codex-sdk`.

### Purpose

Wraps the `codex` CLI by spawning `codex exec --experimental-json` as a child process. Reads JSONL events from stdout and exposes typed `Codex`, `Thread`, and streaming event abstractions for Node.js consumers.

### Key Files

| File | Role |
|------|------|
| `src/index.ts` | Package entry point; re-exports all public types and classes |
| `src/codex.ts` | `Codex` class -- main entry point; creates `Thread` instances via `startThread()` and `resumeThread()` |
| `src/codexOptions.ts` | `CodexOptions` type -- configuration for binary path, API key, base URL, env, and `--config` overrides |
| `src/thread.ts` | `Thread` class -- represents a conversation; provides `run()` (buffered) and `runStreamed()` (async generator) methods |
| `src/threadOptions.ts` | `ThreadOptions` type -- model, sandbox mode, working directory, reasoning effort, web search, approval policy |
| `src/turnOptions.ts` | `TurnOptions` type -- output JSON schema and AbortSignal for cancellation |
| `src/exec.ts` | `CodexExec` class -- spawns the CLI process, builds command args, yields JSONL lines; handles binary resolution via `@openai/codex` platform packages |
| `src/events.ts` | Event type definitions (`ThreadStartedEvent`, `TurnCompletedEvent`, `ItemCompletedEvent`, etc.) mirroring `codex-rs/exec/src/exec_events.rs` |
| `src/items.ts` | Thread item types (`AgentMessageItem`, `CommandExecutionItem`, `FileChangeItem`, `McpToolCallItem`, etc.) |
| `src/outputSchemaFile.ts` | Helper to write a JSON schema to a temp file for the `--output-schema` CLI flag |
| `package.json` | Package manifest; `@openai/codex-sdk` version `0.0.0-dev`; ESM-only |
| `tsconfig.json` | TypeScript config targeting ES2022, strict mode |
| `tsup.config.ts` | Build config: single ESM entry, dts generation, node18 target |

### Architecture

1. `new Codex(options)` creates a `CodexExec` instance that resolves the CLI binary path
2. `codex.startThread()` returns a `Thread` bound to the exec instance
3. `thread.run(input)` calls `thread.runStreamed(input)` internally, collecting events into a `Turn` result
4. `thread.runStreamed(input)` spawns the CLI process via `CodexExec.run()`, piping input via stdin and reading JSONL from stdout
5. Events are yielded as parsed `ThreadEvent` objects

### Binary Resolution

`CodexExec.findCodexPath()` resolves the platform binary by:
1. Determining the target triple from `process.platform` and `process.arch`
2. Looking up the corresponding `@openai/codex-{platform}-{arch}` optional dependency
3. Resolving `vendor/{triple}/codex/codex` within that package

### Imports From

- `@modelcontextprotocol/sdk` (for MCP content block types in `items.ts`)
- Node.js built-ins (`child_process`, `readline`, `fs`, `path`, `os`)
- `@openai/codex` platform packages (optional deps for binary resolution)

### Exports To

- Consumers install `@openai/codex-sdk` and import `Codex`, `Thread`, event types, item types

### Build / Dev Commands

```bash
pnpm install              # install dependencies
pnpm run build            # build with tsup (outputs to dist/)
pnpm run test             # run jest tests
pnpm run lint             # eslint
pnpm run format           # prettier check
```

### Subdirectories

- `src/` -- TypeScript source
- `tests/` -- Jest test suite
- `samples/` -- Runnable example scripts
