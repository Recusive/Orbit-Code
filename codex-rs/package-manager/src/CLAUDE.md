# codex-rs/package-manager/src/

Source for the `orbit-code-package-manager` crate -- generic package download/cache/install framework.

## Module Layout
- **Manager** (`manager.rs`): `PackageManager<P>` with `resolve_cached()` and `ensure_installed()` driving the full download-verify-extract-cache flow; file locking via `fd-lock`
- **Package trait** (`package.rs`): `ManagedPackage` trait defining how a package is located, validated, and loaded; includes `detect_extracted_root()` default looking for `manifest.json`
- **Archive** (`archive.rs`): `PackageReleaseArchive`, `ArchiveFormat` (TarGz/Zip), extraction, SHA-256 verification, size verification, single-root detection
- **Config** (`config.rs`): `PackageManagerConfig<P>` with codex home, package instance, and optional cache root override
- **Platform** (`platform.rs`): `PackagePlatform` enum with `detect_current()` auto-detection from `std::env::consts`
- **Error** (`error.rs`): `PackageManagerError` covering all failure modes
