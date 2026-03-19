# codex-rs/package-manager/

This file applies to `codex-rs/package-manager/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-package-manager` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-package-manager`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-package-manager` -- Generic package download, caching, and installation manager.

### What this crate does

Provides a reusable framework for downloading, verifying, caching, and installing versioned binary packages from remote sources. It handles platform detection, archive extraction (tar.gz, zip), SHA-256 verification, file-locking for concurrent installs, and cache management.

### Main types

- `PackageManager<P>` -- Core manager that orchestrates download, verify, extract, and cache operations for a package type `P`
  - `resolve_cached()` -- Checks if a valid cached install exists for the current platform
  - `ensure_installed()` -- Ensures the package is installed (fast path for cached, full download otherwise)
- `ManagedPackage` trait -- Describes how a specific package is located, validated, and loaded:
  - Associated types: `Error`, `Installed`, `ReleaseManifest`
  - Methods: `version()`, `manifest_url()`, `archive_url()`, `platform_archive()`, `install_dir()`, `load_installed()`, etc.
- `PackageManagerConfig<P>` -- Configuration: Codex home directory, package instance, optional cache root override
- `PackagePlatform` -- Enum for supported OS/arch combinations (DarwinArm64, DarwinX64, LinuxArm64, LinuxX64, WindowsArm64, WindowsX64)
- `PackageReleaseArchive` -- Archive metadata (URL path, SHA-256 hash, format, expected size)
- `ArchiveFormat` -- Supported formats: TarGz, Zip
- `PackageManagerError` -- Error enum for all failure modes

### Key behaviors

- **Concurrent safety**: Uses `fd-lock` file locks to prevent parallel installs from corrupting the cache
- **Integrity verification**: SHA-256 hash and size verification of downloaded archives
- **Platform detection**: Automatic OS/arch detection via `std::env::consts`
- **Archive extraction**: Supports both tar.gz and zip formats with single-root detection

### What it plugs into

- Used by `codex-core` or other crates that need to manage external tool installations (e.g., LanceDB native libraries)
- Downstream crates implement the `ManagedPackage` trait for their specific packages

### Imports from / exports to

**Dependencies:**
- `fd-lock` -- File locking for concurrent install safety
- `flate2` -- Gzip decompression
- `reqwest` -- HTTP client for downloads
- `sha2` -- SHA-256 verification
- `tar` -- Tar archive extraction
- `tempfile` -- Temporary directories for atomic installs
- `tokio` -- Async runtime
- `url` -- URL handling
- `zip` -- Zip archive extraction

**Exports:**
- `PackageManager`, `ManagedPackage`, `PackageManagerConfig`, `PackagePlatform`, `PackageReleaseArchive`, `ArchiveFormat`, `PackageManagerError`

### Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Module declarations and re-exports
- `src/manager.rs` -- `PackageManager` implementation (download, verify, extract, cache)
- `src/package.rs` -- `ManagedPackage` trait definition
- `src/config.rs` -- `PackageManagerConfig`
- `src/platform.rs` -- `PackagePlatform` enum and detection
- `src/archive.rs` -- `PackageReleaseArchive`, `ArchiveFormat`, extraction and verification functions
- `src/error.rs` -- `PackageManagerError` definition
- `src/tests.rs` -- Integration tests
