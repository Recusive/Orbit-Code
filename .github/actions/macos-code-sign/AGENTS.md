# .github/actions/macos-code-sign/

This file applies to `.github/actions/macos-code-sign/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Keep action inputs, outputs, and side effects compatible with the calling workflows. If you rename an input or artifact path, update every workflow that references this action.
- Shell steps should stay explicit and fail-fast; prefer `set -euo pipefail` in inline bash.

## Validate
- No dedicated local build step for this directory; validate by checking the workflows or callers that reference it.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Composite GitHub Action that signs and notarizes macOS release binaries and DMG installers using Apple's code signing and notarization infrastructure.

### Purpose

Handles the full macOS code signing lifecycle: configuring a temporary keychain with the Apple signing certificate, signing binaries with `codesign`, notarizing them with Apple's `notarytool`, and optionally signing/notarizing/stapling a DMG disk image. Cleans up the signing keychain on completion.

### Key Files

- **`action.yml`** -- Multi-step composite action with five steps:
  1. **Configure Apple code signing** -- Creates a temporary keychain, imports the P12 certificate, and extracts the signing identity hash.
  2. **Sign macOS binaries** -- Runs `codesign --force --options runtime --timestamp` on `codex` and `codex-responses-api-proxy` (conditional on `sign-binaries` input).
  3. **Notarize macOS binaries** -- Zips each binary and submits to Apple's notary service via `xcrun notarytool submit --wait` (conditional on `sign-binaries` input).
  4. **Sign and notarize macOS dmg** -- Signs the DMG with `codesign`, notarizes it, and staples the notarization ticket with `xcrun stapler staple` (conditional on `sign-dmg` input).
  5. **Remove signing keychain** -- Always runs to clean up the temporary keychain from the runner.

- **`notary_helpers.sh`** -- Bash helper sourced by the notarization steps. Provides the `notarize_submission()` function that wraps `xcrun notarytool submit` with JSON output parsing, status verification, and GitHub Actions notice annotations.

### Inputs

| Input | Required | Default | Description |
|-------|----------|---------|-------------|
| `target` | Yes | -- | Rust target triple (e.g., `aarch64-apple-darwin`) |
| `sign-binaries` | No | `true` | Whether to sign and notarize the CLI binaries |
| `sign-dmg` | No | `true` | Whether to sign and notarize the DMG installer |
| `apple-certificate` | Yes | -- | Base64-encoded P12 signing certificate |
| `apple-certificate-password` | Yes | -- | Password for the P12 certificate |
| `apple-notarization-key-p8` | Yes | -- | Base64-encoded P8 notarization key |
| `apple-notarization-key-id` | Yes | -- | Apple notarization key ID |
| `apple-notarization-issuer-id` | Yes | -- | Apple notarization issuer ID |

### Plugs Into

- Called by `rust-release.yml` build job twice for each macOS target: once for binaries (`sign-binaries=true, sign-dmg=false`) and once for the DMG (`sign-binaries=false, sign-dmg=true`).
- Referenced as `./.github/actions/macos-code-sign` in workflow files.
- Secrets are passed from the release workflow (`APPLE_CERTIFICATE_P12`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_NOTARIZATION_KEY_P8`, `APPLE_NOTARIZATION_KEY_ID`, `APPLE_NOTARIZATION_ISSUER_ID`).

### Environment Variables Set

- `APPLE_CODESIGN_IDENTITY` -- SHA-1 hash of the signing identity extracted from the keychain.
- `APPLE_CODESIGN_KEYCHAIN` -- Path to the temporary keychain created for the signing operation.
