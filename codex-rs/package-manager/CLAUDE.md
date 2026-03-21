# codex-rs/package-manager/

Generic framework for downloading, verifying, caching, and installing versioned binary packages from remote sources.

## Build & Test
```bash
cargo build -p orbit-code-package-manager
cargo test -p orbit-code-package-manager
```

## Architecture

`PackageManager<P>` orchestrates the full install lifecycle: check cache, acquire a file lock for concurrent safety, fetch a release manifest, download the archive, verify SHA-256 hash and size, extract (tar.gz or zip), and load the installed package. Downstream crates implement the `ManagedPackage` trait to describe how their specific package is located, validated, and loaded. `PackagePlatform` auto-detects OS/arch at runtime. The cache is organized under `CODEX_HOME` with file-lock-based coordination via `fd-lock` to prevent parallel installs from corrupting shared state.

## Key Considerations
- File locking uses `fd-lock` with polling at 50ms intervals (`INSTALL_LOCK_POLL_INTERVAL`) -- not event-driven
- `ManagedPackage::detect_extracted_root()` looks for `manifest.json` to find the package root in extracted archives -- if your package doesn't have one, override this method
- Archive extraction supports both tar.gz and zip with single-root directory detection
- `PackagePlatform` has six variants (Darwin/Linux/Windows x Arm64/X64) -- no FreeBSD/OpenBSD support
- Tests use `wiremock` for HTTP mocking of download/manifest endpoints
