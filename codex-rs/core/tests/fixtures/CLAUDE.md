# codex-rs/core/tests/fixtures/

Test fixture data files for integration tests.

## What this folder does

Contains static data files used by integration tests as inputs or expected outputs.

## Key files

| File | Purpose |
|------|---------|
| `incomplete_sse.json` | JSON fixture containing an incomplete SSE (Server-Sent Events) response, used to test error handling for truncated or malformed API responses |

## Where it plugs into

- Referenced by integration tests in `tests/suite/` via `include_str!()` or file path
- Used to verify resilience to malformed API responses
