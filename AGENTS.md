# Orbit Code — Coding Conventions

> **These rules are mandatory.** All code in this repo must follow these conventions exactly. They are derived from the original OpenAI team's patterns and enforced by tooling (clippy, rustfmt, prettier, eslint). Full evidence and file references: `docs/pattern/CODING_CONVENTIONS.md`.

## Repository Structure

| Directory | Purpose |
|-----------|---------|
| `codex-rs/` | **Primary codebase** — Rust implementation (67+ crates) |
| `codex-cli/` | npm wrapper — thin JS launcher resolving platform-specific Rust binaries |
| `sdk/` | Client SDKs (Python + TypeScript) for programmatic access |
| `shell-tool-mcp/` | MCP server exposing shell tool capabilities |
| `scripts/` | Repo-wide utility scripts (release, install) |
| `docs/` | Documentation (contributing, install, config) |
| `tools/` | Developer tooling (argument-comment linting via Dylint) |

## Common Commands

```bash
just codex                   # Run Orbit Code from source
just test                    # Run Rust tests (nextest)
just fmt                     # Format Rust code
just fix                     # Run clippy fixes
just fix -p <crate>          # Clippy fix scoped to one crate
just write-config-schema     # Regenerate config JSON schema
just write-app-server-schema # Regenerate app-server protocol schemas
just bazel-lock-update       # Update MODULE.bazel.lock after dep changes
just bazel-lock-check        # Verify lockfile is in sync
just argument-comment-lint   # Run /*param*/ comment lint
```

**Build systems:** Cargo (local dev) + Bazel (CI/release). TypeScript: pnpm + tsup. Task runner: `just` (working dir = `codex-rs/`).

---

## Critical Warnings

- **Never** add or modify code related to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` or `CODEX_SANDBOX_ENV_VAR`. These are set by the sandbox runtime and existing code was authored with this in mind. `CODEX_SANDBOX_NETWORK_DISABLED=1` is set whenever the `shell` tool runs in sandbox. `CODEX_SANDBOX=seatbelt` is set on child processes spawned under macOS Seatbelt (`/usr/bin/sandbox-exec`). Existing checks for these env vars are intentional early-exit guards — do not remove or modify them.
- **Never** mutate process environment in tests. Prefer passing environment-derived flags or dependencies from above.
- If you add `include_str!`, `include_bytes!`, or `sqlx::migrate!`, update the crate's `BUILD.bazel` (`compile_data`, `build_script_data`, or test data) or Bazel will fail even when Cargo passes.
- Install any commands the repo relies on (e.g., `just`, `rg`, `cargo-insta`, `cargo-nextest`) if they aren't already available before running instructions.

---

## Rust — Module Organization

1. Declare modules private by default with `mod foo;`. Re-export only needed types via `pub use foo::Type;` in lib.rs.
2. Use `pub mod` only for major subsystem modules that form the crate's public API surface. Protocol crates (`protocol`, `app-server-protocol`) export everything publicly.
3. Use a single `.rs` file for focused modules. Use a subdirectory with `mod.rs` when a module has 3+ sub-modules.
4. Place integration tests in `tests/all.rs` -> `tests/suite/mod.rs` -> `tests/suite/*.rs`. Use `tests/common/` for shared test utilities as a library crate. Never create multiple top-level test binaries.
5. Target modules under 500 LoC excluding tests. At ~800 LoC, add new functionality in a new module instead of extending the existing file. Do not create single-use helper methods. This applies especially to high-touch files: `tui/src/app.rs`, `tui/src/bottom_pane/chat_composer.rs`, `tui/src/bottom_pane/footer.rs`, `tui/src/chatwidget.rs`, `tui/src/bottom_pane/mod.rs`, and similarly central orchestration modules.
6. When extracting code from a large module, move the related tests and module/type docs toward the new implementation so invariants stay close to the code that owns them.

## Rust — Error Handling

7. Define error types with `#[derive(Debug, Error)]` from thiserror. Every variant must have an `#[error(...)]` attribute. Use `#[error(transparent)]` for wrapped errors.
8. Define `pub type Result<T> = std::result::Result<T, YourError>;` alongside each crate-level error type.
9. Prefer `#[from]` for direct error wrapping. Use manual `From` impls only when conversion requires custom logic.
10. Add domain-specific query methods to error types (e.g., `is_retryable()`, `to_protocol_error()`). Use exhaustive matches, not wildcards.
11. Never use `unwrap()` or `expect()` in library code. Use `?` or explicit error handling. Tests may use `expect("descriptive message")`.

## Rust — Async Patterns

12. Tokio is the sole async runtime. `#[tokio::test]` for async tests, `tokio::spawn` for background tasks, `Handle::current()` only when you need async work in Drop impls.
13. Channel selection: `broadcast` for fan-out notifications, `oneshot` for single request/response, `async_channel` for MPMC.
14. Shared state: `tokio::sync::Mutex` for async locking, `tokio::sync::RwLock` for read-heavy state. Always wrap in `Arc<>`.
15. Use `CancellationToken` with `.child_token()` for hierarchical, coordinated shutdown of subsystems.
16. Use `JoinSet` for parallel Tokio tasks. Use `FuturesUnordered` for stream-based concurrent futures.
17. Retries: exponential backoff (`2^(attempt-1)`) with +/-10% jitter via `random_range(0.9..1.1)`. Use saturating arithmetic. Only retry transient errors (429, 5xx, connection failures).

## Rust — Serialization (serde)

18. Derive sets by type category:
    - Config: `Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema`
    - Protocol: add `TS`
    - App-server v2: add `TS` + `ExperimentalApi` where needed
19. `rename_all` by context: `kebab-case` for config TOML, `snake_case` for protocol, `camelCase` for app-server v2. Exception: config RPC payloads use `snake_case` to mirror TOML keys.
20. NEVER use `skip_serializing_if` on v2 `Option<T>` payload fields — use `#[ts(optional = nullable)]` instead. For v2 booleans defaulting to false: `#[serde(default, skip_serializing_if = "std::ops::Not::not")] pub field: bool`.
21. Always add `#[ts(export_to = "v2/")]` on v2 types. Keep `#[serde(rename)]` and `#[ts(rename)]` aligned. For tagged unions: `#[serde(tag = "type")]` and `#[ts(tag = "type")]`.

## Rust — Traits & Visibility

22. Add `Send + Sync` bounds to traits used behind `Arc<dyn Trait>` or across thread boundaries. Add `+ 'static` when the trait object must outlive its scope.
23. Use `#[async_trait]` for any trait with async methods. Place it directly above the trait definition.
24. Provide default implementations for optional trait methods. Use no-op defaults for lifecycle hooks (abort, cleanup). Document the default behavior.
25. Use `pub(crate)` for crate-internal types, `pub(super)` for parent-module-only types. Private by default; selective re-exports via `pub use`.

## Rust — Imports & Naming

26. One import per `use` statement (enforced by `imports_granularity = "Item"` in rustfmt.toml). Run `just fmt` to enforce ordering automatically.
27. Package names: `orbit-code-<name>` (hyphens). Library names: `orbit_code_<name>` (underscores). Binary names: `orbit-code` or `orbit-code-<tool>` (hyphens).
28. App-server types: `*Params` for requests, `*Response` for responses, `*Notification` for notifications. RPC methods: `<singular-resource>/<camelCaseMethod>` (e.g., `thread/start`, `fs/readFile`).

## Rust — API Design

29. Avoid bool or ambiguous `Option` parameters that force callers to write `foo(false)` or `bar(None)`. Prefer enums, named methods, or newtypes to keep callsites self-documenting.
30. When you cannot change the API and have opaque literal arguments (booleans, `None`, numbers), use `/*param_name*/` comments before them. The name must exactly match the callee's parameter. String/char literals are exempt unless the comment adds real clarity.
31. Use `#[derive(Debug, Parser)]` with `#[clap(flatten)]` for shared option groups and `#[clap(visible_alias)]` for subcommand aliases.
32. Prefer plain `String` IDs at API boundaries. Do UUID parsing/conversion internally. Timestamps: integer Unix seconds (`i64`) named `*_at`.
33. When making exhaustive `match` statements, avoid wildcard arms. Prefer listing all variants so the compiler catches new additions.

## Rust — Documentation

34. Add `//!` module docs to every module describing its purpose and key types.
35. Document all public items with `///`. Use [`TypeName`] syntax for cross-references. Skip docs for trivial getters and self-evident enum variants.
36. Use `//` inline comments only for non-obvious logic, invariants, or "why" explanations. Never restate what the code does.
37. When a change adds or modifies an API, update documentation in the `docs/` folder if applicable. At minimum update `app-server/README.md` for app-server changes.

## Rust — Testing

38. Use `pretty_assertions::assert_eq!` in every test module. Compare entire objects with `assert_eq!`, not individual fields.
39. Add `insta` snapshot tests for any UI-affecting change. Snapshot workflow:
    - Run tests: `cargo test -p codex-tui`
    - Check pending: `cargo insta pending-snapshots -p codex-tui`
    - Preview: `cargo insta show -p codex-tui path/to/file.snap.new`
    - Accept: `cargo insta accept -p codex-tui`
    - Install if missing: `cargo install cargo-insta`
40. Use `wiremock::MockServer` and helpers from `core_test_support::responses` for HTTP mocking. Prefer `mount_sse_once` over `mount_sse_once_match` or `mount_sse_sequence`. All `mount_sse*` helpers return a `ResponseMock` — hold onto it to assert against outbound requests. Use `ResponseMock::single_request()` for single-POST tests, `ResponseMock::requests()` to inspect all captured requests. `ResponsesRequest` exposes: `body_json`, `input`, `function_call_output`, `custom_tool_call_output`, `call_output`, `header`, `path`, `query_param`. Build SSE payloads with `ev_*` constructors and `sse(...)`. Typical pattern:
    ```rust
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-1"),
        responses::ev_function_call(call_id, "shell", &serde_json::to_string(&args)?),
        responses::ev_completed("resp-1"),
    ])).await;
    codex.submit(Op::UserTurn { ... }).await?;
    let request = mock.single_request();
    // assert using request.function_call_output(call_id) or request.body_json()
    ```
41. Use `TestCodexBuilder` fluent API for integration test setup. Chain `.with_config()`, `.with_model()`, `.with_auth()`, `.with_home()`, `.with_pre_build_hook()`.
42. Use `wait_for_event(codex, predicate)` for async event assertions. Prefer it over `wait_for_event_with_timeout`.
43. Use `codex_utils_cargo_bin::cargo_bin()` for binary resolution (works with Cargo and Bazel). Use `find_resource!` instead of `env!("CARGO_MANIFEST_DIR")`.
44. Use `#[ctor]` in `tests/common/lib.rs` for process-startup initialization (deterministic IDs, insta workspace root).
45. Avoid boilerplate tests that only assert experimental field markers for individual request fields in `common.rs` — rely on schema generation/tests and behavioral coverage instead.

## Rust — Clippy & Lints

46. 33 clippy lints are denied workspace-wide. Key rules enforced:
    - `unwrap_used`, `expect_used` — no panics in library code (allowed in tests)
    - `uninlined_format_args` — always `format!("{var}")` not `format!("{}", var)`
    - `redundant_closure_for_method_calls` — use method references over closures
    - `collapsible_if` — always collapse nested ifs
    - All `manual_*` rules — use idiomatic Rust
    - All `needless_*` rules — no unnecessary borrows, collects, late inits
47. Disallowed ratatui methods (enforced in clippy.toml): `Color::Rgb`, `Color::Indexed`, `.white()`, `.black()`, `.yellow()`. Use ANSI colors only.
48. Large-error-threshold: 256 bytes. Box large payloads if an error variant exceeds this.
49. Crates with TUI output must add `#![deny(clippy::print_stdout, clippy::print_stderr)]` at the top of lib.rs.

## Rust — TUI (ratatui)

50. Use Stylize trait helpers: `"text".red()`, `"text".dim()`, `"text".bold()`, `"text".cyan()`. Chain: `url.cyan().underlined()`. Use `Span::styled` only for runtime-computed styles.
51. Basic spans: `"text".into()`. Lines: `vec![span1, span2].into()`. Use `Line::from(vec![...])` only when the target type isn't obvious. Prefer the form that stays on one line after rustfmt.
52. Color palette from `tui/styles.md`: Headers = bold. Secondary = dim. Selection/tips = cyan. Success = green. Errors = red. Branding = magenta. Never use blue, yellow, white, black, Rgb, or Indexed.
53. Text wrapping: `adaptive_wrap_lines()` for content with URLs, `word_wrap_lines()` for plain text, `textwrap::wrap()` for raw strings. Use `prefix_lines` from `line_utils` for indented multi-line output. For indented wrapped lines, use `initial_indent` / `subsequent_indent` options from `RtOptions` rather than writing custom logic.
54. Mirror all `tui/` changes in `tui_app_server/` unless there is a documented reason not to.
55. Don't refactor between equivalent forms (`Span::styled` <-> `set_style`, `Line::from` <-> `.into()`) without a clear readability or functional gain. Follow file-local conventions. Do not introduce type annotations solely to satisfy `.into()`.

## Rust — Config & Dependencies

56. Config types must derive `JsonSchema`. Run `just write-config-schema` after any `ConfigToml` change.
57. Run `just write-app-server-schema` (and `--experimental` when needed) after API shape changes. Validate with `cargo test -p codex-app-server-protocol`.
58. Add all dependencies to `[workspace.dependencies]` in root `Cargo.toml`. Per-crate: `{ workspace = true }` with crate-specific feature overrides only.
59. After any dependency change: run `just bazel-lock-update` then `just bazel-lock-check`. Include the lockfile update in the same change.
60. Standard dev-dependencies: `pretty_assertions` (diffs), `tempfile` (temp dirs), `wiremock` (HTTP mocking), `insta` (snapshots).

## TypeScript

61. ESM-first: `"type": "module"` in package.json. Use `import`/`export` syntax exclusively.
62. Always use `node:` prefix for built-in imports: `import from "node:fs"`, `import from "node:path"`.
63. Target ES2022 with ESNext modules. Enable `strict: true` and `noUncheckedIndexedAccess: true`.
64. `export type` for type-only re-exports from index.ts. `export class` for concrete implementations.
65. Prefix unused parameters with `_`.
66. tsup for bundling (ESM output for SDK, CJS for MCP servers). Jest with ts-jest for testing.
67. Prettier with project config (`trailingComma: "all"`). ESLint flat config with typescript-eslint.

## Python

68. Hatchling build system. Pydantic v2 models. `src/` layout.
69. pytest with `-q`. ruff for linting. Auto-generate models from JSON schema.

## Build & Workflow

70. Run `just fmt` after every Rust change. Do not ask for approval — just run it.
71. Run `just fix -p <crate>` before finalizing large changes. Scope with `-p` to avoid slow workspace-wide builds. Only run `just fix` without `-p` if you changed shared crates.
72. Test the changed crate first: `cargo test -p <crate>`. Project-specific or individual tests can be run without asking the user. Ask the user before running the complete test suite (`just test`). Run full suite only if core, common, or protocol crates changed. Avoid `--all-features` for routine runs — it expands the build matrix and increases `target/` disk usage.
73. Do not re-run tests after running `just fix` or `just fmt`.
74. All active API development happens in app-server v2. Do not add new API surface area to v1.
75. For experimental API surface: use `#[experimental("method/or/field")]`, derive `ExperimentalApi` when field-level gating is needed, and use `inspect_params: true` in `common.rs` when only some fields of a method are experimental.

## App-Server V2 Quick Reference

- Payloads: `*Params` (request), `*Response` (response), `*Notification` (notification)
- Wire format: `camelCase` via `#[serde(rename_all = "camelCase")]`
- Optional fields in `*Params`: `#[ts(optional = nullable)]`. Do not use `#[ts(optional = nullable)]` outside `*Params`.
- Optional collections in `*Params`: use `Option<Vec<...>>` + `#[ts(optional = nullable)]`. Never use `#[serde(default)]` for optional collections, and do not use `skip_serializing_if` on v2 payload fields.
- Boolean defaults-to-false: `#[serde(default, skip_serializing_if = "std::ops::Not::not")] pub field: bool`
- No-params exception: client->server requests with no params may use `params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>`
- Cursor pagination for list methods: request `cursor: Option<String>` + `limit: Option<u32>`, response `data: Vec<...>` + `next_cursor: Option<String>`
- TypeScript export: `#[ts(export_to = "v2/")]` on all v2 types
- Validate with `cargo test -p codex-app-server-protocol`
- Key files: `app-server-protocol/src/protocol/common.rs`, `app-server-protocol/src/protocol/v2.rs`, `app-server/README.md`
