# sdk/typescript/src/

TypeScript source for the `@orbit.build/orbit-code-sdk` package. Built with `tsup` into `dist/` as ESM with declaration files.

## Module Layout

- **Entry/API surface**: `index.ts` (re-exports), `codex.ts` (Codex class), `thread.ts` (Thread class with `run`/`runStreamed`)
- **Options types**: `codexOptions.ts`, `threadOptions.ts`, `turnOptions.ts`
- **CLI integration**: `exec.ts` (process spawning, binary resolution, arg building)
- **Wire types**: `events.ts` (JSONL event definitions), `items.ts` (thread item union types)
- **Utilities**: `outputSchemaFile.ts` (temp file for `--output-schema` flag)
