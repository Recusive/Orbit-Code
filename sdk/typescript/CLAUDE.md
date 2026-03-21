# sdk/typescript/

TypeScript SDK (`@orbit.build/orbit-code-sdk`) that wraps the `codex` CLI by spawning `codex exec --experimental-json` and reading JSONL events from stdout.

## Build & Test

```bash
pnpm install          # install dependencies
pnpm run build        # build with tsup (ESM output to dist/)
pnpm run test         # run jest tests
pnpm run lint         # eslint
pnpm run format       # prettier check
```

## Architecture

`new Codex(options)` creates a `CodexExec` instance that resolves the platform binary. `codex.startThread()` returns a `Thread` that can `run(input)` (buffered) or `runStreamed(input)` (async generator). Under the hood, `CodexExec` spawns the CLI process, pipes input via stdin, and yields parsed JSONL events from stdout. Binary resolution walks `@orbit.build/orbit-code-{platform}-{arch}` optional dependencies to find the native executable.

## Key Considerations

- ESM-only package (`"type": "module"`). Target ES2022, Node 18+.
- Event types in `events.ts` must mirror `codex-rs/exec/src/exec_events.rs` -- keep them in sync.
- `@modelcontextprotocol/sdk` is a dependency for MCP content block types in `items.ts`.
- Uses `tsup` with dts generation. Config in `tsup.config.ts`.
