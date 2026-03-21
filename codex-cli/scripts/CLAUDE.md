# codex-cli/scripts/

Build, packaging, and deployment scripts for producing publishable npm tarballs of the `@orbit.build/orbit-code` package and its platform-specific variants.

## Build & Test

These scripts are invoked by the repo-root `scripts/stage_npm_packages.py`, not run directly in most workflows:

```bash
python install_native_deps.py --run-id <GH_RUN_ID>   # download CI artifacts to vendor/
python build_npm_package.py <package-name> --version <ver> --vendor-dir vendor/   # stage + pack
```

## Architecture

The pipeline has two stages: `install_native_deps.py` downloads Rust build artifacts and ripgrep from GitHub (via `gh run download` and GitHub Releases), then `build_npm_package.py` stages a clean npm package directory with the native binaries and produces a tarball. The Docker scripts (`build_container.sh`, `run_in_container.sh`, `init_firewall.sh`) handle sandboxed container execution with iptables-based network restrictions.

## Key Considerations

- `build_npm_package.py` defines the `PACKAGE_NATIVE_COMPONENTS` and `CODEX_PLATFORM_PACKAGES` mappings -- these are the source of truth for which binaries go into which npm package.
- The repo-root `scripts/stage_npm_packages.py` imports from `build_npm_package.py` dynamically, so the two must stay in sync.
- Container scripts assume Docker and iptables/ipset are available; they default to allowing only `api.openai.com`.
