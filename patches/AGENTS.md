# patches/

This file applies to `patches/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Patch files applied to third-party Rust dependencies during the build process. These fix compatibility issues in upstream crates that affect the Codex build, particularly in hermetic (Bazel) build environments.

### Key Files

| File | Role |
|------|------|
| `BUILD.bazel` | Empty Bazel build file (makes this directory a valid Bazel package) |
| `aws-lc-sys_memcmp_check.patch` | Patches the `aws-lc-sys` crate's `cc_builder.rs` to fix two issues in Bazel sandboxed builds: (1) strips debug flags (`-g`) from the memcmp probe compilation to avoid invoking `dsymutil` which may be missing, and (2) rewrites relative `bazel-out/` paths to absolute paths so linking succeeds when the process working directory differs from the Bazel execroot. |
| `windows-link.patch` | Patches the `windows-link` crate to replace `include_str!("../readme.md")` with a literal string, avoiding a build failure when the readme is not present in the expected location. |

### How Patches Are Applied

These patches are referenced in the Bazel build configuration (`MODULE.bazel`) and applied automatically during dependency fetching. The `aws-lc-sys` patch is particularly important for macOS Bazel builds where the sandbox environment differs from a normal `cargo build`.

### Relationship to Other Directories

- Referenced by `MODULE.bazel` at the repo root
- Fixes build issues for crates used by the `codex-rs/` Rust workspace
