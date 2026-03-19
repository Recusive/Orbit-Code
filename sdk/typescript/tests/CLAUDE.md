# sdk/typescript/tests/

Jest test suite for the `@openai/codex-sdk` TypeScript package.

## Purpose

Tests the SDK's core functionality: thread runs, streaming, abort handling, CLI exec arg building, and environment variable passing. Uses a mock HTTP server to simulate the Codex API responses.

## Key Files

| File | Role |
|------|------|
| `run.test.ts` | Tests for `thread.run()` -- verifies completed turns, item collection, final response extraction, error handling, config override serialization |
| `runStreamed.test.ts` | Tests for `thread.runStreamed()` -- verifies async generator event streaming |
| `abort.test.ts` | Tests for turn cancellation via `AbortSignal` |
| `exec.test.ts` | Tests for `CodexExec` -- CLI argument building, config override flattening/serialization |
| `testCodex.ts` | Test helper: `createTestClient()` and `createMockClient()` that configure a `Codex` instance pointing at the debug CLI binary with mock provider config |
| `codexExecSpy.ts` | Jest spy wrapper for `child_process.spawn` to capture CLI args and env vars passed to the Codex binary |
| `responsesProxy.ts` | Mock HTTP server that simulates OpenAI Responses API SSE streams for testing |
| `setupCodexHome.ts` | Jest setup file: creates a temporary `CODEX_HOME` directory per test to isolate session state |

## Imports From

- `../src/` -- the SDK source modules under test
- `@jest/globals` for test lifecycle hooks
- `node:child_process`, `node:http`, `node:fs/promises` for test infrastructure

## Running

```bash
cd sdk/typescript
pnpm run test       # run all tests
pnpm run test:watch # watch mode
pnpm run coverage   # with coverage report
```

## Test Configuration

Configured via `jest.config.cjs` at the package root:
- Uses `ts-jest` with ESM preset
- Transforms `import.meta.url` via `ts-jest-mock-import-meta`
- Setup file `setupCodexHome.ts` runs before each test
