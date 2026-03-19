# codex-rs/artifacts/src/runtime/

This file applies to `codex-rs/artifacts/src/runtime/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-artifacts` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-artifacts`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Artifact runtime discovery, installation, validation, and JavaScript executable resolution.

### What this folder does

Manages the full lifecycle of the artifact runtime package (`@oai/artifact-tool`): locating cached installations on disk, downloading and extracting new versions from GitHub releases via `codex-package-manager`, validating `package.json` metadata, and resolving a suitable JavaScript runtime (Node.js or Electron) to execute the build entrypoint.

### Where it plugs in

- Consumed by `client.rs` (one level up), which uses `InstalledArtifactRuntime` and `ArtifactRuntimeManager` to execute artifact builds
- Delegates download/extraction/caching to `codex-package-manager` via the `ManagedPackage` trait implementation
- All public types are re-exported through `mod.rs` up to `lib.rs`

### Imports from

- `codex_package_manager` -- `ManagedPackage`, `PackageManager`, `PackageManagerConfig`, `PackagePlatform`, `PackageReleaseArchive`, `PackageManagerError`
- `reqwest` -- HTTP client (passed through to `PackageManager`)
- `serde` / `serde_json` -- `ReleaseManifest` and `package.json` deserialization
- `url` -- manifest and archive URL construction
- `which` -- PATH-based discovery of `node` and `electron` executables

### Exports to

Via `mod.rs`, exports to `lib.rs`:

- `ArtifactRuntimeManager` / `ArtifactRuntimeManagerConfig` / `ArtifactRuntimeReleaseLocator` -- runtime resolver and installer
- `InstalledArtifactRuntime` / `load_cached_runtime` -- on-disk runtime loading
- `JsRuntime` / `JsRuntimeKind` -- JS executable metadata
- `ReleaseManifest` -- release metadata type
- `ArtifactRuntimeError` -- error enum
- `ArtifactRuntimePlatform` -- re-export of `PackagePlatform`
- `is_js_runtime_available` / `can_manage_artifact_runtime` -- capability checks
- Several `pub(crate)` helpers for cross-module use within the crate

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module declarations and re-exports (public + crate-internal) |
| `manager.rs` | `ArtifactRuntimeManager` -- wraps `PackageManager` to resolve/install runtimes; implements `ManagedPackage` trait for `ArtifactRuntimePackage` |
| `installed.rs` | `InstalledArtifactRuntime` -- loads an extracted runtime directory, validates `package.json` (name, version, exports), resolves build entrypoint path; `detect_runtime_root` finds the runtime root inside an extraction directory |
| `js_runtime.rs` | `JsRuntime` / `JsRuntimeKind` -- discovers Node.js and Electron executables on the system; checks Codex desktop app bundles on macOS, Windows, and Linux |
| `manifest.rs` | `ReleaseManifest` -- serde struct for release metadata JSON (schema version, runtime version, per-platform archives) |
| `error.rs` | `ArtifactRuntimeError` -- error variants for package manager failures, I/O errors, invalid metadata, missing JS runtime |
