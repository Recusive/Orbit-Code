# .devcontainer/ — Containerized Development Environment

## What This Folder Does

Provides Docker and VS Code Dev Container configuration for developing and building the Codex Rust codebase inside a Linux container. This is primarily useful for verifying Linux builds when working on a macOS host, and for cross-platform CI reproducibility.

## Key Files

| File | Role |
|------|------|
| `Dockerfile` | Ubuntu 24.04-based image with Rust toolchain, musl target, clippy, rustfmt, and build dependencies (clang, pkg-config, libcap-dev, libssl-dev, just). Runs as the default `ubuntu` user (UID 1000). |
| `devcontainer.json` | VS Code Dev Container spec. Builds the Dockerfile for `linux/arm64`, sets `CARGO_TARGET_DIR` to a platform-specific path to avoid conflicts with host builds, and installs rust-analyzer + even-better-toml extensions. |
| `README.md` | Usage instructions for both standalone Docker and VS Code Dev Container workflows. |

## What It Plugs Into

- **VS Code**: Recognized automatically by VS Code's Dev Containers extension. Opening the repo in VS Code triggers a prompt to reopen in the container.
- **Docker**: Can be used standalone with `docker build` and `docker run` for manual Linux builds.
- **`codex-rs/`**: The container's working directory is `/workspace`, and builds target the Rust code in `codex-rs/`.
- **Root `justfile`**: The `just` task runner is installed in the container image.

## Dockerfile Details

Base: `ubuntu:24.04` with `universe` repo enabled.

Installed packages:
- Build essentials: `build-essential`, `curl`, `git`, `ca-certificates`
- Rust dependencies: `pkg-config`, `libcap-dev`, `clang`, `musl-tools`, `libssl-dev`
- Task runner: `just`

Rust setup (as `ubuntu` user):
- Minimal rustup profile
- `aarch64-unknown-linux-musl` target pre-installed
- clippy and rustfmt components

## `devcontainer.json` Details

- Platform: `linux/arm64` (for ARM Mac hosts; change to `linux/amd64` for Intel).
- Container env: `RUST_BACKTRACE=1`, `CARGO_TARGET_DIR` set to `target-arm64` to keep container builds separate from host builds.
- Extensions: `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`.

## Cross-Platform Build Notes

The `CARGO_TARGET_DIR` separation is critical: host builds go to `codex-rs/target/` while container builds go to `codex-rs/target-arm64/` (or `target-amd64`). This prevents binary incompatibility issues when switching between host and container builds.

For x64 containers, use `--platform=linux/amd64` for both `docker build` and `docker run`, and manually add the x86 musl target: `rustup target add x86_64-unknown-linux-musl`.
