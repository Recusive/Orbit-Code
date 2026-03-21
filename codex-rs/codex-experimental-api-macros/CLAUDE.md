# codex-rs/codex-experimental-api-macros/

Procedural macro crate for marking API fields and variants as experimental.

## What this folder does

Provides the `ExperimentalApi` derive macro that generates runtime introspection for experimental API fields. When a struct field or enum variant is annotated with `#[experimental("reason")]`, the macro generates an `experimental_reason()` method that returns the reason string when the experimental field is populated. Also supports `#[experimental(nested)]` for delegating to nested types, and registers experimental fields via the `inventory` crate for static enumeration.

## Where it plugs in

- Used by the `orbit-code-app-server-protocol` crate to mark experimental API protocol fields
- This is a `proc-macro` crate -- it cannot export non-macro items

## Imports from

- `proc-macro2` / `quote` / `syn` -- standard procedural macro infrastructure

## Exports to

- `#[derive(ExperimentalApi)]` -- derive macro for structs and enums
  - Generates `impl ExperimentalApi for T` with `fn experimental_reason(&self) -> Option<&'static str>`
  - Generates `T::EXPERIMENTAL_FIELDS` constant array for structs
  - Registers experimental fields with `inventory::submit!`

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Proc-macro crate manifest; depends on `proc-macro2`, `quote`, `syn` |
| `src/lib.rs` | Full macro implementation: `derive_experimental_api` entry point; `derive_for_struct` / `derive_for_enum`; field presence detection for `Option`, `Vec`, `HashMap`, `bool`; `snake_to_camel` serialized name conversion; `experimental_reason` / `has_nested_experimental` attribute parsing |
