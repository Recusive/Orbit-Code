# sdk/typescript/samples/

This file applies to `sdk/typescript/samples/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- This subtree belongs to the `@openai/codex-sdk` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm build`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm test`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm lint`
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/typescript && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Runnable TypeScript sample scripts demonstrating the SDK.

### Purpose

Provides example applications showing how to use `@openai/codex-sdk` for common use cases: interactive streaming chat, structured output, and structured output with Zod schemas.

### Key Files

| File | Role |
|------|------|
| `helpers.ts` | Shared helper: `codexPathOverride()` resolves the CLI binary from `CODEX_EXECUTABLE` env var or the debug build at `codex-rs/target/debug/codex` |
| `basic_streaming.ts` | Interactive chat loop with streaming events -- demonstrates `runStreamed()`, event handling for all item types (agent messages, commands, file changes, todo lists), and token usage reporting |
| `structured_output.ts` | Demonstrates structured output using a plain JSON schema object with `run()` |
| `structured_output_zod.ts` | Demonstrates structured output using a Zod schema converted via `zod-to-json-schema` |

### Imports From

- `@openai/codex-sdk` (the SDK package)
- `helpers.ts` for binary path resolution
- `zod` and `zod-to-json-schema` (in the Zod sample)

### Running

Samples use `ts-node-esm` via the shebang line:

```bash
cd sdk/typescript
./samples/basic_streaming.ts
./samples/structured_output.ts
./samples/structured_output_zod.ts
```

Or with explicit ts-node:

```bash
pnpm ts-node-esm --files samples/basic_streaming.ts
```
