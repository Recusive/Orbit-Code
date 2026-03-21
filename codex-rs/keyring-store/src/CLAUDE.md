# codex-rs/keyring-store/src/

Source for the `orbit-code-keyring-store` crate -- single-file OS keyring trait with default and mock implementations.

## Module Layout
- **Single file** (`lib.rs`): `KeyringStore` trait (load/save/delete); `DefaultKeyringStore` using `keyring::Entry` for platform-native storage; `CredentialStoreError` wrapping `keyring::Error`; public `tests` module with `MockKeyringStore` backed by `HashMap` for downstream test support
