# codex-rs/feedback/

This file applies to `codex-rs/feedback/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-feedback` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-feedback`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-feedback` -- User feedback collection and upload for the Codex CLI.

### What this crate does

Provides a ring-buffer-based log capture system and Sentry-based feedback upload mechanism. When users report bugs or rate their experience, this crate packages up recent logs, connectivity diagnostics, and metadata tags, then uploads them to Sentry for triage.

### Main types and functions

- `CodexFeedback` -- Central feedback collector:
  - Contains a 4 MB ring buffer for captured logs and a tag map for structured metadata
  - `logger_layer()` -- Returns a `tracing_subscriber` layer that captures full-fidelity logs (TRACE level) into the ring buffer
  - `metadata_layer()` -- Returns a layer that collects key/value tags from events with target `"feedback_tags"`
  - `snapshot(session_id)` -- Takes a point-in-time snapshot of logs and tags
- `FeedbackSnapshot` -- Immutable snapshot of captured feedback:
  - `upload_feedback(classification, reason, include_logs, extra_attachments, session_source, logs_override)` -- Uploads to Sentry with classification tags and optional log/diagnostic attachments
  - `save_to_temp_file()` -- Saves logs to a temp file for local debugging
- `FeedbackMakeWriter` / `FeedbackWriter` -- `tracing_subscriber::fmt::MakeWriter` implementation that writes into the ring buffer
- `FeedbackDiagnostics` (in `feedback_diagnostics.rs`) -- Collects connectivity diagnostics from environment variables (OPENAI_BASE_URL, proxy vars)

### Key behaviors

- **Ring buffer**: Capped at 4 MB by default; oldest bytes are evicted when full
- **Sentry upload**: Uses a hardcoded DSN; 10-second flush timeout
- **Tag collection**: Up to 64 metadata tags from tracing events with target `"feedback_tags"`
- **Classifications**: "bug", "bad_result", "good_result", "safety_check"

### What it plugs into

- Integrated into the TUI and CLI startup via `tracing_subscriber` layers
- Called when users invoke feedback commands (thumbs up/down, bug report)
- `codex-core` and `codex-tui` emit `feedback_tags` tracing events to attach metadata

### Imports from / exports to

**Dependencies:**
- `codex-protocol` -- `ThreadId`, `SessionSource` types
- `sentry` -- Error reporting/upload
- `tracing`, `tracing-subscriber` -- Log capture

**Exports:**
- `CodexFeedback`, `FeedbackSnapshot`, `FeedbackMakeWriter`, `FeedbackWriter`
- `feedback_diagnostics::FeedbackDiagnostics`, `FeedbackDiagnostic`

### Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Main implementation: ring buffer, feedback collector, Sentry upload
- `src/feedback_diagnostics.rs` -- Connectivity diagnostics collection from environment
