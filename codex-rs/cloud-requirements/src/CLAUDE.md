# codex-rs/cloud-requirements/src/

Source directory for the `codex-cloud-requirements` crate.

## What this folder does

Contains the single-file implementation of the cloud requirements loading system. All logic is in `lib.rs`.

## Key files

| File | Role |
|------|------|
| `lib.rs` | Complete implementation: `CloudRequirementsService` (fetch with retries, HMAC-signed cache read/write, background refresh); `BackendRequirementsFetcher` (HTTP fetch via `codex-backend-client`); `RequirementsFetcher` trait for testability; cache types (`CloudRequirementsCacheFile`, `CloudRequirementsCacheSignedPayload`); auth recovery on 401; metrics emission; comprehensive test suite with `StaticFetcher`, `PendingFetcher`, `SequenceFetcher`, `TokenFetcher`, `UnauthorizedFetcher` mocks |
