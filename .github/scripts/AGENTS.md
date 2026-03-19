# .github/scripts/

This file applies to `.github/scripts/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Workflow and automation changes should be validated against their callers. Prefer small, explicit changes to job names, permissions, and artifact paths.

## Validate
- No dedicated local build step for this directory; validate by checking the workflows or callers that reference it.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Shell scripts used by CI workflows for build environment setup.

### Purpose

Contains helper scripts that prepare CI runner environments with the necessary build tools, primarily for cross-compiling Rust code targeting musl-based Linux.

### Key Files

- **`install-musl-build-tools.sh`** -- Comprehensive build environment setup script for musl-based Linux cross-compilation. This script:
  1. Installs system packages via `apt-get`: `musl-tools`, `pkg-config`, `libcap-dev`, `clang`, `lld`, `xz-utils`, and more.
  2. Downloads and builds `libcap` (v2.75) from source as a static library for musl, generating a pkg-config `.pc` file.
  3. Configures the C/C++ compiler toolchain:
     - If Zig is available, creates wrapper scripts (`zigcc`/`zigcxx`) that translate Rust target triples to Zig target format, filter problematic flags (e.g., `--target`, `-I/usr/include`, `-Wp,-U_FORTIFY_SOURCE`), and invoke `zig cc`/`zig c++`.
     - Falls back to native musl-gcc/g++ if Zig is not present.
  4. Exports environment variables to `$GITHUB_ENV`: `CC`, `CXX`, `CFLAGS`, `CXXFLAGS`, `CARGO_TARGET_*_LINKER`, `CMAKE_C_COMPILER`, `CMAKE_CXX_COMPILER`, `CMAKE_ARGS`, `PKG_CONFIG_PATH`, `PKG_CONFIG_ALLOW_CROSS`, `BORING_BSSL_SYSROOT`, and target-specific variants.

  Supports both `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets. Requires the `TARGET` environment variable to be set.

### Plugs Into

- Called by `rust-ci.yml` (lint_build job, musl targets) and `rust-release.yml` (build job, musl targets).
- Referenced as `$GITHUB_WORKSPACE/.github/scripts/install-musl-build-tools.sh` in workflow files.

### Imports / Dependencies

- Expects `TARGET` and `GITHUB_ENV` environment variables to be set.
- Optionally uses Zig (installed separately via `mlugg/setup-zig` in workflows).
- Downloads `libcap` source from `mirrors.edge.kernel.org`.
