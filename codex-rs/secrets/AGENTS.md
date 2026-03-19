# codex-rs/secrets/

This file applies to `codex-rs/secrets/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-secrets` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-secrets`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-secrets` -- Encrypted secrets management for the Codex CLI.

### What this crate does

Provides a secrets management system that allows users to store, retrieve, and delete encrypted secrets (API keys, tokens, etc.). Secrets are encrypted at rest using `age` (scrypt-based) encryption, with the encryption passphrase stored in the OS keyring. Also provides a best-effort secret redaction utility for sanitizing output.

### Main types and functions

- `SecretsManager` -- High-level API for managing secrets:
  - `set(scope, name, value)` -- Store a secret
  - `get(scope, name)` -- Retrieve a secret
  - `delete(scope, name)` -- Delete a secret
  - `list(scope_filter)` -- List all secrets, optionally filtered by scope
- `SecretName` -- Validated secret name (uppercase A-Z, digits, underscores only)
- `SecretScope` -- Scoping enum: `Global` or `Environment(id)` (per-project secrets)
- `SecretListEntry` -- Entry in a secret listing (scope + name)
- `SecretsBackendKind` -- Backend selection enum (currently only `Local`)
- `SecretsBackend` trait -- Backend abstraction with `set`, `get`, `delete`, `list`
- `LocalSecretsBackend` -- Default backend that stores secrets in an age-encrypted file (`local.age`) under `~/.codex/secrets/`
- `redact_secrets(input: String) -> String` -- Best-effort redaction of secrets from text output using regex patterns

### Key behaviors

- **Encryption**: Uses `age` crate with scrypt recipient/identity; passphrase stored in OS keyring via `codex-keyring-store`
- **Storage**: Secrets stored as a JSON `BTreeMap<String, String>` inside an age-encrypted file
- **Scoping**: Secrets can be global or scoped to an environment (derived from git repo name or cwd hash)
- **Redaction patterns**: OpenAI API keys (`sk-*`), AWS access keys (`AKIA*`), Bearer tokens, and generic secret/password/token assignments

### What it plugs into

- Used by `codex-core` to inject secrets as environment variables during command execution
- Used by `codex-cli` for the `codex secret` subcommand (set/get/delete/list)
- The `redact_secrets` function is used to sanitize model output and logs

### Imports from / exports to

**Dependencies:**
- `age` -- Encryption/decryption
- `codex-keyring-store` -- OS keyring for passphrase storage
- `base64` -- Encoding
- `rand` -- Secure random passphrase generation
- `regex` -- Secret redaction patterns
- `schemars` -- JSON schema generation for `SecretsBackendKind`
- `serde`, `serde_json` -- Serialization of the secrets file
- `sha2` -- Hashing for keyring account derivation

**Exports:**
- `SecretsManager`, `SecretName`, `SecretScope`, `SecretListEntry`, `SecretsBackendKind`, `SecretsBackend`, `LocalSecretsBackend`
- `redact_secrets`, `environment_id_from_cwd`

### Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Core types, `SecretsManager`, scope/name validation, keyring account computation
- `src/local.rs` -- `LocalSecretsBackend` with age encryption, keyring passphrase management
- `src/sanitizer.rs` -- `redact_secrets` function with regex-based pattern matching
