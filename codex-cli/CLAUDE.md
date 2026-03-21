# codex-cli/

npm wrapper package (`@orbit.build/orbit-code`) that resolves and spawns the platform-specific native Rust binary. No runtime npm dependencies -- the package relies entirely on native binaries bundled in `vendor/`.

## Build & Test

```bash
pnpm install    # install dependencies (dev-only)
```

There is no build step for this package itself -- it ships raw ESM JavaScript. The native binaries it wraps are built from `codex-rs/` and packaged by `scripts/build_npm_package.py`.

## Architecture

The entry point `bin/codex.js` maps `process.platform` + `process.arch` to a Rust target triple, resolves the native binary from the corresponding `@orbit.build/orbit-code-{platform}-{arch}` optional dependency (or a local `vendor/` fallback), prepends the vendor directory to `$PATH` for bundled `rg`, and spawns the binary with full signal forwarding.

Platform-specific packages are published for linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64, and win32-arm64.

## Key Considerations

- The `bin/rg` file is a DotSlash manifest (JSON), not a binary -- it describes how to fetch ripgrep 15.1.0 per platform.
- `scripts/` contains the build pipeline for producing npm tarballs from CI artifacts. The repo-root `scripts/stage_npm_packages.py` orchestrates these.
- The Dockerfile in this directory is for sandboxed container execution with network firewall rules.
