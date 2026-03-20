# Orbit Code — Comprehensive Coding Conventions

> **Generated:** 2026-03-19 via full repository audit
> **Scope:** codex-rs/ (Rust), codex-cli/ (TypeScript), sdk/ (TypeScript + Python), shell-tool-mcp/ (TypeScript)

---

## 1. Rust Module Organization

### 1a. Private Modules with Selective Re-exports (Dominant Pattern)

**Convention**: Modules are declared `mod foo;` (private) by default, with only the types needed by consumers re-exported via `pub use foo::Type;` in `lib.rs`.

**Evidence**:
- `codex-rs/core/src/lib.rs:17,20` — `mod codex;` (private), then `pub use codex::SteerInputError;` (selective re-export)
- `codex-rs/secrets/src/lib.rs:15-19` — `mod local;` / `mod sanitizer;` (private), then `pub use local::LocalSecretsBackend;` / `pub use sanitizer::redact_secrets;`
- `codex-rs/config/src/lib.rs:1-58` — All modules private (`mod cloud_requirements;`, `mod config_requirements;`, etc.), all re-exported selectively via `pub use` statements

**Rule**: Declare modules private by default with `mod foo;`. Re-export only the types consumers need via `pub use foo::Type;` in `lib.rs`.

### 1b. `pub mod` for Major Subsystems

**Convention**: `pub mod` is used for large, self-contained feature areas intended for external consumption — protocol definitions, config, tools, auth, etc.

**Evidence**:
- `codex-rs/core/src/lib.rs:9,13,28-30,64,82,100` — `pub mod api_bridge;`, `pub mod auth;`, `pub mod config;`, `pub mod exec;`, `pub mod mcp;`, `pub mod shell;`, `pub mod skills;` (major subsystems)
- `codex-rs/protocol/src/lib.rs:1-21` — Nearly all modules are `pub mod` (public API crate)
- `codex-rs/app-server-protocol/src/lib.rs:1-5` — Private modules with glob `pub use protocol::v2::*;` re-exports (everything public)

**Rule**: Use `pub mod` for major subsystem modules that form the crate's public API surface. Use private `mod` + `pub use` for internal implementation modules.

### 1c. File-as-Module vs. Subdirectory Pattern

**Convention**: Single `.rs` files for focused modules; subdirectory with `mod.rs` for complex feature areas with multiple sub-modules.

**Evidence**:
- `codex-rs/core/src/auth.rs` (1,462 lines) — File-as-module for focused auth implementation
- `codex-rs/core/src/tools/mod.rs` — Subdirectory pattern for complex tool system (`tools/handlers/`, `tools/runtimes/`, `tools/js_repl/`)
- `codex-rs/app-server/src/lib.rs:57-75` — Flat file-as-module pattern (all `.rs` files, no subdirectories)

**Rule**: Use a single `.rs` file for focused modules. Use a subdirectory with `mod.rs` when a module has 3+ sub-modules or distinct sub-areas.

### 1d. The `all.rs` Test Aggregator Pattern

**Convention**: Integration tests use a single `tests/all.rs` entry point that declares `mod suite;`, aggregating all test modules into one binary for faster linking.

**Evidence**:
- `codex-rs/core/tests/all.rs:1-3` — `mod suite;` (single integration test binary)
- `codex-rs/tui/tests/all.rs:1-9` — Same pattern with conditional `#[cfg(feature = "vt100-tests")]`
- `codex-rs/exec/tests/all.rs`, `codex-rs/mcp-server/tests/all.rs`, `codex-rs/app-server/tests/all.rs`, `codex-rs/otel/tests/all.rs` — All follow the same aggregator pattern

**Rule**: Place integration tests in `tests/all.rs` → `tests/suite/mod.rs` → `tests/suite/*.rs`. Use `tests/common/` for shared test utilities. Never create multiple top-level test binaries.

### 1e. Module Size Limits

**Convention**: Target modules under 500 LoC (excluding tests). Split at ~800 LoC. Test files may exceed these limits.

**Evidence**:
- `codex-rs/core/src/flags.rs` (6 lines), `codex-rs/core/src/mention_syntax.rs` (4 lines) — Small, focused modules
- `codex-rs/core/src/codex.rs` (7,365 lines) — Acknowledged large file; AGENTS.md explicitly calls out high-touch files
- `codex-rs/core/src/config/config_tests.rs` (6,238 lines) — Test files permitted to be large

**Rule**: Target modules under 500 LoC excluding tests. At ~800 LoC, add new functionality in a new module. Test files are exempt from this limit. Do not create single-use helper methods.

---

## 2. Rust Error Handling

### 2a. thiserror Derives

**Convention**: All error types use `#[derive(Error, Debug)]` (or `#[derive(Debug, Error)]`) with `#[error(...)]` display attributes from thiserror.

**Evidence**:
- `codex-rs/core/src/error.rs:63-186` — `CodexErr` enum with 30+ variants, each with `#[error(...)]` messages
- `codex-rs/codex-api/src/error.rs:7-32` — `ApiError` with `#[error(transparent)]` and `#[error("api error {status}: {message}")]`
- `codex-rs/utils/git/src/errors.rs:10-35` — `GitToolingError` with structured error messages

**Rule**: Define error types with `#[derive(Debug, Error)]` from thiserror. Every variant must have an `#[error(...)]` attribute. Use `#[error(transparent)]` for wrapped errors.

### 2b. Per-Crate Result Type Aliases

**Convention**: Each crate with its own error type defines a convenience `pub type Result<T> = std::result::Result<T, CrateError>;`.

**Evidence**:
- `codex-rs/core/src/error.rs:23` — `pub type Result<T> = std::result::Result<T, CodexErr>;`
- `codex-rs/execpolicy/src/error.rs:4` — `pub type Result<T> = std::result::Result<T, Error>;`
- `codex-rs/otel/src/metrics/error.rs:3` — `pub type Result<T> = std::result::Result<T, MetricsError>;`

**Rule**: Define `pub type Result<T> = std::result::Result<T, YourError>;` alongside each crate-level error type.

### 2c. `#[from]` for Automatic Conversions

**Convention**: Use `#[from]` for straightforward error wrapping. Use manual `From` impls only when conversion requires custom logic (e.g., mapping to a different variant).

**Evidence**:
- `codex-rs/core/src/error.rs:150,168,171,175` — Multiple `#[from]` attributes: `Io(#[from] io::Error)`, `Json(#[from] serde_json::Error)`, etc.
- `codex-rs/core/src/error.rs:188-192` — Manual `From<CancelErr>` mapping to `CodexErr::TurnAborted` (custom logic)
- `codex-rs/codex-api/src/error.rs:34-38` — Manual `From<RateLimitError>` converting to `Self::RateLimit(err.to_string())`

**Rule**: Prefer `#[from]` for direct error wrapping. Use manual `From` impls only when the conversion requires logic beyond simple wrapping.

### 2d. Domain-Specific Error Methods

**Convention**: Error types include domain methods like `is_retryable()`, `to_protocol_error()`, `http_status_code_value()` for rich error inspection.

**Evidence**:
- `codex-rs/core/src/error.rs:195-230` — `is_retryable()` with exhaustive match on all variants
- `codex-rs/core/src/error.rs:583-609` — `to_orbit_code_protocol_error()` mapping to protocol error codes
- `codex-rs/core/src/error.rs:623-632` — `http_status_code_value()` extracting HTTP status from error variants

**Rule**: Add domain-specific query methods to error types (e.g., `is_retryable()`, conversion to protocol errors). Use exhaustive matches, not wildcards.

### 2e. No `unwrap`/`expect` in Library Code

**Convention**: `unwrap_used` and `expect_used` are denied workspace-wide in clippy. Tests are exempt via `clippy.toml` settings.

**Evidence**:
- `codex-rs/Cargo.toml:316,348` — `expect_used = "deny"` and `unwrap_used = "deny"` in `[workspace.lints.clippy]`
- `codex-rs/clippy.toml` — `allow-expect-in-tests = true`, `allow-unwrap-in-tests = true`
- `codex-rs/core/src/error.rs` — Zero `unwrap`/`expect` calls in the entire file

**Rule**: Never use `unwrap()` or `expect()` in library code. Use `?` operator or explicit error handling. Tests may use `expect("descriptive message")`.

---

## 3. Rust Async Patterns

### 3a. Tokio Runtime

**Convention**: Tokio is the sole async runtime. Tests use `#[tokio::test]`. Background work uses `tokio::spawn`. `Handle::current()` captures the runtime handle for Drop impls.

**Evidence**:
- `codex-rs/exec/src/lib.rs:1832,1851` — `#[tokio::test]` on integration tests
- `codex-rs/tui/src/chatwidget.rs:1449,4510` — `tokio::spawn` for background tasks
- `codex-rs/app-server/src/thread_status.rs:44,54` — `Handle::current()` captured in struct for async Drop cleanup

**Rule**: Use Tokio as the sole async runtime. Use `#[tokio::test]` for async tests, `tokio::spawn` for background tasks, and `Handle::current()` only when you need async work in Drop impls.

### 3b. Channel Selection

**Convention**: `tokio::sync::broadcast` for pub/sub (multiple receivers), `tokio::sync::oneshot` for single request/response, `async_channel` for multi-producer/multi-consumer communication.

**Evidence**:
- `codex-rs/app-server/src/lib.rs:808,815` — `broadcast::error::RecvError` handling for thread creation notifications
- `codex-rs/app-server/src/orbit_code_message_processor.rs:296` — `tokio::sync::oneshot` for request/response
- `codex-rs/core/src/mcp_connection_manager.rs:28,1168` — `async_channel::Sender` for cross-task communication

**Rule**: Use `broadcast` for fan-out notifications, `oneshot` for single request/response pairs, `async_channel` for MPMC scenarios.

### 3c. Shared State

**Convention**: `Arc<tokio::sync::Mutex<T>>` for async-aware mutual exclusion, `Arc<tokio::sync::RwLock<T>>` for read-heavy configurations.

**Evidence**:
- `codex-rs/app-server/src/thread_status.rs:22` — `state: Arc<Mutex<ThreadWatchState>>`
- `codex-rs/network-proxy/src/runtime.rs:200` — `state: Arc<RwLock<ConfigState>>` (read-heavy config)

**Rule**: Use `tokio::sync::Mutex` for async-aware locking. Use `tokio::sync::RwLock` when reads vastly outnumber writes.

### 3d. `CancellationToken` for Graceful Shutdown

**Convention**: `tokio_util::sync::CancellationToken` with child tokens for hierarchical, coordinated shutdown of subsystems.

**Evidence**:
- `codex-rs/app-server/src/lib.rs:44,107,361` — Import, struct field `disconnect_sender: Option<CancellationToken>`, and shutdown handler parameter
- `codex-rs/core/src/mcp_connection_manager.rs:665,676` — Parent `CancellationToken` with `.child_token()` for individual MCP server lifecycle

**Rule**: Use `CancellationToken` for cooperative shutdown. Create child tokens for subsystems so cancelling the parent cascades.

### 3e. Task Collection

**Convention**: `JoinSet` for structured concurrency with Tokio tasks. `FuturesUnordered` for stream-based concurrent execution of futures.

**Evidence**:
- `codex-rs/core/src/mcp_connection_manager.rs:668,899` — `JoinSet::new()` for parallel MCP server initialization and resource collection
- `codex-rs/core/src/tools/handlers/agent_jobs.rs:18,919` — `FuturesUnordered::new()` for parallel watch channel status monitoring

**Rule**: Use `JoinSet` when spawning parallel Tokio tasks. Use `FuturesUnordered` when polling a collection of futures as a stream.

### 3f. Retry with Exponential Backoff + Jitter

**Convention**: Retries use exponential backoff (`base * 2^(attempt-1)`) with ±10% jitter. Retry on 429, 5xx, and transport failures.

**Evidence**:
- `codex-rs/codex-client/src/retry.rs:38-47` — `backoff()` function: `2^(attempt-1)` scaling with `random_range(0.9..1.1)` jitter
- `codex-rs/codex-client/src/retry.rs:8-20` — `RetryPolicy` struct with max attempts and delay
- `codex-rs/codex-client/src/retry.rs:23-36` — `should_retry()` checking 429, 5xx, and transport errors

**Rule**: Use exponential backoff with ±10% jitter for retries. Use saturating arithmetic to prevent overflow. Only retry on transient errors (429, 5xx, connection failures).

---

## 4. Rust Serde & Serialization

### 4a. Derive Combinations

**Convention**: Derive sets vary by type category. Protocol/API types include `TS` for TypeScript generation. Config types include `JsonSchema` for schema generation.

**Evidence**:
- `codex-rs/protocol/src/mcp.rs:12` — Protocol: `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]`
- `codex-rs/core/src/config/permissions.rs:21` — Config: `#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:588` — App-server v2: `#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS, ExperimentalApi)]`

**Rule**: Config types: `Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema`. Protocol types: add `TS`. App-server v2 types: add `TS` + `ExperimentalApi` where needed.

### 4b. `rename_all` by Context

**Convention**: `kebab-case` for TOML config files, `snake_case` for protocol/API, `camelCase` for app-server v2 wire format.

**Evidence**:
- `codex-rs/config/src/requirements_exec_policy.rs:76` — `#[serde(rename_all = "kebab-case")]` (config)
- `codex-rs/protocol/src/protocol.rs:268` — `#[serde(rename_all = "snake_case")]` (protocol)
- `codex-rs/app-server-protocol/src/protocol/v2.rs:385,2287` — `#[serde(rename_all = "camelCase")]` (app-server v2)

**Rule**: Use `kebab-case` for config TOML structs, `snake_case` for protocol types, `camelCase` for app-server v2 types. Exception: config RPC payloads use `snake_case` to mirror TOML keys.

### 4c. Field Attributes

**Convention**: `#[serde(default)]` for optional fields with defaults. `#[serde(skip_serializing_if = "Option::is_none")]` for optional serialization (config/protocol only, NOT v2 payloads).

**Evidence**:
- `codex-rs/core/src/config/types.rs:73` — `#[serde(default = "default_enabled")]` with custom default function
- `codex-rs/protocol/src/protocol.rs:108` — `#[serde(default, skip_serializing_if = "Option::is_none")]` on `Submission::trace`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2299` — v2 `Option<String>` with NO `skip_serializing_if` (uses `#[ts(optional = nullable)]` instead)

**Rule**: Use `#[serde(default)]` for optional config fields. Use `skip_serializing_if = "Option::is_none"` in config/protocol types. NEVER use `skip_serializing_if` on v2 `Option<T>` payload fields — use `#[ts(optional = nullable)]` instead.

### 4d. V2 Boolean Fields Pattern

**Convention**: Boolean fields that default to `false` use `#[serde(default, skip_serializing_if = "std::ops::Not::not")]` instead of `Option<bool>`.

**Evidence**:
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2303-2304` — `tty: bool` with `#[serde(default, skip_serializing_if = "std::ops::Not::not")]`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2308-2309` — `stream_stdin: bool` (same pattern)
- `codex-rs/app-server-protocol/src/protocol/v2.rs:551` — `defer_loading: bool` (same pattern)

**Rule**: For v2 boolean fields where omission means `false`, use `#[serde(default, skip_serializing_if = "std::ops::Not::not")] pub field: bool` — not `Option<bool>`.

### 4e. Custom Serialization Modules

**Convention**: `#[serde(with = "module")]` for custom type conversions, especially Duration ↔ seconds.

**Evidence**:
- `codex-rs/core/src/config/types.rs:87-89` — `#[serde(with = "option_duration_secs")]` for `startup_timeout_sec`
- `codex-rs/core/src/config/types.rs:279-303` — `mod option_duration_secs` implementing serialize/deserialize for `Option<Duration>` as `f64`
- `codex-rs/hooks/src/types.rs:70,140-145` — `#[serde(serialize_with = "serialize_triggered_at")]` converting `DateTime<Utc>` to RFC3339

**Rule**: Use `#[serde(with = "module")]` for custom serialization. Define the module adjacent to the types that use it.

### 4f. App-Server V2 TypeScript Export

**Convention**: All v2 types must include `#[ts(export_to = "v2/")]` to land TypeScript definitions in the correct namespace. Keep `#[serde(rename)]` and `#[ts(rename)]` aligned.

**Evidence**:
- `codex-rs/app-server-protocol/src/protocol/v2.rs:137` — `#[ts(export_to = "v2/")]` on `CodexErrorInfo`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2288` — `#[ts(export_to = "v2/")]` on `CommandExecParams`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:143-145` — Aligned renames: `#[serde(rename = "httpStatusCode")]` + `#[ts(rename = "httpStatusCode")]`

**Rule**: Always add `#[ts(export_to = "v2/")]` on v2 types. If using `#[serde(rename)]`, add matching `#[ts(rename)]`. For tagged unions, use `#[serde(tag = "type")]` and `#[ts(tag = "type")]`.

---

## 5. Rust Trait Design

### 5a. Thread Safety Bounds

**Convention**: Traits used across threads require explicit `Send + Sync` bounds (often with `+ 'static`).

**Evidence**:
- `codex-rs/core/src/tasks/mod.rs:113` — `pub(crate) trait SessionTask: Send + Sync + 'static`
- `codex-rs/core/src/tools/registry.rs:33` — `pub trait ToolHandler: Send + Sync`
- `codex-rs/secrets/src/lib.rs:88` — `pub trait SecretsBackend: Send + Sync`

**Rule**: Add `Send + Sync` bounds to any trait that will be used behind `Arc<dyn Trait>` or across thread boundaries. Add `+ 'static` when the trait object must outlive the current scope.

### 5b. `#[async_trait]` Usage

**Convention**: All async methods in traits use `#[async_trait]` from the `async-trait` crate.

**Evidence**:
- `codex-rs/core/src/tasks/mod.rs:112` — `#[async_trait]` on `SessionTask` with `async fn run()` and `async fn abort()`
- `codex-rs/core/src/tools/registry.rs:32` — `#[async_trait]` on `ToolHandler` with `async fn handle()`
- `codex-rs/network-proxy/src/runtime.rs:164,176` — `#[async_trait]` on `ConfigReloader` and `BlockedRequestObserver`

**Rule**: Use `#[async_trait]` for any trait with async methods. Place it directly above the trait definition.

### 5c. Associated Types with Bounds

**Convention**: Traits use associated types with trait bounds to enforce contracts while allowing flexibility.

**Evidence**:
- `codex-rs/core/src/tools/registry.rs:34` — `type Output: ToolOutput + 'static;`
- `codex-rs/core/src/tools/sandboxing.rs:233` — `type ApprovalKey: Hash + Eq + Clone + Debug + Serialize;`

**Rule**: Use associated types with bounds when a trait's output type must satisfy specific interfaces (e.g., `type Output: ToolOutput + 'static`).

### 5d. Default Method Implementations

**Convention**: Traits provide default implementations for optional behavior, reducing boilerplate for implementers.

**Evidence**:
- `codex-rs/core/src/tools/registry.rs:51-52` — `async fn is_mutating(&self, _invocation: &ToolInvocation) -> bool { false }` (default: non-mutating)
- `codex-rs/core/src/tasks/mod.rs:142-144` — `async fn abort(&self, ...) { let _ = (session, ctx); }` (default: no-op cleanup)
- `codex-rs/core/src/tools/sandboxing.rs:294-296` — `fn escalate_on_failure(&self) -> bool { true }` (default: escalate)

**Rule**: Provide default implementations for optional trait methods. Use no-op defaults for lifecycle hooks (abort, cleanup). Document the default behavior.

---

## 6. Rust Visibility & API Surface

### 6a. `pub(crate)` for Internal Types

**Convention**: Types and functions intended for crate-internal use are marked `pub(crate)`.

**Evidence**:
- `codex-rs/core/src/connectors.rs:50` — `pub(crate) struct AppToolPolicy`
- `codex-rs/core/src/tasks/mod.rs:82` — `pub(crate) struct SessionTaskContext`
- `codex-rs/core/src/features.rs` — `pub(crate) fn feature_for_key(key: &str) -> Option<Feature>`

**Rule**: Use `pub(crate)` for types/functions that need to be shared within a crate but should not be part of the public API.

### 6b. `pub(super)` for Module-Local APIs

**Convention**: Types and functions exposed only to the parent module use `pub(super)`.

**Evidence**:
- `codex-rs/core/src/tools/runtimes/shell/zsh_fork_backend.rs` — `pub(super) async fn maybe_run_shell_command(...)`, `pub(super) async fn maybe_prepare_unified_exec(...)`
- `codex-rs/core/src/tools/code_mode/protocol.rs` — `pub(super) enum CodeModeToolKind`, `pub(super) struct EnabledTool`, `pub(super) struct CodeModeToolCall`

**Rule**: Use `pub(super)` for types/functions that are implementation details of a sub-module but needed by the parent module.

### 6c. Protocol Crates: Full Public API

**Convention**: Protocol crates (`protocol`, `app-server-protocol`) export everything publicly since they define cross-crate message types.

**Evidence**:
- `codex-rs/protocol/src/lib.rs:1-21` — Nearly all modules declared `pub mod` (16 out of 17)
- `codex-rs/app-server-protocol/src/lib.rs:1-5` — Private modules with `pub use protocol::common::*;` and `pub use protocol::v2::*;` glob re-exports

**Rule**: Protocol/message-definition crates should export all types publicly. Use `pub mod` for the modules or glob `pub use` re-exports.

---

## 7. Rust Import Ordering

### 7a. One Import Per Line

**Convention**: `imports_granularity = "Item"` in rustfmt.toml enforces one import per `use` statement. No multi-imports.

**Evidence**:
- `codex-rs/rustfmt.toml:4` — `imports_granularity = "Item"`
- `codex-rs/tui/src/lib.rs:6-58` — Every import is individual: `use app::App;`, `use orbit_code_core::AuthManager;`, etc.
- `codex-rs/app-server-protocol/src/protocol/v2.rs:1-98` — All single-item imports

**Rule**: Always use one import per `use` statement. Never write `use X::{Y, Z};`. Run `just fmt` to enforce.

### 7b. Import Group Ordering

**Convention**: Imports are ordered: crate-local → workspace crates → external crates → std. Alphabetical within each group. (Note: `just fmt` handles this automatically.)

**Evidence**:
- `codex-rs/cli/src/main.rs:1-31` — External (clap) → workspace (orbit_code_*) → std
- `codex-rs/exec/src/lib.rs:7-96` — Local crate → workspace → external → std

**Rule**: Let `just fmt` order imports. The expected grouping is crate-local, then workspace, then external, then std — alphabetical within groups.

---

## 8. Rust Naming Conventions

### 8a. Crate Names

**Convention**: Hyphens in `Cargo.toml` package names, underscores in library names. Published names use `orbit-code-*` prefix.

**Evidence**:
- `codex-rs/cli/Cargo.toml:2,12` — Package: `name = "orbit-code"`, lib: `name = "orbit_code_cli"`
- `codex-rs/core/Cargo.toml:4,9` — Package: `name = "orbit-code-core"`, lib: `name = "orbit_code_core"`
- `codex-rs/Cargo.toml:82-143` — All ~50 workspace deps follow `orbit-code-*` pattern

**Rule**: Package names use `orbit-code-<name>` (hyphens). Library names use `orbit_code_<name>` (underscores). Binary names use `orbit-code` or `orbit-code-<tool>` (hyphens).

### 8b. App-Server Naming

**Convention**: Request payloads end in `*Params`, responses in `*Response`, notifications in `*Notification`. RPC methods use `<resource>/<method>` with singular resource names.

**Evidence**:
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2454,2530` — `ThreadStartParams` → `ThreadStartResponse`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:2122,2131` — `FsReadFileParams` → `FsReadFileResponse`
- `codex-rs/app-server-protocol/src/protocol/v2.rs:3828,3840` — `TurnStartParams` → `TurnStartResponse`

**Rule**: Name request types `*Params`, response types `*Response`, notification types `*Notification`. RPC methods: `thread/start`, `fs/readFile`, `turn/interrupt` (singular resource, camelCase method).

---

## 9. Rust CLI Patterns (clap)

### 9a. Parser Derive with Attributes

**Convention**: CLI structs use `#[derive(Debug, Parser)]` with `#[clap(...)]` for metadata and `#[clap(flatten)]` for shared option groups.

**Evidence**:
- `codex-rs/cli/src/main.rs:54-81` — `MultitoolCli` with `#[derive(Debug, Parser)]`, `#[clap(author, version)]`, three `#[clap(flatten)]` groups
- `codex-rs/exec/src/cli.rs:8-115` — `Cli` with `#[derive(Parser, Debug)]`, `#[command(version)]`, `#[arg(...)]` attributes

**Rule**: Use `#[derive(Debug, Parser)]` for CLI structs. Group shared options with `#[clap(flatten)]`. Use `#[arg(...)]` for individual arguments.

### 9b. Subcommands with Visible Aliases

**Convention**: Subcommand enums use `#[clap(subcommand)]` with `#[clap(visible_alias = "...")]` for short aliases.

**Evidence**:
- `codex-rs/cli/src/main.rs:86-87` — `#[clap(visible_alias = "e")] Exec(ExecCli)`
- `codex-rs/cli/src/main.rs:232-237` — `#[clap(visible_alias = "seatbelt")] Macos(SeatbeltCommand)`

**Rule**: Use `#[clap(visible_alias = "...")]` for commonly used subcommand abbreviations.

### 9c. Arg0 Dispatch Pattern

**Convention**: The binary inspects `argv[0]` to dispatch to different execution modes without needing subcommands (used for sandbox and patch binaries invoked via symlinks).

**Evidence**:
- `codex-rs/arg0/src/lib.rs:47-121` — `arg0_dispatch()` inspects `argv[0]` file name against constants (`LINUX_SANDBOX_ARG0`, `APPLY_PATCH_ARG0`, etc.)
- `codex-rs/arg0/src/lib.rs:144-170` — `arg0_dispatch_or_else()` generic async entry point with fallback

**Rule**: Use the arg0 crate for binary dispatch. Match against named constants, not raw strings. The arg0 check runs before clap parsing.

---

## 10. Rust Documentation Style

### 10a. Module-Level Docs

**Convention**: `//!` comments at the top of modules describe what the module does, its key types, and design patterns.

**Evidence**:
- `codex-rs/protocol/src/protocol.rs:1-4` — `//! Defines the protocol for a Codex session...` with SQ/EQ pattern explanation
- `codex-rs/core/src/lib.rs:1` — `//! Root of the codex-core library.`
- `codex-rs/arg0/src/lib.rs:123-143` — Extended doc explaining the arg0 symlink trick

**Rule**: Add `//!` module docs to every module describing its purpose. Include key types and design patterns for complex modules.

### 10b. Public Item Docs

**Convention**: `///` doc comments on all public items. Use `[`TypeName`]` syntax for cross-references.

**Evidence**:
- `codex-rs/protocol/src/protocol.rs:100` — `/// Submission Queue Entry - requests from user`
- `codex-rs/protocol/src/protocol.rs:243-244` — Doc links: `/// Similar to [`Op::UserInput`], but contains additional context required for a turn of a [`crate::orbit_code_thread::CodexThread`].`
- `codex-rs/protocol/src/protocol.rs:932-935` — Multi-line doc on public method

**Rule**: Document all public items with `///`. Use `[`TypeName`]` for cross-references. Skip docs for trivial getters and self-evident enum variants.

### 10c. Inline Comments

**Convention**: Inline `//` comments only for non-obvious logic. Never restate what code does.

**Evidence**:
- `codex-rs/protocol/src/protocol.rs:867-869` — `fn is_enabled(self) -> bool` has NO inline comment (trivially obvious)
- `codex-rs/protocol/src/protocol.rs:161-169` — `NetworkAccess` enum variants have no individual comments (self-evident)

**Rule**: Use `//` comments only for non-obvious logic, invariants, or "why" explanations. Never add comments that restate the code.

---

## 11. Rust Testing Patterns

### 11a. Test Directory Structure

**Convention**: `tests/all.rs` → `tests/suite/mod.rs` → `tests/suite/*.rs`. `tests/common/` for shared test utilities as a support crate.

**Evidence**:
- `codex-rs/core/tests/all.rs:1-3` — `mod suite;`
- `codex-rs/core/tests/common/lib.rs` — Shared test utilities with `wait_for_event`, `TestCodexBuilder`, etc.
- `codex-rs/core/tests/suite/` — 20+ test modules (client.rs, compact.rs, approvals.rs, etc.)

**Rule**: One `all.rs` aggregator per crate. Test modules in `suite/`. Shared utilities in `tests/common/` as a library crate.

### 11b. `pretty_assertions::assert_eq!`

**Convention**: Always use `pretty_assertions::assert_eq!` for complex type comparisons. Prefer whole-object comparison over field-by-field.

**Evidence**:
- `codex-rs/exec/src/cli.rs:264` — `use pretty_assertions::assert_eq;` at top of test module
- `codex-rs/core/tests/suite/client.rs:63` — Same import pattern

**Rule**: Import `use pretty_assertions::assert_eq;` in every test module. Compare entire objects with `assert_eq!`, not individual fields.

### 11c. Snapshot Tests (`insta`)

**Convention**: Any change that affects user-visible UI must include `insta` snapshot coverage.

**Evidence**:
- `codex-rs/core/tests/suite/compact.rs:1803-1816` — `insta::assert_snapshot!("pre_sampling_model_switch_compaction_shapes", ...)`
- `codex-rs/core/tests/suite/compact.rs:2424,2680,3053,3200` — Additional snapshot assertions

**Rule**: Add `insta` snapshot tests for any UI-affecting change. Run `cargo insta accept -p <crate>` to review and accept snapshots.

### 11d. `wiremock` for HTTP Mocking

**Convention**: Use `wiremock::MockServer` for HTTP mocking. Use the `mount_sse_once` / `sse(vec![...])` helpers from `core_test_support::responses`.

**Evidence**:
- `codex-rs/core/tests/suite/client.rs:69-76` — Full wiremock import set
- `codex-rs/core/tests/suite/client.rs:145-151` — `MockServer::start().await` + `mount_sse_once(&server, sse(vec![...]))`

**Rule**: Use `MockServer::start().await` for test HTTP servers. Prefer `mount_sse_once` over `mount_sse_once_match` or `mount_sse_sequence`. Use `ResponseMock::single_request()` for assertions.

### 11e. `TestCodexBuilder` Fluent API

**Convention**: Tests use a fluent builder pattern to configure test Codex instances.

**Evidence**:
- `codex-rs/core/tests/common/test_codex.rs:64-117` — `TestCodexBuilder` with `.with_config()`, `.with_auth()`, `.with_model()`, `.with_pre_build_hook()`, `.with_home()`

**Rule**: Use `TestCodexBuilder` for setting up integration tests. Chain configuration methods fluently.

### 11f. `wait_for_event()` with Predicate

**Convention**: Async event testing uses `wait_for_event(codex, predicate)` with a closure predicate. Default timeout is 1 second.

**Evidence**:
- `codex-rs/core/tests/common/lib.rs:218-226` — `wait_for_event()` delegates to `wait_for_event_with_timeout()` with 1s default
- `codex-rs/core/tests/common/lib.rs:237-256` — `wait_for_event_with_timeout()` loops with `tokio::time::timeout`

**Rule**: Use `wait_for_event(codex, |ev| matches!(ev, ...))` for async event assertions. Prefer `wait_for_event` over `wait_for_event_with_timeout`.

### 11g. Binary Resolution (`cargo_bin`)

**Convention**: Use `codex_utils_cargo_bin::cargo_bin("name")` for binary resolution in tests (works with both Cargo and Bazel).

**Evidence**:
- `codex-rs/core/tests/common/lib.rs:158` — `orbit_code_utils_cargo_bin::cargo_bin("codex-linux-sandbox")`
- `codex-rs/core/tests/common/lib.rs:39` — `orbit_code_utils_cargo_bin::repo_root()` for workspace root

**Rule**: Use `codex_utils_cargo_bin::cargo_bin()` instead of `assert_cmd::Command::cargo_bin()`. Use `find_resource!` instead of `env!("CARGO_MANIFEST_DIR")`.

### 11h. Global Test Setup (`#[ctor]`)

**Convention**: Global test initialization uses `#[ctor]` for deterministic behavior (e.g., setting process IDs, configuring insta workspace root).

**Evidence**:
- `codex-rs/core/tests/common/lib.rs:27-31` — `#[ctor]` enabling deterministic process IDs
- `codex-rs/core/tests/common/lib.rs:33-51` — `#[ctor]` configuring `INSTA_WORKSPACE_ROOT`

**Rule**: Use `#[ctor]` in `tests/common/lib.rs` for process-startup initialization. Place safety comments on `unsafe` env var mutations.

### 11i. Nextest Configuration

**Convention**: Default 15s slow-timeout with 2 termination attempts. Serial execution for code generation and integration test groups.

**Evidence**:
- `codex-rs/.config/nextest.toml:1-3` — `slow-timeout = { period = "15s", terminate-after = 2 }`
- `codex-rs/.config/nextest.toml:5-9` — Serial test groups: `app_server_protocol_codegen`, `app_server_integration` with `max-threads = 1`
- `codex-rs/.config/nextest.toml:12-29` — Per-test timeout overrides (rmcp_client: 1min, approval_matrix: 30s)

**Rule**: Run tests via `cargo nextest run`. Default timeout is 15s. Add test group overrides in `.config/nextest.toml` for slow tests.

---

## 12. Rust Macro Usage

### 12a. Derive Macros (Primary)

**Convention**: Derive macros are the dominant macro type. Most common: `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema, TS, Error, Parser`.

**Evidence**:
- `codex-rs/protocol/src/mcp.rs:12` — `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]`
- `codex-rs/core/src/config/permissions.rs:21` — `#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]`
- `codex-rs/core/src/error.rs:63` — `#[derive(Error, Debug)]`

**Rule**: Use derive macros as the primary mechanism for trait implementations. Prefer derives over manual impls unless custom logic is required.

### 12b. `/*param_name*/` Argument Comments

**Convention**: Use `/*param_name*/` comments before opaque literal arguments (booleans, `None`, numbers) when passing by position. The name must exactly match the callee signature.

**Evidence**:
- `codex-rs/network-proxy/src/policy.rs:61-68` — `/*prefix*/` annotations for CIDR prefix lengths
- `codex-rs/network-proxy/src/http_proxy.rs:190,472` — `/*audit_endpoint_override*/`, `/*port*/`
- `codex-rs/rmcp-client/src/perform_oauth_login.rs:95,98` — `/*launch_browser*/`, `/*timeout_secs*/`
- `codex-rs/core/src/tasks/compact.rs:35,42` — `/*inc*/` for counter increments

**Rule**: Use `/*param_name*/` before opaque literals (booleans, None, numbers). The comment must exactly match the callee's parameter name. Skip for string/char literals unless the comment adds clarity.

### 12c. Custom Macros (Rare)

**Convention**: Custom `macro_rules!` definitions are rare and used only for genuine boilerplate reduction.

**Evidence**:
- `codex-rs/app-server-protocol/src/protocol/v2.rs` — `v2_enum_from_core!` macro for generating v2 enum wrappers
- `codex-rs/exec/src/event_processor_with_human_output.rs` — Macro for event processing boilerplate
- `codex-rs/tui/src/frames.rs` — Macro for frame definitions

**Rule**: Avoid custom macros unless they eliminate significant, unavoidable boilerplate. Prefer functions, generics, or traits over macros.

### 12d. Platform-Specific Code

**Convention**: `#[cfg(...)]` for OS-specific code and feature gates.

**Evidence**:
- `codex-rs/core/src/error.rs:175` — `#[cfg(target_os = "linux")] LandlockRuleset(#[from] landlock::RulesetError)`
- `codex-rs/tui/tests/all.rs:7-9` — `#[cfg(feature = "vt100-tests")]`

**Rule**: Use `#[cfg(target_os = "...")]` for OS-specific code. Use `#[cfg(feature = "...")]` for feature-gated code. Place cfg attributes on the smallest possible scope.

---

## 13. Rust Clippy & Lint Rules

### 13a. Workspace-Level Denies (33 Rules)

**Convention**: 33 clippy lints are denied workspace-wide in `[workspace.lints.clippy]`.

**Evidence** (`codex-rs/Cargo.toml:316-348`):

Key denies:
- `unwrap_used`, `expect_used` — No panics in library code
- `uninlined_format_args` — Always `format!("{var}")` not `format!("{}", var)`
- `redundant_closure_for_method_calls` — Use method references over closures
- `redundant_clone`, `redundant_closure` — No unnecessary clones or closures
- `manual_*` (12 rules) — Use idiomatic Rust instead of manual implementations
- `needless_*` (8 rules) — No unnecessary borrows, collects, late inits, etc.
- `trivially_copy_pass_by_ref` — Pass Copy types by value, not reference

**Rule**: All 33 workspace clippy denies are enforced. Run `just fix -p <crate>` to auto-fix. Key rules: inline format args, use method references, no unwrap/expect in lib code.

### 13b. Disallowed Ratatui Methods

**Convention**: Specific ratatui color methods are banned to ensure theme compatibility.

**Evidence** (`codex-rs/clippy.toml`):
- `Color::Rgb` — "Use ANSI colors, which work better in various terminal themes."
- `Color::Indexed` — Same reason
- `.white()` — "Avoid hardcoding white; prefer default fg or dim/bold."
- `.black()` — "Avoid hardcoding black; prefer default fg or dim/bold."
- `.yellow()` — "Avoid yellow; prefer other colors in `tui/styles.md`."

**Rule**: Never use `Color::Rgb`, `Color::Indexed`, `.white()`, `.black()`, or `.yellow()` in TUI code. Use ANSI colors from `styles.md`.

### 13c. Error Size Threshold

**Convention**: `large-error-threshold = 256` bytes (increased from default 128).

**Evidence** (`codex-rs/clippy.toml`):
- `large-error-threshold = 256`

**Rule**: Keep error types under 256 bytes. Box large payloads if an error variant exceeds this.

---

## 14. Rust TUI Conventions (ratatui)

### 14a. Stylize Trait Helpers

**Convention**: Use ratatui's `Stylize` trait method chaining instead of `Span::styled` with `Style` objects.

**Evidence**:
- `codex-rs/tui/src/history_cell.rs:364` — `"› ".bold().dim()`
- `codex-rs/tui/src/history_cell.rs:512` — `update_action.command_str().cyan()`
- `codex-rs/tui/src/history_cell.rs:523-524` — `padded_emoji("✨").bold().cyan()`, `"Update available!".bold().cyan()`

**Rule**: Use `.red()`, `.dim()`, `.bold()`, `.cyan()`, `.magenta()`, `.underlined()` instead of manual Style construction. Chain styles: `text.cyan().underlined()`. Use `Span::styled` only for runtime-computed styles.

### 14b. Span and Line Construction

**Convention**: `"text".into()` for basic spans. `vec![...].into()` for lines. Avoid unnecessary type annotations.

**Evidence**:
- Per AGENTS.md TUI conventions and `codex-rs/tui/styles.md`

**Rule**: Use `"text".into()` for spans, `vec![span1, span2].into()` for lines. Use `Line::from(vec![...])` only when the target type isn't obvious. Don't refactor between equivalent forms without clear gain.

### 14c. Color Palette

**Convention**: Strict ANSI color palette. Headers: bold. Primary: default fg. Secondary: dim. Selection/tips: cyan. Success: green. Errors: red. Branding: magenta.

**Evidence**:
- `codex-rs/tui/styles.md:1-22` — Full style reference document
- `codex-rs/clippy.toml` — Disallowed: Rgb, Indexed, white, black, yellow

**Rule**: Follow `tui/styles.md`. Use only: bold, dim, cyan, green, red, magenta. Never use blue, yellow, white, black, Rgb, or Indexed colors.

### 14d. Text Wrapping

**Convention**: `textwrap::wrap` for plain strings. `adaptive_wrap_lines()` for content that may contain URLs. `word_wrap_lines()` for structured UI elements.

**Evidence**:
- `codex-rs/tui/src/wrapping.rs:490,507,638,793` — `adaptive_wrap_lines()`, `word_wrap_lines()` functions
- `codex-rs/tui/src/history_cell.rs:38,302,318` — `adaptive_wrap_lines()` usage for agent messages
- `codex-rs/tui/src/status_indicator_widget.rs:32,214` — `word_wrap_lines()` for status indicators

**Rule**: Use `adaptive_wrap_lines()` for content with potential URLs. Use `word_wrap_lines()` for plain text UI elements. Use `textwrap::wrap()` only for raw strings. Use `prefix_lines` from `line_utils` for indented multi-line output.

### 14e. Parallel TUI Implementations

**Convention**: Changes to `codex-rs/tui/` must be mirrored in `codex-rs/tui_app_server/` unless documented otherwise.

**Evidence**:
- Both directories contain matching files: `app.rs`, `app_event.rs`, `chatwidget.rs`, `bottom_pane/`, `history_cell.rs`, `wrapping.rs`, etc.
- AGENTS.md explicitly states this requirement

**Rule**: When modifying `tui/`, check if `tui_app_server/` has a parallel implementation and mirror the change.

---

## 15. Rust Config & Schema Patterns

### 15a. Config Types with Schema Generation

**Convention**: Config types derive `Serialize, Deserialize, JsonSchema` for automatic JSON Schema generation.

**Evidence**:
- `codex-rs/core/src/config/permissions.rs:21` — `#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema)]`
- `codex-rs/config/src/requirements_exec_policy.rs:76` — `#[serde(rename_all = "kebab-case")]` on config enums

**Rule**: Config types must derive `JsonSchema`. Run `just write-config-schema` after any `ConfigToml` change.

### 15b. Layered Config Loading

**Convention**: Configuration is loaded through a `ConfigLayerStack` that merges multiple sources (file, MDM, cloud) with defined precedence.

**Evidence**:
- `codex-rs/config/src/state.rs:27-33` — `ConfigLayerEntry` with source, config, raw TOML, version, disabled reason
- `codex-rs/config/src/state.rs:120` — `ConfigLayerStack` managing ordered layers
- `codex-rs/config/src/merge.rs` — `merge_toml_values()` for deep TOML merging

**Rule**: Use `ConfigLayerStack` for config loading. Never bypass the layered config system. Update `app-server/README.md` when config behavior changes.

---

## 16. Rust Dependency Management

### 16a. Workspace Dependencies

**Convention**: All dependencies declared in `[workspace.dependencies]` of root `Cargo.toml`. Per-crate `Cargo.toml` uses `{ workspace = true }`, adding features as needed.

**Evidence**:
- `codex-rs/Cargo.toml:79-310` — ~150 workspace dependencies
- `codex-rs/cli/Cargo.toml:22` — `orbit-code-app-server = { workspace = true }`
- `codex-rs/tui/Cargo.toml:65` — `ratatui = { workspace = true, features = ["scrolling-regions", ...] }` (workspace + feature override)

**Rule**: Add new dependencies to `[workspace.dependencies]` in root `Cargo.toml`. Per-crate Cargo.toml: use `{ workspace = true }`, add only crate-specific features.

### 16b. Patched Crates

**Convention**: Four crates are patched via git forks for custom fixes.

**Evidence** (`codex-rs/Cargo.toml:375-387`):
- `crossterm` — nornagon/color-query branch
- `ratatui` — nornagon-v0.29.0-patch branch
- `tokio-tungstenite` — openai-oss-forks, pinned rev
- `tungstenite` — openai-oss-forks, pinned rev

**Rule**: Use the patched forks. After any dependency change: run `just bazel-lock-update` and `just bazel-lock-check`.

### 16c. Standard Dev Dependencies

**Convention**: Consistent dev-dependency set across crates: `pretty_assertions`, `tempfile`, `wiremock`, `insta`.

**Evidence**:
- `codex-rs/config/Cargo.toml:29` — `pretty_assertions = { workspace = true }`
- `codex-rs/app-server/Cargo.toml:82,92` — `pretty_assertions`, `wiremock`
- `codex-rs/mcp-server/Cargo.toml:45-47` — `pretty_assertions`, `tempfile`, `wiremock`

**Rule**: Use workspace dev-dependencies: `pretty_assertions` for assertion diffs, `tempfile` for temporary directories, `wiremock` for HTTP mocking, `insta` for snapshot testing.

---

## 17. TypeScript Patterns (SDK & CLI)

### 17a. ESM-First

**Convention**: All TypeScript packages use `"type": "module"` in `package.json`.

**Evidence**:
- `codex-cli/package.json:8` — `"type": "module"`
- `sdk/typescript/package.json:18` — `"type": "module"`

**Rule**: Always set `"type": "module"` in package.json. Use `import`/`export` syntax exclusively.

### 17b. tsup for Bundling

**Convention**: SDK and MCP server use tsup for bundling with ESM output and `.d.ts` generation.

**Evidence**:
- `sdk/typescript/tsup.config.ts:1-12` — ESM format, dts: true
- `shell-tool-mcp/tsup.config.ts:1-15` — CJS format for MCP compatibility

**Rule**: Use tsup for TypeScript bundling. SDK: ESM output. MCP servers: CJS output for compatibility.

### 17c. Strict TypeScript

**Convention**: `strict: true` and `noUncheckedIndexedAccess: true` in tsconfig.

**Evidence**:
- `sdk/typescript/tsconfig.json:10-11` — Both enabled
- Target: ES2022, module: ESNext, moduleResolution: bundler

**Rule**: Enable `strict: true` and `noUncheckedIndexedAccess: true`. Target ES2022 with ESNext modules.

### 17d. Node Protocol Imports

**Convention**: Always use `node:` prefix for Node.js built-in imports.

**Evidence**:
- `sdk/typescript/src/exec.ts:1-4` — `import from "node:child_process"`, `"node:path"`, `"node:readline"`, `"node:module"`
- `codex-cli/bin/codex.js:4-8` — `"node:child_process"`, `"node:module"`, `"node:path"`, `"node:url"`

**Rule**: Always use `node:` prefix for built-in imports: `import from "node:fs"`, `import from "node:path"`, etc.

### 17e. Export Patterns

**Convention**: `export type {}` for type-only exports. `export class {}` for concrete implementations.

**Evidence**:
- `sdk/typescript/src/index.ts:1-13` — `export type { ... }` for event types
- `sdk/typescript/src/codex.ts:11` — `export class Codex`
- `sdk/typescript/src/exec.ts:57` — `export class CodexExec`

**Rule**: Use `export type` for type-only re-exports from index.ts. Use `export class` for concrete implementations.

### 17f. Formatting

**Convention**: Prettier with project-specific configs. Root: 80 printWidth. SDK: 100 printWidth.

**Evidence**:
- `.prettierrc.toml:1-8` — Root config: printWidth 80
- `sdk/typescript/.prettierrc:1-5` — Override: printWidth 100

**Rule**: Follow project Prettier config. Use `trailingComma: "all"`.

### 17g. ESLint

**Convention**: Flat config with typescript-eslint. `_` prefix for unused parameters.

**Evidence**:
- `sdk/typescript/eslint.config.js:1-21` — Flat config with typescript-eslint
- `sdk/typescript/eslint.config.js:13-18` — `argsIgnorePattern: "^_"` and `varsIgnorePattern: "^_"`

**Rule**: Use ESLint flat config. Prefix unused parameters with `_`.

### 17h. Testing

**Convention**: Jest with ts-jest preset for ESM testing.

**Evidence**:
- `sdk/typescript/jest.config.cjs:3` — `preset: "ts-jest/presets/default-esm"`
- `shell-tool-mcp/jest.config.cjs:2` — `preset: "ts-jest"`

**Rule**: Use Jest with ts-jest. SDK: use `default-esm` preset. MCP: use standard preset.

### 17i. CLI Binary Resolution

**Convention**: codex-cli is pure JS that resolves platform-specific Rust binaries via `require.resolve` and spawns them.

**Evidence**:
- `codex-cli/bin/codex.js:89-105` — Platform-specific binary resolution via `require.resolve`
- `codex-cli/bin/codex.js:176-179` — `spawn` with `stdio: "inherit"` and env passthrough

**Rule**: The CLI launcher resolves platform binaries via npm optional dependencies, then `spawn`s them with inherited stdio.

---

## 18. Python SDK Patterns

### 18a. Build System

**Convention**: Hatchling build system with Pydantic v2 models and src/ layout.

**Evidence**:
- `sdk/python/pyproject.toml:1-3` — `build-system` with `hatchling`
- `sdk/python/pyproject.toml:45` — `packages = ["src/orbit_code_app_server"]`

**Rule**: Use Hatchling for Python builds. Place packages under `src/`.

### 18b. Pydantic v2 Models

**Convention**: Generated models use Pydantic v2 with `BaseModel`, `ConfigDict`, and `Field` aliases.

**Evidence**:
- `sdk/python/src/orbit_code_app_server/generated/v2_all.py:5` — `from pydantic import BaseModel, ConfigDict, Field`
- `sdk/python/src/orbit_code_app_server/generated/v2_all.py:10-14` — `model_config = ConfigDict(...)` usage
- `sdk/python/src/orbit_code_app_server/generated/v2_all.py:36-52` — Fields with aliases

**Rule**: Use Pydantic v2 `BaseModel` with `ConfigDict`. Auto-generate models from JSON schema.

### 18c. Testing & Linting

**Convention**: pytest with `-q` flag. ruff for linting.

**Evidence**:
- `sdk/python/pyproject.toml:60-62` — `addopts = "-q"`
- `sdk/python/pyproject.toml:33` — `ruff>=0.11` in dev dependencies

**Rule**: Run pytest with `-q`. Use ruff for linting.

---

## 19. Build & Release Pipeline

### 19a. Task Runner

**Convention**: `just` is the primary task runner. Justfile in root with `set working-directory := "codex-rs"`.

**Evidence**:
- `justfile:1` — `set working-directory := "codex-rs"`
- `justfile:28-29` — `fmt: cargo fmt -- --config imports_granularity=Item`
- `justfile:47-48` — `test: cargo nextest run --no-fail-fast`

**Rule**: Use `just` commands for all standard operations: `just fmt`, `just test`, `just fix`, `just codex`.

### 19b. Dual Build Systems

**Convention**: Cargo for local development, Bazel for CI/release builds.

**Evidence**:
- `justfile:53-55` — `bazel-codex` target for Bazel builds
- `codex-rs/Cargo.toml:360-368` — Release profile: LTO fat, strip symbols, single codegen unit

**Rule**: Use Cargo for local dev. Use Bazel for CI/release. After dependency changes: `just bazel-lock-update` + `just bazel-lock-check`.

### 19c. pnpm Workspace

**Convention**: pnpm workspace for JavaScript packages with Node >=22 requirement.

**Evidence**:
- `package.json` — `"engines": { "node": ">=22" }`
- `pnpm-workspace.yaml:1-7` — Packages: codex-cli, sdk/typescript, shell-tool-mcp

**Rule**: Use pnpm for JS package management. Require Node >=22.

---

## 20. Code Quality Gates & Workflow

### 20a. Format After Changes

**Convention**: Run `just fmt` automatically after Rust code changes. No approval needed.

**Evidence**:
- AGENTS.md: "Run `just fmt` automatically after you have finished making Rust code changes; do not ask for approval to run it."

**Rule**: Run `just fmt` after every Rust change. It runs `cargo fmt -- --config imports_granularity=Item`.

### 20b. Lint Before Finalizing

**Convention**: Run `just fix -p <crate>` before finalizing large changes. Scope with `-p` to avoid slow workspace builds.

**Evidence**:
- AGENTS.md: "Before finalizing a large change, run `just fix -p <project>`"

**Rule**: Run `just fix -p <crate>` before finalizing. Only run `just fix` without `-p` if you changed shared crates.

### 20c. Test Incrementally

**Convention**: Test the changed crate first. Full suite only if core/common/protocol changed.

**Evidence**:
- AGENTS.md: "Run the test for the specific project that was changed... if any changes were made in common, core, or protocol, run the complete test suite"

**Rule**: `cargo test -p <crate>` for targeted testing. Full `just test` only for shared crate changes. Avoid `--all-features` for routine runs.

### 20d. Snapshot Tests for UI

**Convention**: Any UI-affecting change requires corresponding `insta` snapshot coverage.

**Evidence**:
- AGENTS.md: "any change that affects user-visible UI must include corresponding `insta` snapshot coverage"

**Rule**: Add or update `insta` snapshots for every UI change. Run `cargo insta accept -p <crate>` to review.

### 20e. Schema Regeneration

**Convention**: Regenerate schemas when API shapes change.

**Evidence**:
- AGENTS.md: "If you change `ConfigToml`, run `just write-config-schema`"
- AGENTS.md: "Regenerate schema fixtures: `just write-app-server-schema`"

**Rule**: Run `just write-config-schema` after ConfigToml changes. Run `just write-app-server-schema` after API shape changes.

---

## CLAUDE.md Ready Rules

### Rust — Module Organization
1. Declare modules private by default with `mod foo;`. Re-export only needed types via `pub use foo::Type;` in lib.rs.
2. Use `pub mod` only for major subsystem modules that form the crate's public API surface.
3. Use a single `.rs` file for focused modules. Use a subdirectory with `mod.rs` when a module has 3+ sub-modules.
4. Place integration tests in `tests/all.rs` → `tests/suite/mod.rs` → `tests/suite/*.rs`. Use `tests/common/` for shared utilities.
5. Target modules under 500 LoC excluding tests. At ~800 LoC, add new functionality in a new module. Do not create single-use helper methods.

### Rust — Error Handling
6. Define error types with `#[derive(Debug, Error)]` from thiserror. Every variant must have `#[error(...)]`.
7. Define `pub type Result<T> = std::result::Result<T, YourError>;` alongside each crate-level error type.
8. Prefer `#[from]` for direct error wrapping. Use manual `From` impls only when conversion requires custom logic.
9. Add domain-specific query methods to error types (e.g., `is_retryable()`). Use exhaustive matches, not wildcards.
10. Never use `unwrap()` or `expect()` in library code. Use `?` or explicit error handling. Tests may use `expect("descriptive message")`.

### Rust — Async Patterns
11. Use Tokio as the sole async runtime. `#[tokio::test]` for async tests, `tokio::spawn` for background tasks.
12. Use `broadcast` for fan-out notifications, `oneshot` for request/response, `async_channel` for MPMC.
13. Use `tokio::sync::Mutex` for async locking, `tokio::sync::RwLock` for read-heavy state.
14. Use `CancellationToken` with child tokens for hierarchical shutdown.
15. Use `JoinSet` for parallel Tokio tasks, `FuturesUnordered` for stream-based concurrent futures.
16. Retries: exponential backoff with ±10% jitter, saturating arithmetic, only for transient errors (429, 5xx, connection failures).

### Rust — Serialization
17. Config types: `Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, JsonSchema`. Protocol types: add `TS`. App-server v2: add `TS` + `ExperimentalApi`.
18. Use `kebab-case` for config TOML, `snake_case` for protocol, `camelCase` for app-server v2. Exception: config RPC payloads use `snake_case`.
19. NEVER use `skip_serializing_if` on v2 `Option<T>` payload fields — use `#[ts(optional = nullable)]` instead. For v2 booleans defaulting to false: `#[serde(default, skip_serializing_if = "std::ops::Not::not")]`.
20. Always add `#[ts(export_to = "v2/")]` on v2 types. Keep `#[serde(rename)]` and `#[ts(rename)]` aligned.

### Rust — Traits & Visibility
21. Add `Send + Sync` bounds to traits used behind `Arc<dyn Trait>`. Add `+ 'static` when the trait object must outlive its scope.
22. Use `#[async_trait]` for any trait with async methods.
23. Provide default implementations for optional trait methods. Document the default behavior.
24. Use `pub(crate)` for crate-internal types, `pub(super)` for parent-module-only types. Protocol crates: everything public.

### Rust — Imports & Naming
25. One import per `use` statement (enforced by `imports_granularity = "Item"`). Run `just fmt` to enforce.
26. Package names: `orbit-code-<name>` (hyphens). Library names: `orbit_code_<name>` (underscores). Binary names: `orbit-code[-<tool>]`.
27. App-server types: `*Params`, `*Response`, `*Notification`. RPC methods: `<singular-resource>/<camelCaseMethod>`.

### Rust — CLI
28. Use `#[derive(Debug, Parser)]` with `#[clap(flatten)]` for shared option groups and `#[clap(visible_alias)]` for subcommand aliases.

### Rust — Documentation
29. Add `//!` module docs to every module. Document public items with `///`. Use `[`TypeName`]` for cross-references. Skip docs for trivial getters and self-evident variants.
30. Use `//` inline comments only for non-obvious logic. Never restate what the code does.

### Rust — Testing
31. Use `pretty_assertions::assert_eq!` in every test module. Compare entire objects, not individual fields.
32. Add `insta` snapshot tests for any UI-affecting change.
33. Use `wiremock::MockServer` and `mount_sse_once` helpers for HTTP mocking. Use `TestCodexBuilder` for integration test setup.
34. Use `wait_for_event(codex, predicate)` for async event assertions. Use `codex_utils_cargo_bin::cargo_bin()` for binary resolution.
35. Use `#[ctor]` in `tests/common/lib.rs` for process-startup initialization.

### Rust — Macros & Lints
36. Use `/*param_name*/` comments before opaque literal arguments. Name must exactly match the callee's parameter.
37. Follow all 33 workspace clippy denies. Key: inline format args, method references over closures, no unwrap/expect in lib code.
38. Never use `Color::Rgb`, `Color::Indexed`, `.white()`, `.black()`, or `.yellow()` in TUI code.

### Rust — TUI
39. Use Stylize trait helpers: `.red()`, `.dim()`, `.bold()`, `.cyan()`. Use `Span::styled` only for runtime-computed styles.
40. Follow `tui/styles.md` color palette. Headers: bold. Secondary: dim. Selection: cyan. Success: green. Errors: red. Branding: magenta.
41. Use `adaptive_wrap_lines()` for content with URLs, `word_wrap_lines()` for plain text, `textwrap::wrap()` for raw strings.
42. Mirror all `tui/` changes in `tui_app_server/` unless documented otherwise.

### Rust — Config & Dependencies
43. Run `just write-config-schema` after ConfigToml changes. Run `just write-app-server-schema` after API shape changes.
44. Add dependencies to workspace `[workspace.dependencies]`. Per-crate: `{ workspace = true }` with crate-specific features only.
45. After dependency changes: `just bazel-lock-update` + `just bazel-lock-check`.

### TypeScript
46. ESM-first: `"type": "module"`. Use `node:` prefix for built-in imports. Target ES2022.
47. `export type` for type-only re-exports. `export class` for concrete implementations. `_` prefix for unused parameters.
48. tsup for bundling. Jest with ts-jest for testing. Prettier + ESLint flat config for formatting/linting.

### Python
49. Hatchling build system. Pydantic v2 models. `src/` layout. pytest with `-q`. ruff for linting.

### Build & Workflow
50. Run `just fmt` after every Rust change (no approval needed).
51. Run `just fix -p <crate>` before finalizing large changes.
52. `cargo test -p <crate>` for targeted testing. Full `just test` only for shared crate changes. Avoid `--all-features`.
53. Use `just` as the primary task runner. Cargo for local dev, Bazel for CI/release.
