# codex-rs/hooks/src/

Source for the `orbit-code-hooks` crate.

`lib.rs` declares modules and re-exports public types. `registry.rs` defines `Hooks` (public API) and `HooksConfig`. `types.rs` has core types (`Hook`, `HookPayload`, `HookEvent`, `HookResult`, `HookResponse`). `schema.rs` defines JSON Schema wire types and `write_schema_fixtures()`. `legacy_notify.rs` handles backward-compatible fire-and-forget hooks. `engine/` contains the discovery, command execution, and output parsing internals. `events/` has per-event request/outcome types and run/preview logic for each of the three event types.
