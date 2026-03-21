# codex-rs/utils/absolute-path/src/

Newtype wrapper (`AbsolutePathBuf`) guaranteeing paths are absolute and normalized, with tilde expansion, serde deserialization via a thread-local base path guard, and `JsonSchema`/`TS` support.

## Build & Test
```bash
cargo build -p orbit-code-utils-absolute-path
cargo test -p orbit-code-utils-absolute-path
```

## Key Considerations
- `AbsolutePathBufGuard` sets a thread-local base path used during `Deserialize` of relative paths -- must be set before deserializing config types that contain `AbsolutePathBuf`.
- Tilde expansion only works on non-Windows platforms.
