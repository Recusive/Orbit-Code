# .github/workflows/

This file applies to `.github/workflows/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Workflow and automation changes should be validated against their callers. Prefer small, explicit changes to job names, permissions, and artifact paths.

## Validate
- No dedicated local build step for this directory; validate by checking the workflows or callers that reference it.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

GitHub Actions workflow definitions for CI, release pipelines, issue automation, and repository maintenance.

### Purpose

Contains all GitHub Actions workflows that power the repository's continuous integration, multi-platform release pipeline, AI-assisted issue management, and housekeeping automation.

### Workflow Categories

#### CI / Build Validation

| File | Trigger | Description |
|------|---------|-------------|
| `ci.yml` | PR, push to main | Node.js CI: installs pnpm, stages npm packages, validates README ASCII/ToC, runs Prettier formatting checks. |
| `rust-ci.yml` | PR, push to main | Rust CI: detects changed paths, runs `cargo fmt`, `cargo shear`, argument comment lint, `cargo clippy` (lint/build), and `cargo nextest` (tests) across a matrix of 8+ target/runner combinations (macOS, Linux gnu/musl, Windows, x86_64/aarch64). Uses sccache for build caching. Gated by change detection to skip when irrelevant files change. |
| `bazel.yml` | PR, push to main, manual | Experimental Bazel build/test across macOS and Linux targets. Uses BuildBuddy for remote execution/caching when API key is available, falls back to local builds for forks. |
| `sdk.yml` | PR, push to main | Builds, lints, and tests TypeScript SDK packages under `sdk/typescript/`. |
| `shell-tool-mcp-ci.yml` | Push/PR touching `shell-tool-mcp/` paths | CI for the shell-tool-mcp package: format check, tests, and build. |
| `cargo-deny.yml` | PR, push to main | Runs `cargo-deny` to check Rust dependency licenses and advisories. |
| `codespell.yml` | PR, push to main | Spell-checks the codebase using codespell. |
| `blob-size-policy.yml` | PR | Enforces a 512 KB max blob size on PR changes, with an allowlist at `.github/blob-size-allowlist.txt`. |

#### Release Pipelines

| File | Trigger | Description |
|------|---------|-------------|
| `rust-release.yml` | Tag push (`rust-v*.*.*`) | Full release pipeline: validates tag matches Cargo.toml version, builds release binaries for 6 Linux/macOS targets, invokes Windows build and shell-tool-mcp sub-workflows, signs artifacts (cosign on Linux, Apple codesign on macOS), creates DMGs, compresses with zstd/tar.gz, stages npm packages, creates GitHub Release, publishes to npm (OIDC), publishes to WinGet, updates DotSlash, triggers developers.openai.com deploy, and updates the `latest-alpha-cli` branch. |
| `rust-release-windows.yml` | Called by `rust-release.yml` | Windows-specific release build: compiles primary binaries and helper binaries (sandbox setup, command runner) in parallel across x86_64 and aarch64, signs with Azure Trusted Signing, stages and compresses artifacts (zstd, tar.gz, zip). |
| `rust-release-prepare.yml` | Schedule (every 4 hours), manual | Fetches latest `models.json` from the OpenAI API and opens a PR if it changed. |
| `shell-tool-mcp.yml` | Called by `rust-release.yml` | Builds patched Bash and zsh binaries across multiple Linux distros (Ubuntu, Debian, CentOS) and macOS versions, packages them into an npm module, and publishes to npm. |

#### Issue / PR Automation

| File | Trigger | Description |
|------|---------|-------------|
| `issue-labeler.yml` | Issue opened/labeled | Uses the Codex GitHub Action to auto-label new issues based on content analysis (bug/enhancement/docs, product surface, topic tags). |
| `issue-deduplicator.yml` | Issue opened/labeled | Two-pass duplicate detection using the Codex GitHub Action: first searches all issues (open + closed), then falls back to open-only. Posts a comment listing potential duplicates. |
| `cla.yml` | PR opened/synced/closed, issue comment | CLA Assistant: manages Contributor License Agreement signing via `contributor-assistant/github-action`. Only runs on the canonical `openai/codex` repo. |
| `close-stale-contributor-prs.yml` | Daily at 06:00 UTC, manual | Closes PRs from contributors (admin/maintain/write) that have been inactive for 14+ days. |

#### Supporting Files

| File | Description |
|------|-------------|
| `ci.bazelrc` | Bazel configuration for CI: enables remote download minimal, keep-going, verbose failures, platform-specific remote execution strategies (full remote on Linux, remote build + local test on macOS). |
| `Dockerfile.bazel` | Docker image for Bazel CI: Ubuntu 24.04 base with Node.js, DotSlash, curl, git, Python 3. Published for both amd64 and arm64 architectures. |
| `zstd` | DotSlash script that wraps the `zstd` compression tool for Windows runners (downloads zstd v1.5.7 from the facebook/zstd GitHub release). Used by `rust-release-windows.yml`. |

### Shared Patterns

- **Change detection**: `rust-ci.yml` uses a dedicated `changed` job to detect which areas of the codebase were modified and conditionally skips irrelevant jobs.
- **Matrix builds**: Release and CI workflows use strategy matrices across multiple OS/target combinations.
- **sccache**: Rust CI and release workflows use sccache with GitHub Actions cache backend (or local disk fallback) to accelerate repeated builds.
- **Concurrency groups**: Release workflows use concurrency groups with `cancel-in-progress` to prevent parallel builds.
- **Fork safety**: Workflows requiring secrets (issue automation, CLA, etc.) gate on `github.repository == 'openai/codex'` to prevent failures on forks.

### Imports / Dependencies

- Composite actions from `.github/actions/` (code signing).
- Scripts from `.github/scripts/` (musl build tools).
- Bazel config from `ci.bazelrc`.
- External actions: `actions/checkout`, `actions/setup-node`, `dtolnay/rust-toolchain`, `pnpm/action-setup`, `openai/codex-action`, `sigstore/cosign-installer`, `azure/login`, `azure/trusted-signing-action`, `softprops/action-gh-release`, `facebook/dotslash-publish-release`, `peter-evans/create-pull-request`, `contributor-assistant/github-action`, and others.
