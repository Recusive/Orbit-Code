# codex-rs/codex-experimental-api-macros/src/

Source directory for the `orbit-code-experimental-api-macros` proc-macro crate.

## What this folder does

Contains the single-file implementation of the `ExperimentalApi` derive macro.

## Key files

| File | Role |
|------|------|
| `lib.rs` | `derive_experimental_api` proc macro: parses `#[experimental("reason")]` and `#[experimental(nested)]` attributes on struct fields and enum variants; generates `ExperimentalApi` trait impl with `experimental_reason()` method; generates `EXPERIMENTAL_FIELDS` constant; handles `Option<T>`, `Vec<T>`, `HashMap<K,V>`, and `bool` presence detection; snake_to_camel field name conversion for serialized names |
