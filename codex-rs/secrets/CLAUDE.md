# codex-rs/secrets/

Encrypted secrets management: age-encrypted storage with OS keyring passphrase, scoped secrets (global vs per-environment), and best-effort output redaction.

## Build & Test
```bash
cargo build -p orbit-code-secrets
cargo test -p orbit-code-secrets
```

## Architecture

`SecretsManager` is the high-level API for storing/retrieving/deleting secrets, backed by the `SecretsBackend` trait. The default `LocalSecretsBackend` stores all secrets in a single age-encrypted file (`~/.codex/secrets/local.age`) as a JSON `BTreeMap`. The encryption passphrase is generated randomly and persisted in the OS keyring via `orbit-code-keyring-store`.

Secrets are scoped via `SecretScope` -- either `Global` or `Environment(id)`, where the environment ID is derived from the git repo name or a SHA-256 hash of the working directory. The `sanitizer` module provides `redact_secrets()` for best-effort redaction of common secret patterns (API keys, bearer tokens, AWS keys) from text output.

## Key Considerations

- `SecretName` validation is strict: uppercase A-Z, digits, and underscores only. Invalid names are rejected at parse time.
- The secrets file uses a read-modify-write pattern (decrypt, mutate, re-encrypt, write) -- not concurrency-safe across multiple processes.
- `redact_secrets()` uses lazy-compiled regex patterns and is designed for best-effort output sanitization, not security-critical redaction.
- The `environment_id_from_cwd()` function prefers git repo name over SHA-256 hash -- falls back to hashing when not in a git repo.
- Tests that touch the keyring may behave differently on CI vs local (OS keyring availability).
