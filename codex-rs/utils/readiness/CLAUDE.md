# codex-rs/utils/readiness/

Async readiness flag with subscription-based authorization. Components subscribe to receive a token, any token holder can mark the flag as ready, and other components can asynchronously wait for readiness.

## Build & Test
```bash
cargo build -p orbit-code-utils-readiness
cargo test -p orbit-code-utils-readiness
```

## Key Considerations
- The flag becomes ready automatically if no subscribers exist.
- Lock acquisition has a 1-second timeout to prevent deadlocks; Token 0 is reserved and never authorized.
