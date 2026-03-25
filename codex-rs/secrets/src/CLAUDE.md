# codex-rs/secrets/src/

Encrypted secrets storage, backend abstraction, and output sanitization.

## Module Layout

- **lib** (`lib.rs`) -- Core types (`SecretName`, `SecretScope`, `SecretListEntry`, `SecretsBackendKind`), `SecretsBackend` trait, `SecretsManager` wrapper, keyring account computation, environment ID derivation
- **local** (`local.rs`) -- `LocalSecretsBackend`: age-encrypted file storage at `~/.orbit/secrets/local.age`, passphrase management via OS keyring, read-modify-write encryption cycle
- **sanitizer** (`sanitizer.rs`) -- `redact_secrets()`: regex-based redaction of OpenAI keys, AWS keys, bearer tokens, and secret assignment patterns
