# codex-rs/package-manager/src/

This file applies to `codex-rs/package-manager/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-package-manager` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-package-manager`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-package-manager` crate.

### What this folder does

Contains the implementation of the generic package download, caching, and installation framework.

### Key files

- `lib.rs` -- Module declarations and public re-exports of all main types.

- `manager.rs` -- `PackageManager<P>` implementation:
  - `resolve_cached()` / `resolve_cached_at()` -- Check for valid cached install
  - `ensure_installed()` -- Full install flow: fast-path cache check, file-lock acquisition, manifest fetch, archive download, verification, extraction, and loading
  - Uses `fd-lock` (`FileRwLock`) for concurrent install safety with polling (`INSTALL_LOCK_POLL_INTERVAL`: 50ms)
  - Downloads via `reqwest::Client`

- `package.rs` -- `ManagedPackage` trait:
  - Associated types: `Error`, `Installed`, `ReleaseManifest`
  - Required methods: `default_cache_root_relative`, `version`, `manifest_url`, `archive_url`, `release_version`, `platform_archive`, `install_dir`, `load_installed`
  - Default method: `detect_extracted_root` -- Finds the package root in extracted archives (looks for `manifest.json`)

- `config.rs` -- `PackageManagerConfig<P>`:
  - Holds `codex_home`, `package`, and optional `cache_root` override
  - `cache_root()` -- Derives effective cache directory

- `platform.rs` -- `PackagePlatform`:
  - Six variants: DarwinArm64, DarwinX64, LinuxArm64, LinuxX64, WindowsArm64, WindowsX64
  - `detect_current()` -- Auto-detects from `std::env::consts`
  - `as_str()` -- Returns platform string for cache/manifest keys

- `archive.rs` -- Archive handling:
  - `PackageReleaseArchive` -- Metadata struct (path, sha256, format, size)
  - `ArchiveFormat` -- Enum: TarGz, Zip
  - `extract_archive()` -- Dispatches to tar.gz or zip extraction
  - `verify_sha256()` / `verify_archive_size()` -- Integrity checks
  - `detect_single_package_root()` -- Finds single top-level directory in extracted archives

- `error.rs` -- `PackageManagerError` enum with variants for all failure modes (download, verification, extraction, platform, etc.)

- `tests.rs` -- Integration tests using `wiremock` for HTTP mocking

### Imports from / exports to

**Key imports:**
- `fd_lock::RwLock` -- File locking
- `reqwest::Client` -- HTTP downloads
- `sha2::{Sha256, Digest}` -- Hash verification
- `flate2`, `tar`, `zip` -- Archive extraction
- `tempfile::tempdir_in` -- Atomic installs

**All public types re-exported through `lib.rs`.**
